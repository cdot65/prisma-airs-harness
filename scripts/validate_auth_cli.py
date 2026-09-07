# /// script
# requires-python = ">=3.12"
# dependencies = ["requests==2.32.5"]
# ///
"""Operator fixture: real native CLI, two resource logins and actual gateway tools.
All bearer credentials, passwords and authorization codes stay in memory.
"""

import argparse, base64, importlib.util, json, os, re, secrets, select, subprocess, sys, tempfile, time, uuid
from pathlib import Path
from urllib.parse import urlsplit, urljoin
import requests

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--binary", type=Path, required=True)
parser.add_argument("--infrastructure-root", type=Path, required=True)
parser.add_argument(
    "--management-module",
    type=Path,
    required=True,
    help="Operator-only Node module exporting an authenticated AIRS SDK client as gw",
)
parser.add_argument("--output", type=Path, required=True)
parser.add_argument(
    "--observe-tools",
    action="store_true",
    help="Diagnostic loopback relay records only tool names and model-key presence",
)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(root / "scripts"))
from validate_rust_oidc import Form

p = args.infrastructure_root.resolve() / "keycloak/stacks/terminal/validate-refresh.py"
spec = importlib.util.spec_from_file_location("protocol", p)
protocol = importlib.util.module_from_spec(spec)
spec.loader.exec_module(protocol)
op = protocol.Operator()
issuer = protocol.ISSUER
binary = args.binary.resolve()
clients = [
    op.api("/clients?clientId=" + name)[0]
    for name in ["airs-terminal-pilot", "airs-terminal-mcp"]
]
assert all(not c["enabled"] for c in clients), "Clients must start disabled"
users = []
rows = []
daemon = None
current = None
mcpurl = "https://mcp-airs.cdot.io/ws-prisma-ff3d74/airs-terminal-runtime-scanner/mcp"


def check(name, condition, **details):
    row = {"test": name, "passed": bool(condition), **details}
    rows.append(row)
    print(json.dumps(row), flush=True)
    assert condition, "Acceptance failed: " + name


def server_access(access):
    code = (
        "import {gw} from "
        + json.dumps(args.management_module.resolve().as_uri())
        + "; await gw.mcpServers.updateUserAccess('1a3dcdd1-7c1e-473a-b7c4-ec78720bd0cc',{default_user_access:"
        + json.dumps(access)
        + "});"
    )
    subprocess.run(
        ["node", "--input-type=module", "-e", code],
        check=True,
        stdout=subprocess.DEVNULL,
    )


with tempfile.TemporaryDirectory(prefix="airs-cli-auth-") as tmp:
    tmp = Path(tmp)
    home = tmp / "state"
    work = tmp / "work"
    work.mkdir()
    env = dict(os.environ, AIRS_TERMINAL_HOME=str(home), BROWSER="/bin/true")
    for name in ["data", "config", "runtime"]:
        (tmp / name).mkdir(mode=0o700)
    env.update(
        XDG_DATA_HOME=str(tmp / "data"),
        XDG_CONFIG_HOME=str(tmp / "config"),
        XDG_RUNTIME_DIR=str(tmp / "runtime"),
    )

    def run(*args, timeout=120):
        return subprocess.run(
            [str(binary), *args],
            env=env,
            cwd=work,
            capture_output=True,
            text=True,
            timeout=timeout,
        )

    def authenticate(args, user, device=False, expect_success=True):
        global current
        current = subprocess.Popen(
            [str(binary), *args],
            env=env,
            cwd=work,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        deadline = time.monotonic() + 45
        url = None
        code = None
        # read one byte at a time to avoid TextIO prefetch defeating select() on a buffered pipe.
        line = ""
        while time.monotonic() < deadline and url is None:
            ready, _, _ = select.select([current.stderr], [], [], 1)
            if not ready:
                if current.poll() is not None:
                    break
                continue
            char = os.read(current.stderr.fileno(), 1).decode(errors="replace")
            if not char:
                break
            line += char
            if char == "\n":
                if line.startswith("https://"):
                    url = line.strip()
                if line.startswith("Open https://"):
                    match = re.fullmatch(
                        r"Open (https://\S+) and enter code (\S+)\n", line
                    )
                    if match:
                        url, code = match.groups()
                line = ""
        assert url, (
            "CLI did not produce a login instruction (credential/command setup failure)"
        )
        assert urlsplit(url).netloc == "auth.dev.cdot.io"
        session = requests.Session()
        r = session.get(
            url,
            params={"user_code": code} if device else None,
            timeout=20,
            allow_redirects=False,
        )
        for _ in range(12):
            if r.status_code in (302, 303):
                target = urljoin(r.url, r.headers["Location"])
                parsed = urlsplit(target)
                if parsed.hostname == "127.0.0.1" and not device:
                    assert parsed.scheme == "http" and parsed.path == "/callback"
                    r = session.get(target, timeout=20, allow_redirects=False)
                    assert r.status_code == 200
                    break
                assert parsed.netloc == "auth.dev.cdot.io"
                r = session.get(target, timeout=20, allow_redirects=False)
                continue
            form = Form(r.text)
            if not form.action:
                assert device
                break
            action = urljoin(r.url, form.action)
            assert urlsplit(action).netloc == "auth.dev.cdot.io"
            r = session.post(
                action,
                data={
                    **form.fields,
                    "username": user["username"],
                    "password": user["password"],
                    "accept": "Yes",
                    "user_code": code or "",
                },
                timeout=20,
                allow_redirects=False,
            )
        output, error = current.communicate(timeout=60)
        success = current.returncode == 0
        current = None
        # CLI errors use static messages; never print form redirects or credentials.
        if success != expect_success:
            print("CLI outcome mismatch: " + error[-500:], flush=True)
        return success

    try:
        daemon = subprocess.Popen(
            [
                "gnome-keyring-daemon",
                "--foreground",
                "--unlock",
                "--components=secrets",
            ],
            env=env,
            stdin=subprocess.PIPE,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            text=True,
        )
        daemon.stdin.write(secrets.token_urlsafe(48))
        daemon.stdin.close()
        for _ in range(50):
            r = subprocess.run(
                [
                    "dbus-send",
                    "--session",
                    "--print-reply",
                    "--dest=org.freedesktop.DBus",
                    "/org/freedesktop/DBus",
                    "org.freedesktop.DBus.NameHasOwner",
                    "string:org.freedesktop.secrets",
                ],
                env=env,
                capture_output=True,
                text=True,
            )
            if "boolean true" in r.stdout:
                break
            time.sleep(0.2)
        else:
            raise AssertionError("Isolated credential store did not start")
        for _ in range(2):
            u = {
                "username": "airs-cli-acceptance-" + secrets.token_hex(8),
                "password": secrets.token_urlsafe(32) + "Aa1!",
            }
            op.api(
                "/users",
                "POST",
                {
                    "username": u["username"],
                    "enabled": True,
                    "emailVerified": True,
                    "email": u["username"] + "@example.invalid",
                    "firstName": "CLI",
                    "lastName": "Acceptance",
                    "credentials": [
                        {"type": "password", "value": u["password"], "temporary": False}
                    ],
                },
            )
            u["id"] = op.api("/users?exact=true&username=" + u["username"])[0]["id"]
            users.append(u)
            for client, role_name in zip(clients, ["terminal-user", "scanner-user"]):
                role = op.api("/clients/" + client["id"] + "/roles/" + role_name)
                op.api(
                    "/users/" + u["id"] + "/role-mappings/clients/" + client["id"],
                    "POST",
                    [role],
                )
        for client in clients:
            op.api("/clients/" + client["id"], "PUT", {"enabled": True})
        server_access("allow")
        gateway = "https://airs.cdot.io/v1"
        setup_extra = []
        if args.observe_tools:
            import threading
            from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

            class Observer(BaseHTTPRequestHandler):
                def log_message(self, *args):
                    pass

                def do_POST(self):
                    body = self.rfile.read(int(self.headers.get("Content-Length", "0")))
                    obj = json.loads(body)
                    catalog = []
                    for tool in obj.get("tools", []):
                        catalog.append(
                            {
                                "type": tool.get("type"),
                                "name": tool.get("name"),
                                "members": [
                                    t.get("name") for t in tool.get("tools", [])
                                ],
                            }
                        )
                    print(
                        json.dumps(
                            {
                                "diagnostic": "inference-tool-catalog",
                                "tools": catalog,
                                "has_model_key": "model" in obj,
                            }
                        ),
                        flush=True,
                    )
                    headers = {
                        k: v
                        for k, v in self.headers.items()
                        if k.lower() not in ("host", "content-length", "connection")
                    }
                    r = requests.post(
                        "https://airs.cdot.io" + self.path,
                        data=body,
                        headers=headers,
                        timeout=120,
                        allow_redirects=False,
                    )
                    self.send_response(r.status_code)
                    self.send_header(
                        "Content-Type",
                        r.headers.get("Content-Type", "application/json"),
                    )
                    self.send_header("Content-Length", str(len(r.content)))
                    self.end_headers()
                    self.wfile.write(r.content)

            observer = ThreadingHTTPServer(("127.0.0.1", 0), Observer)
            threading.Thread(target=observer.serve_forever, daemon=True).start()
            gateway = f"http://127.0.0.1:{observer.server_port}/v1"
            setup_extra = ["--allow-http-loopback"]
        check(
            "setup",
            run("setup", "--gateway-url", gateway, *setup_extra).returncode == 0,
        )
        login = [
            "login",
            "--issuer-url",
            issuer,
            "--oidc-client-id",
            "airs-terminal-pilot",
            "--audience",
            "airs-terminal-inference",
        ]
        check("browser-login", authenticate(login, users[0]))
        binding = json.loads((home / "credential-binding.json").read_text())
        check(
            "verified-subject",
            binding["source"]["identity"]["subject"] == users[0]["id"],
        )
        credential = ["credential", "--home", str(home), "--binding", binding["id"]]
        first = run(*credential)
        check("credential-helper", first.returncode == 0)
        jwt = first.stdout.strip()
        claims = json.loads(base64.urlsafe_b64decode(jwt.split(".")[1] + "=="))
        check(
            "original-resource-jwt",
            claims["sub"] == users[0]["id"]
            and claims["aud"] == "airs-terminal-inference",
        )
        mcp = [
            "setup-mcp",
            "--name",
            "security",
            "--url",
            mcpurl,
            "--issuer-url",
            issuer,
            "--oidc-client-id",
            "airs-terminal-mcp",
            "--audience",
            "airs-terminal-security",
            "--tool",
            "pan_inline_scan",
            "--required",
        ]
        check(
            "different-mcp-user-rejected",
            not authenticate(mcp, users[1], expect_success=False),
        )
        check("mcp-login", authenticate(mcp, users[0]))
        mb = json.loads(next((home / "mcp-bindings").glob("*.json")).read_text())
        helper = ["mcp-credential", "--home", str(home), "--binding", mb["id"]]
        r = run(*helper)
        check("mcp-credential-helper", r.returncode == 0)
        mjwt = json.loads(r.stdout)["x-portkey-api-key"]
        mc = json.loads(base64.urlsafe_b64decode(mjwt.split(".")[1] + "=="))
        check(
            "separate-mcp-audience",
            mc["aud"] == "airs-terminal-security"
            and mc["sub"] == claims["sub"]
            and mjwt != jwt,
        )
        result = run(
            "exec",
            "--skip-git-repo-check",
            "--sandbox",
            "workspace-write",
            "--json",
            "Use a local shell tool to write auth-proof.txt containing exactly OIDC_LOCAL_OK. Then call the executable mcp__security.pan_inline_scan tool with scan_request.profile Prisma AIRS Terminal and scan_request.response Hello from AIRS Terminal. This is a tools/call operation, not read_mcp_resource; do not use resource functions to invoke a tool. Report the actual action and scan_id. Do both tools now.",
            timeout=180,
        )
        # Save only this synthetic run's transcript in a private local file for diagnosis.
        diagnostic = args.output.resolve().with_suffix(".private-exec.log")
        diagnostic.write_text(result.stdout + "\n" + result.stderr)
        diagnostic.chmod(0o600)
        check(
            "exec-local-filesystem",
            result.returncode == 0
            and (work / "auth-proof.txt").exists()
            and (work / "auth-proof.txt").read_text().strip() == "OIDC_LOCAL_OK",
            exit_code=result.returncode,
        )
        events = []
        for line in result.stdout.splitlines():
            try:
                events.append(json.loads(line))
            except ValueError:
                pass
        calls = [
            e.get("item", {})
            for e in events
            if e.get("item", {}).get("type") == "mcp_tool_call"
        ]
        scans = [
            (c.get("result") or {}).get("structured_content", {}).get("results", {})
            for c in calls
            if c.get("status") == "completed"
            and c.get("tool") == "pan_inline_scan"
            and c.get("error") is None
        ]
        scan = next(
            (
                s
                for s in scans
                if s.get("action") == "allow"
                and s.get("scan_id")
                and s.get("profile_name") == "Prisma AIRS Terminal"
            ),
            None,
        )
        check("exec-remote-mcp", scan is not None, tool_calls=len(calls), scan=scan)
        # Real expiration interval: validate persisted rotation across two helper processes.
        while time.time() < claims["exp"] - 25:
            time.sleep(min(2, claims["exp"] - 25 - time.time()))
        r = run(*credential)
        check(
            "refresh-after-expiry-window", r.returncode == 0 and r.stdout.strip() != jwt
        )
        refreshed = r.stdout.strip()
        rc = json.loads(base64.urlsafe_b64decode(refreshed.split(".")[1] + "=="))
        check(
            "refresh-preserves-identity",
            rc["sub"] == claims["sub"]
            and json.loads((home / "credential-binding.json").read_text())[
                "credential_fingerprint"
            ]
            == binding["credential_fingerprint"],
        )
        check("logout", run("logout").returncode == 0)
        check(
            "logout-disables-both-helpers",
            run(*credential).returncode != 0 and run(*helper).returncode != 0,
        )
        check(
            "different-inference-user-rejected",
            not authenticate(login, users[1], expect_success=False),
        )
        check(
            "device-reauthentication",
            authenticate([*login, "--device-auth"], users[0], device=True),
        )
        check(
            "same-user-history-binding",
            json.loads((home / "credential-binding.json").read_text())["id"]
            == binding["id"],
        )
        check("mcp-remains-logged-out", run(*helper).returncode != 0)
        check(
            "mcp-device-reauthentication",
            authenticate([*mcp, "--device-auth"], users[0], device=True),
        )
        check(
            "same-mcp-binding",
            json.loads(next((home / "mcp-bindings").glob("*.json")).read_text())["id"]
            == mb["id"],
        )
        check("final-logout", run("logout").returncode == 0)
    finally:
        if current and current.poll() is None:
            current.terminate()
            current.communicate(timeout=10)
        for client in clients:
            op.api("/clients/" + client["id"], "PUT", {"enabled": False})
        for u in users:
            op.api("/users/" + u["id"], "DELETE")
        server_access("deny")
        if daemon:
            daemon.terminate()
            daemon.wait(timeout=10)
        out = args.output.resolve()
        out.write_text(
            json.dumps(
                {
                    "passed": len(rows) >= 20 and all(r["passed"] for r in rows),
                    "checks": rows,
                    "fixtures_removed": True,
                    "clients_disabled": True,
                    "scanner_default_access": "deny",
                },
                indent=2,
            )
            + "\n"
        )
