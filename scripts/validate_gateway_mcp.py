#!/usr/bin/env python3
"""Validate native gateway MCP with interactive SSO and separately observed gateway evidence.

Run Linux under dbus-run-session. --browser-via-ssh opens the browser on the
Apple Silicon desktop and forwards its loopback callback to this process.
No user password is collected. Logs containing authorization URLs remain private.
"""

import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import secrets
import shlex
import subprocess
import threading
import time
import tomllib
from urllib.parse import parse_qs, urlsplit

from airs_gateway_test_identity import GatewayTestIdentity
from airs_gateway_cleanup import wait_for_cleanup_release
from promote_airs_mcp_prerelease import TOOLS
from airs_gateway_expiry import (
    ACTIVITY_INTERVAL_SECONDS,
    WAIT_SECONDS,
    wait_with_active_session,
)

ISSUER = "https://auth.redtail.cdot.io/realms/redtail"
GATEWAY_SCOPES = ["mcp:servers:read", "mcp:tools:list", "mcp:tools:call"]
OBSERVED_CASES = {
    "gateway_cas_browser_login",
    "gateway_mcp_request_observed",
    "gateway_upstream_request_observed",
    "gateway_mcp_denial",
    "gateway_managed_upstream_oauth",
    "gateway_managed_upstream_refresh",
    "gateway_native_refresh_observed",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument(
        "--launcher", type=Path, help="Installed npm command for managed CLI wiring"
    )
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--endpoint", required=True)
    parser.add_argument("--upstream", required=True)
    parser.add_argument("--browser-via-ssh")
    parser.add_argument("--gateway-evidence", type=Path, required=True)
    parser.add_argument("--refresh-cycles", type=int, choices=[0, 2], default=2)
    parser.add_argument(
        "--cleanup-barrier",
        type=Path,
        help="New coordination directory: controller must release all SSO-sharing peers before issuer logout",
    )
    args = parser.parse_args()
    if args.cleanup_barrier and (
        not args.cleanup_barrier.is_absolute() or args.cleanup_barrier.exists()
    ):
        parser.error("Cleanup coordination requires a new absolute directory")
    identity = GatewayTestIdentity(args.endpoint)
    gateway, upstream = urlsplit(args.endpoint), urlsplit(args.upstream)
    if (
        any(
            u.scheme != "https" or not u.hostname or u.username or u.query or u.fragment
            for u in [gateway, upstream]
        )
        or gateway.netloc == upstream.netloc
    ):
        parser.error(
            "Distinct, credential-free HTTPS gateway and upstream URLs are required"
        )
    args.binary = args.binary.resolve(strict=True)
    args.state.mkdir(parents=True, mode=0o700, exist_ok=False)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, AIRS_HARNESS_HOME=str(args.state.resolve()))
    if os.uname().sysname == "Linux":
        if not env.get("DBUS_SESSION_BUS_ADDRESS"):
            parser.error("Run Linux acceptance under dbus-run-session")
        for key, name in [("XDG_DATA_HOME", "keyring"), ("XDG_RUNTIME_DIR", "runtime")]:
            env[key] = str(args.state / name)
            Path(env[key]).mkdir(mode=0o700)
        subprocess.run(
            ["gnome-keyring-daemon", "--unlock", "--components=secrets"],
            input=secrets.token_urlsafe(48).encode(),
            env=env,
            capture_output=True,
            check=True,
        )
    binary_sha = hashlib.sha256(args.binary.read_bytes()).hexdigest()
    rows = []

    def run(label, command, login=False):
        child = subprocess.Popen(
            [str(args.launcher or args.binary), *command],
            env=env,
            # Keep manual callback input alive while the browser/tunnel completes.
            stdin=subprocess.PIPE if login else None,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        output, pending, tunnels = [], queue.Queue(), []

        def collect():
            for line in child.stdout:
                pending.put(line)
            pending.put(None)

        threading.Thread(target=collect, daemon=True).start()
        try:
            deadline = time.monotonic() + (1800 if login else 300)
            while True:
                if time.monotonic() >= deadline:
                    raise TimeoutError(f"{label} exceeded its execution deadline")
                line = pending.get(timeout=max(1, deadline - time.monotonic()))
                if line is None:
                    break
                output.append(line)
                match = re.search(r"https://\S+\?\S+", line)
                if login and match:
                    url = match.group(0)
                    parsed = urlsplit(url)
                    if parsed.netloc not in [gateway.netloc, urlsplit(ISSUER).netloc]:
                        raise ValueError(
                            "Native client returned an unexpected authorization destination"
                        )
                    query = parse_qs(parsed.query)
                    if (
                        query.get("code_challenge_method") != ["S256"]
                        or len(query.get("state", [])) != 1
                    ):
                        raise ValueError(
                            "Native authorization must use state and S256 PKCE"
                        )
                    callback = urlsplit(query["redirect_uri"][0])
                    if (
                        callback.hostname != "127.0.0.1"
                        or callback.scheme != "http"
                        or not callback.port
                    ):
                        raise ValueError("Expected the native loopback callback")
                    if args.browser_via_ssh:
                        remote = "import sys,subprocess,time; subprocess.run(['open',sys.stdin.read().strip()],check=True); time.sleep(1800)"
                        tunnel = subprocess.Popen(
                            [
                                "ssh",
                                "-o",
                                "BatchMode=yes",
                                "-o",
                                "ExitOnForwardFailure=yes",
                                "-R",
                                f"127.0.0.1:{callback.port}:127.0.0.1:{callback.port}",
                                args.browser_via_ssh,
                                "/usr/bin/python3 -c " + shlex.quote(remote),
                            ],
                            stdin=subprocess.PIPE,
                            stdout=subprocess.DEVNULL,
                            stderr=subprocess.DEVNULL,
                        )
                        tunnels.append(tunnel)
                        tunnel.stdin.write(url.encode())
                        tunnel.stdin.close()
                    elif os.uname().sysname == "Darwin":
                        subprocess.run(["open", url], check=True, capture_output=True)
                    else:
                        raise ValueError(
                            "A desktop browser or --browser-via-ssh is required"
                        )
                    print(
                        json.dumps(
                            {
                                "case": label,
                                "waiting_for_browser": True,
                                "authorization_host": parsed.netloc,
                                "client_id": query.get("client_id", [None])[0],
                            }
                        ),
                        flush=True,
                    )
            code = child.wait(timeout=15)
            rows.append({"case": label, "passed": code == 0, "exit_code": code})
            if code:
                raise RuntimeError(f"{label} failed; inspect the private diagnostic")
            print(json.dumps(rows[-1]), flush=True)
            return "".join(output)
        finally:
            if child.poll() is None:
                child.terminate()
                child.wait(timeout=15)
            if child.stdin is not None:
                child.stdin.close()
            for tunnel in tunnels:
                tunnel.terminate()
                tunnel.wait(timeout=15)
            diagnostic = args.output.with_name(
                args.output.stem + "." + label + ".private.log"
            )
            with open(
                diagnostic, "x", opener=lambda path, flags: os.open(path, flags, 0o600)
            ) as stream:
                stream.write("".join(output))

    def model(label, prompt, expected):
        output = run(
            label,
            [
                "exec",
                "--skip-git-repo-check",
                "-c",
                "features.shell_tool=false",
                "-c",
                "features.multi_agent=false",
                "--json",
                prompt,
            ],
        )
        completed = set()
        for line in output.splitlines():
            try:
                item = json.loads(line).get("item", {})
            except (ValueError, AttributeError):
                continue
            if (
                isinstance(item, dict)
                and item.get("type") == "mcp_tool_call"
                and item.get("server") == identity.name
                and item.get("status") == "completed"
                and not item.get("error")
                and not (item.get("result") or {}).get("isError", False)
            ):
                completed.add(item["tool"])
        if not expected <= completed:
            raise ValueError(
                f"{label}: missing successful tools {sorted(expected - completed)}"
            )
        rows.append(
            {
                "case": label + "_tool_results",
                "passed": True,
                "tools": sorted(completed),
            }
        )

    def credential_metadata(label):
        # Homes share the OS keyring: read only this run's unique server account.
        result = None
        for account in identity.accounts():
            if os.uname().sysname == "Darwin":
                command = [
                    "security",
                    "find-generic-password",
                    "-s",
                    "Codex MCP Credentials",
                    "-a",
                    account,
                    "-w",
                ]
            else:
                command = [
                    "secret-tool",
                    "lookup",
                    "service",
                    "Codex MCP Credentials",
                    "username",
                    account,
                ]
            result = subprocess.run(command, env=env, capture_output=True)
            if result.returncode == 0:
                break
        if result is None or result.returncode:
            raise RuntimeError(
                "Native credential metadata lookup failed; no secret output emitted"
            )
        record = json.loads(result.stdout)
        identity.verify_record(record)
        token = record["token_response"]
        row = {
            "case": label,
            "passed": True,
            "observed_at": time.time(),
            "client_id": record["client_id"],
            "expires_at": record["expires_at"],
            "access_token_sha256": hashlib.sha256(
                token["access_token"].encode()
            ).hexdigest(),
            "refresh_token_sha256": hashlib.sha256(
                token["refresh_token"].encode()
            ).hexdigest(),
        }
        rows.append(row)
        print(json.dumps(row), flush=True)
        return row

    passed, logged_in, mcp_added = False, False, False
    started_at = time.time()
    print(
        json.dumps(
            {
                "case": "run_started",
                "started_at": started_at,
                "binary_sha256": binary_sha,
                "endpoint": args.endpoint,
                "server_name": identity.name,
                "upstream_endpoint": args.upstream,
            }
        ),
        flush=True,
    )
    try:
        run(
            "setup",
            [
                "env",
                "create",
                "work",
                "--gateway-url",
                "https://gateway.redtail.cdot.io/v1",
            ],
        )
        registry = json.loads((args.state / "environments.json").read_text())
        environment_home = (
            args.state / "environments" / registry["environments"]["work"]["id"]
        )
        config = environment_home / "config.toml"
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
            login=True,
        )
        logged_in = True
        run(
            "history_before_mcp",
            [
                "exec",
                "--skip-git-repo-check",
                "--json",
                "Reply with the single word ready.",
            ],
        )
        binding = environment_home / "session-binding.json"
        pinned = binding.read_bytes()
        run(
            "mcp_browser_pkce",
            [
                "mcp",
                "add",
                identity.name,
                "--url",
                args.endpoint,
                "--scopes",
                ",".join(GATEWAY_SCOPES),
                "--no-browser",
            ],
            login=True,
        )
        mcp_added = True
        settings = tomllib.loads(config.read_text())["mcp_servers"][identity.name]
        if settings["url"] != args.endpoint or any(
            k in settings
            for k in [
                "command",
                "http_headers",
                "http_headers_helper",
                "env_http_headers",
                "bearer_token_env_var",
                "oauth",
            ]
        ):
            raise ValueError("Expected native dynamic OAuth against the gateway only")
        if (environment_home / ".credentials.json").exists():
            raise ValueError("Native credential storage fell back to a file")
        previous_credential = credential_metadata("gateway_credential_initial")
        run("doctor_after_mcp", ["doctor", "--verify-access"])
        model(
            "all_read_tools",
            f"Use the {identity.name} MCP tools to inspect the authorized AIRS deployment. Call ALL eight tools: "
            + ", ".join(sorted(TOOLS))
            + ". Use IDs from list calls for detail calls. Report the names actually retrieved. "
            "Do not use shell commands, MCP resources or sub-agents.",
            TOOLS,
        )
        if binding.read_bytes() != pinned:
            raise ValueError("MCP onboarding changed the inference history binding")
        rows.append({"case": "inference_history_preserved", "passed": True})
        for cycle in range(1, args.refresh_cycles + 1):
            # Inspected gateway 2.22.0 issues one-hour gateway-facing tokens.
            # Wait real time: changing a local expiry field is not expiry acceptance.
            print(
                json.dumps(
                    {
                        "case": "await_gateway_token_expiry",
                        "cycle": cycle,
                        "seconds": WAIT_SECONDS,
                        "scenario": "active-user-session",
                        "activity_interval_seconds": ACTIVITY_INTERVAL_SECONDS,
                    }
                ),
                flush=True,
            )

            def activity(index):
                model(
                    f"active_session_{cycle}_{index}",
                    f"Call the {identity.name} list_workspaces tool and report the returned workspace name. Do not use shell commands or sub-agents.",
                    {"list_workspaces"},
                )
                credential = credential_metadata(
                    f"gateway_credential_active_{cycle}_{index}"
                )
                if any(
                    credential[key] != previous_credential[key]
                    for key in ["access_token_sha256", "expires_at"]
                ):
                    raise ValueError(
                        "Activity rotated the gateway token before the expiry check"
                    )

            wait_with_active_session(previous_credential["expires_at"], activity)
            try:
                with ThreadPoolExecutor(max_workers=2) as pool:
                    futures = [
                        pool.submit(
                            model,
                            f"refresh_{cycle}_{i}",
                            f"Call the {identity.name} list_workspaces tool and report the returned workspace name. Do not use shell commands or sub-agents.",
                            {"list_workspaces"},
                        )
                        for i in range(2)
                    ]
                    for future in futures:
                        future.result()
            finally:
                # Preserve renewal evidence even if inference or a backend tool fails.
                # Metadata alone never marks the workflow or expiry cycle passed.
                current_credential = credential_metadata(
                    f"gateway_credential_refresh_{cycle}"
                )
            if (
                time.time() * 1000 <= previous_credential["expires_at"]
                or current_credential["expires_at"] <= previous_credential["expires_at"]
                or current_credential["access_token_sha256"]
                == previous_credential["access_token_sha256"]
            ):
                raise ValueError(
                    "Native gateway token did not rotate after actual expiration"
                )
            previous_credential = current_credential
            if binding.read_bytes() != pinned:
                raise ValueError("Refresh changed the inference history binding")
        # Allow the separate read-only collector to correlate the final refresh calls.
        observation_deadline = time.monotonic() + 120
        while (
            not args.gateway_evidence.exists()
            and time.monotonic() < observation_deadline
        ):
            time.sleep(2)
        observations = json.loads(args.gateway_evidence.read_text())
        if (
            observations["endpoint"] != args.endpoint
            or observations["upstream_endpoint"] != args.upstream
            or observations["binary_sha256"] != binary_sha
            or observations["started_at"] < started_at
        ):
            raise ValueError(
                "Gateway observations must bind this executable, route and run window"
            )
        observed = observations["results"]
        if not OBSERVED_CASES <= {
            r["case"] for r in observed if r.get("passed") is True and r.get("evidence")
        }:
            raise ValueError("Correlated gateway/CAS/upstream evidence is incomplete")
        rows.extend(observed)
        passed = args.refresh_cycles == 2
    finally:
        cleanup_error = None
        if args.cleanup_barrier:
            try:
                wait_for_cleanup_release(args.cleanup_barrier, workflow_passed=passed)
            except BaseException as error:
                passed = False
                cleanup_error = error
        try:
            try:
                if mcp_added:
                    run("mcp_logout", ["mcp", "logout", identity.name])
                    listing = json.loads(
                        run("mcp_list_after_logout", ["mcp", "list", "--json"])
                    )
                    if (
                        next(s for s in listing if s["name"] == identity.name)[
                            "auth_status"
                        ]
                        == "o_auth"
                    ):
                        raise ValueError("MCP logout left native OAuth credentials")
                    run("inference_after_mcp_logout", ["doctor", "--verify-access"])
            finally:
                if logged_in:
                    run("inference_logout", ["logout"])
        except Exception:
            passed = False
            raise
        finally:
            receipt = {
                "passed": passed,
                "binary_sha256": binary_sha,
                "platform": os.uname().sysname,
                "endpoint": args.endpoint,
                "server_name": identity.name,
                "routing": {
                    "mode": "gateway-proxied-mcp",
                    "gateway_endpoint": args.endpoint,
                    "upstream_endpoint": args.upstream,
                },
                "validation_tooling_sha256": {
                    name: hashlib.sha256(
                        (Path(__file__).resolve().parent / name).read_bytes()
                    ).hexdigest()
                    for name in [
                        "validate_gateway_mcp.py",
                        "airs_gateway_test_identity.py",
                        "airs_gateway_cleanup.py",
                    ]
                },
                "identity": "interactive human SSO; isolated native state",
                "refresh_cycles": args.refresh_cycles,
                "refresh_scenario": "active-user-session",
                "activity_interval_seconds": ACTIVITY_INTERVAL_SECONDS,
                "started_at": started_at,
                "results": rows,
            }
            args.output.write_text(json.dumps(receipt, indent=2) + "\n")
        if cleanup_error:
            raise cleanup_error
    if not passed:
        raise SystemExit("Gateway smoke checks are not complete release acceptance")


if __name__ == "__main__":
    main()
