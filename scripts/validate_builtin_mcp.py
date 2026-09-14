#!/usr/bin/env python3
"""Exercise the installed Codex MCP client through real browser PKCE and gateway tools.

Use a disposable identity authorized by the selected MCP service. Linux requires
an isolated dbus-run-session. macOS requires the logged-in native runner session.
No owner credentials or external helper executables are used.
"""

import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import html
import http.cookiejar
import json
import os
from pathlib import Path
import queue
import re
import secrets
import subprocess
import threading
import time
import tomllib
import urllib.error
import urllib.parse
import urllib.request

ISSUER = "https://auth.redtail.cdot.io/realms/redtail"
SCOPES = {"airs.gateway.read", "airs.profiles.read"}
TOOLS = {
    "list_workspaces",
    "get_workspace",
    "list_gateway_configs",
    "get_gateway_config",
    "list_gateway_guardrails",
    "get_gateway_guardrail",
    "list_security_profiles",
    "get_security_profile",
}


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def browser(authorize, user):
    assert authorize.startswith(ISSUER + "/protocol/openid-connect/auth?")
    params = urllib.parse.parse_qs(urllib.parse.urlparse(authorize).query)
    redirect = urllib.parse.urlparse(params["redirect_uri"][0])
    assert (
        redirect.scheme == "http"
        and redirect.hostname == "127.0.0.1"
        and redirect.path == "/callback"
    )
    opener = urllib.request.build_opener(
        urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()), NoRedirect
    )
    with opener.open(authorize, timeout=20) as response:
        page = response.read().decode()
    action = html.unescape(re.search(r'<form[^>]*action="([^"]+)"', page).group(1))
    assert action.startswith(ISSUER + "/login-actions/")
    try:
        opener.open(
            urllib.request.Request(
                action,
                data=urllib.parse.urlencode(
                    {
                        "username": user["username"],
                        "password": user["password"],
                        "credentialId": "",
                    }
                ).encode(),
            ),
            timeout=20,
        )
        raise RuntimeError("Expected login redirect")
    except urllib.error.HTTPError as error:
        assert error.code == 302
        callback = error.headers["Location"]
    actual = urllib.parse.urlparse(callback)
    assert (
        actual.scheme == redirect.scheme
        and actual.netloc == redirect.netloc
        and actual.path == redirect.path
    )
    with urllib.request.urlopen(callback, timeout=20) as response:
        assert response.status == 200


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--endpoint", default="https://prisma-airs-mcp-dev.cdot.io/mcp")
    parser.add_argument("--client", default="prisma-airs-harness-mcp-dev")
    parser.add_argument("--refresh-cycles", type=int, choices=[0, 1, 2], default=0)
    args = parser.parse_args()
    args.state.mkdir(parents=True, mode=0o700, exist_ok=False)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    user = json.loads(args.fixture.read_text())
    env = dict(os.environ, AIRS_HARNESS_HOME=str(args.state))
    if os.uname().sysname == "Linux":
        for key, name in [("XDG_DATA_HOME", "keyring"), ("XDG_RUNTIME_DIR", "runtime")]:
            env[key] = str(args.state / name)
            Path(env[key]).mkdir(mode=0o700)
        password = secrets.token_urlsafe(48).encode()
        subprocess.run(
            ["gnome-keyring-daemon", "--unlock", "--components=secrets"],
            input=password,
            env=env,
            capture_output=True,
            check=True,
        )
    rows = []

    def run(label, commands, authenticate=False, timeout=240):
        child = subprocess.Popen(
            [str(args.binary), *commands],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        lines = queue.Queue()
        output = []
        started = time.monotonic()

        def collect():
            for line in child.stdout:
                output.append(line)
                lines.put(line)
            lines.put(None)

        threading.Thread(target=collect, daemon=True).start()
        try:
            while True:
                line = lines.get(timeout=max(1, timeout - (time.monotonic() - started)))
                if line is None:
                    break
                match = re.search(
                    r"https://auth\.redtail\.cdot\.io/realms/redtail/protocol/openid-connect/auth\?\S+",
                    line,
                )
                if match and authenticate:
                    url = match.group(0)
                    params = urllib.parse.parse_qs(urllib.parse.urlparse(url).query)
                    assert params["code_challenge_method"] == ["S256"]
                    assert len(params["state"]) == 1
                    if label == "mcp_browser_pkce":
                        assert set(params["scope"][0].split()) == SCOPES
                        assert params["resource"] == [args.endpoint]
                        assert params["client_id"] == [args.client]
                    browser(url, user)
            code = child.wait(timeout=15)
        finally:
            if child.poll() is None:
                child.terminate()
                child.wait(timeout=10)
            diagnostic = args.output.with_name(
                args.output.stem + "." + label + ".private.log"
            )
            diagnostic.touch(mode=0o600)
            diagnostic.write_text("".join(output))
        row = {"case": label, "passed": code == 0, "exit_code": code}
        rows.append(row)
        print(json.dumps(row), flush=True)
        assert code == 0, label + " failed; inspect private diagnostic"
        return "".join(output)

    def model(label, prompt, expected):
        output = run(
            label,
            [
                "exec",
                "--skip-git-repo-check",
                "-c",
                "features.shell_tool=false",
                "--json",
                prompt,
            ],
        )
        events = []
        for line in output.splitlines():
            try:
                events.append(json.loads(line))
            except ValueError:
                pass
        calls = [
            e["item"]
            for e in events
            if isinstance(e.get("item"), dict)
            and e["item"].get("type") == "mcp_tool_call"
        ]
        completed = {
            c["tool"]
            for c in calls
            if c.get("server") == "prisma-airs"
            and c.get("status") == "completed"
            and not c.get("error")
            and not (c.get("result") or {}).get("isError", False)
        }
        assert expected <= completed, (
            f"{label}: missing successful tools {expected - completed}"
        )
        rows.append(
            {
                "case": label + "_tool_results",
                "passed": True,
                "tools": sorted(completed),
            }
        )
        return events

    passed = False
    mcp_added = False
    inference_logged_in = False
    try:
        run("setup", ["setup", "--gateway-url", "https://gateway.redtail.cdot.io/v1"])
        config = args.state / "config.toml"
        config.write_text(
            'mcp_oauth_credentials_store = "keyring"\n' + config.read_text()
        )
        run(
            "inference_browser_pkce",
            [
                "login",
                "--no-browser",
                "--issuer-url",
                ISSUER,
                "--oidc-client-id",
                "prisma-airs-harness",
                "--audience",
                "stack-airs-inference",
            ],
            True,
        )
        inference_logged_in = True
        run(
            "history_before_mcp",
            [
                "exec",
                "--skip-git-repo-check",
                "-c",
                "features.shell_tool=false",
                "--json",
                "Reply with the single word ready.",
            ],
        )
        binding = args.state / "session-binding.json"
        pinned = binding.read_bytes()
        run(
            "mcp_browser_pkce",
            [
                "mcp",
                "add",
                "prisma-airs",
                "--url",
                args.endpoint,
                "--oauth-client-id",
                args.client,
                "--scopes",
                ",".join(sorted(SCOPES)),
                "--no-browser",
            ],
            True,
        )
        mcp_added = True
        settings = tomllib.loads(config.read_text())["mcp_servers"]["prisma-airs"]
        assert settings["url"] == args.endpoint and set(settings["scopes"]) == SCOPES
        assert settings["oauth"]["client_id"] == args.client
        assert not any(
            k in settings
            for k in [
                "command",
                "http_headers",
                "http_headers_helper",
                "bearer_token_env_var",
            ]
        )
        listing = json.loads(run("mcp_list", ["mcp", "list", "--json"]))
        assert (
            next(s for s in listing if s["name"] == "prisma-airs")["auth_status"]
            == "o_auth"
        )
        assert not (args.state / ".credentials.json").exists(), (
            "File fallback used instead of native keyring"
        )
        run("doctor_after_mcp", ["doctor", "--verify-access"])
        model(
            "all_read_tools",
            "Use the prisma-airs MCP tools to inspect the authorized AIRS deployment. "
            "Call ALL eight tools: list_workspaces, get_workspace, list_gateway_configs, get_gateway_config, "
            "list_gateway_guardrails, get_gateway_guardrail, list_security_profiles, get_security_profile. "
            "Use IDs returned by the list calls for each get call. The workspace is ce06de57-3ccb-4a98-a653-bcc2b1de76ca "
            "and profile 032bbecd-5c44-4d30-a13e-bb742f6c9773 if needed. Report the names you actually retrieved. "
            "Do not use resource methods or shell commands.",
            TOOLS,
        )
        assert binding.read_bytes() == pinned, (
            "Adding native OAuth invalidated inference history binding"
        )
        rows.append({"case": "inference_history_preserved", "passed": True})
        for cycle in range(args.refresh_cycles):
            # Redtail access tokens expire in five minutes; independent CLI processes
            # after 310 seconds exercise the stored refresh credential without login.
            deadline = time.monotonic() + 310
            print(
                json.dumps(
                    {"case": "await_token_expiry", "cycle": cycle + 1, "seconds": 310}
                ),
                flush=True,
            )
            while time.monotonic() < deadline:
                time.sleep(min(30, max(0, deadline - time.monotonic())))

            def refresh(index):
                return model(
                    f"refresh_{cycle + 1}_{index}",
                    "Call prisma-airs list_workspaces and report the workspace name.",
                    {"list_workspaces"},
                )

            with ThreadPoolExecutor(max_workers=2) as pool:
                list(pool.map(refresh, range(2)))
            assert binding.read_bytes() == pinned
        passed = True
    finally:
        if mcp_added:
            try:
                run("mcp_logout", ["mcp", "logout", "prisma-airs"])
                listing = json.loads(
                    run("mcp_list_after_logout", ["mcp", "list", "--json"])
                )
                assert (
                    next(s for s in listing if s["name"] == "prisma-airs")[
                        "auth_status"
                    ]
                    != "o_auth"
                )
                run("inference_after_mcp_logout", ["doctor", "--verify-access"])
            except Exception:
                passed = False
        if inference_logged_in:
            try:
                run("inference_logout", ["logout"])
            except Exception:
                passed = False
        receipt = {
            "passed": passed,
            "binary_sha256": hashlib.sha256(
                args.binary.resolve().read_bytes()
            ).hexdigest(),
            "platform": os.uname().sysname,
            "endpoint": args.endpoint,
            "identity": "disposable human; native browser PKCE and OS credential store",
            "refresh_cycles": args.refresh_cycles,
            "results": rows,
        }
        args.output.write_text(json.dumps(receipt, indent=2) + "\n")
    assert passed, "Acceptance or logout failed"


if __name__ == "__main__":
    main()
