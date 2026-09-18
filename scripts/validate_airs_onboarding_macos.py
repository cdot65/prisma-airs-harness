#!/usr/bin/env python3
"""Exercise an installed, signed AIRS candidate in the macOS desktop session.

Uses local HTTPS identity/gateway fixtures and uniquely bound native Keychain
entries. Device and manual-browser authorization avoid changing browser state.
Never changes Keychain search lists, locks the user's store or uses production SSO.
"""

import argparse
import hashlib
import json
import os
import re
import ssl
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from urllib.request import urlopen

from airs_onboarding_fixture import IdentityFixture
from airs_onboarding_tls_checks import check_tls_rejection
from validate_airs_onboarding_preview import Preview, write_gallery


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--native-binary", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    assert sys.platform == "darwin", "Run this fixture on native macOS"
    binary = args.binary.resolve(strict=True)
    native = (args.native_binary or binary).resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=True)
    checks, captures, states = [], {}, []
    with tempfile.TemporaryDirectory(prefix="airs-onboarding-macos-") as directory:
        root = Path(directory)
        fixture = IdentityFixture(root)
        tls = ssl.create_default_context(cafile=str(fixture.certificate))
        environment = dict(os.environ, SSL_CERT_FILE=str(fixture.certificate))
        for key in list(environment):
            if key.startswith(("AIRS_", "OPENAI_")) or key in (
                "CODEX_HOME",
                "CODEX_SQLITE_HOME",
                "CODEX_CA_CERTIFICATE",
                "NO_COLOR",
            ):
                environment.pop(key)

        def cli(env, *arguments, check=True):
            return subprocess.run(
                [str(binary), *arguments],
                cwd=root,
                env=env,
                capture_output=True,
                text=True,
                timeout=45,
                check=check,
            )

        def state(name, configure=True):
            home = root / name
            home.mkdir()
            env = dict(environment, AIRS_HARNESS_HOME=str(home))
            states.append(env)
            if configure:
                cli(
                    env,
                    "env",
                    "create",
                    "work",
                    "--gateway-url",
                    fixture.issuer + "/v1",
                )
                settings = {
                    "schema_version": 1,
                    "gateway_url": fixture.issuer + "/v1",
                    "identity": {
                        "issuer": fixture.issuer,
                        "client_id": fixture.client,
                        "audience": fixture.audience,
                    },
                }
                (selected_home(env) / "login-settings.json").write_text(
                    json.dumps(settings)
                )
            return env

        def selected_home(env, name="work"):
            home = Path(env["AIRS_HARNESS_HOME"])
            registry = json.loads((home / "environments.json").read_text())
            return home / "environments" / registry["environments"][name]["id"]

        def company(terminal, method=b"2"):
            terminal.expect("Sign in to continue")
            terminal.send(b"1")
            terminal.expect("Continue with saved settings")
            terminal.send(b"1")
            terminal.expect("How would you like to sign in?")
            terminal.send(method)

        def authorize_device(terminal):
            terminal.expect("Authorize this device")
            code = list(fixture.devices.values())[-1]["user_code"]
            with urlopen(
                fixture.issuer + "/verify?user_code=" + code, context=tls, timeout=10
            ) as response:
                assert response.status == 200

        def manual_url(terminal):
            deadline = time.monotonic() + 10
            pattern = rb"https://127\.0\.0\.1:[0-9]+/authorize\?[^\r\n\x1b]+[\r\n]"
            while time.monotonic() < deadline:
                terminal.pump(0.05)
                match = re.search(pattern, bytes(terminal.transcript))
                if match:
                    return match.group().strip().decode()
            raise AssertionError("Manual browser flow did not display its complete URL")

        try:
            env = state("tls-rejection")
            checks.extend(
                check_tls_rejection(binary, env, selected_home(env), root, fixture)
            )
            env = state("cancel-creation", configure=False)
            terminal = Preview(binary, arguments=[], environment=env, directory=root)
            try:
                terminal.expect("Connect an environment")
                terminal.send(b"\x1b")
                terminal.finish("", status=1)
                assert not (
                    Path(env["AIRS_HARNESS_HOME"]) / "environments.json"
                ).exists()
                checks.append(
                    "first-run cancellation restores the terminal and creates no environment"
                )
            finally:
                terminal.close()

            env = state("first-session", configure=False)
            terminal = Preview(
                binary,
                arguments=[],
                environment=env,
                directory=root,
                terminal_stdout=True,
            )
            try:
                terminal.expect("Connect an environment")
                captures["Apple Silicon welcome"] = terminal.capture()
                terminal.send(b"1")
                terminal.expect("Environment name")
                terminal.send(b"\r")
                terminal.expect("AI Gateway URL")
                terminal.send(
                    b"\x1b[200~" + (fixture.issuer + "/v1").encode() + b"\x1b[201~\r"
                )
                terminal.expect("Create this environment?")
                terminal.send(b"1")
                terminal.expect("Sign in to continue")
                terminal.send(b"1")
                for label, value in [
                    ("Company issuer URL", fixture.issuer),
                    ("Public client ID", fixture.client),
                    ("Gateway audience", fixture.audience),
                ]:
                    terminal.expect(label)
                    terminal.send(b"\x1b[200~" + value.encode() + b"\x1b[201~\r")
                terminal.expect("How would you like to sign in?")
                terminal.send(b"2")
                authorize_device(terminal)
                terminal.expect("You're ready to use AIRS")
                captures["Apple Silicon Keychain sign-in"] = terminal.capture()
                terminal.send(b"\r")
                terminal.expect("Yes, continue")
                terminal.pump(0.4)
                terminal.send(b"\r")
                terminal.expect("permissions:")
                terminal.pump(0.4)
                captures["Apple Silicon first session"] = terminal.capture()
                terminal.send(b"\x04")
                terminal.finish("")
                assert (selected_home(env) / "credential-binding.json").exists()
                checks.append(
                    "fresh airs creates the environment, saves device SSO in Keychain, verifies the gateway and enters the agent"
                )
            finally:
                terminal.close()

            before = len(fixture.requests)
            terminal = Preview(
                binary,
                arguments=[],
                environment=env,
                directory=root,
                terminal_stdout=True,
            )
            try:
                # The first session already trusted this workspace on macOS.
                terminal.expect("permissions:")
                assert b"Sign in to continue" not in terminal.transcript
                terminal.pump(0.4)
                terminal.send(b"\x04")
                terminal.finish("")
                assert len(fixture.requests) == before
                checks.append(
                    "returning signed-in startup bypasses onboarding and additional SSO"
                )
            finally:
                terminal.close()

            env = state("manual-browser")
            terminal = Preview(
                binary, arguments=["login"], environment=env, directory=root
            )
            try:
                company(terminal, b"3")
                with urlopen(manual_url(terminal), context=tls, timeout=15) as response:
                    assert response.status == 200
                terminal.expect("You're ready to use AIRS")
                terminal.send(b"\r")
                terminal.finish("Credentials stored in the OS store")
                checks.append(
                    "manual browser callback validates PKCE and JWT and persists the identity in Keychain"
                )
            finally:
                terminal.close()

            fixture.gateway_status = 403
            env = state("denied-gateway")
            terminal = Preview(
                binary, arguments=["login"], environment=env, directory=root
            )
            try:
                company(terminal)
                authorize_device(terminal)
                terminal.expect("gateway access needs attention")
                captures["Apple Silicon gateway denial"] = terminal.capture()
                terminal.send(b"\r")
                terminal.expect("Your credential is saved")
                terminal.send(b"3")
                terminal.finish("", status=1)
                assert b"Credential saved" in terminal.transcript
                assert (selected_home(env) / "credential-binding.json").exists()
                checks.append(
                    "gateway denial and exit preserve the saved identity with accurate recovery guidance"
                )
            finally:
                fixture.gateway_status = 200
                terminal.close()

            env = state("cancel-device")
            terminal = Preview(
                binary, arguments=["login"], environment=env, directory=root
            )
            try:
                company(terminal)
                terminal.expect("Authorize this device")
                terminal.send(b"\x1b")
                terminal.finish("", status=1)
                assert not (selected_home(env) / "credential-binding.json").exists()
                checks.append(
                    "pending device authorization cancels without publishing credentials"
                )
            finally:
                terminal.close()

            env = state("choose-environment")
            cli(
                env, "env", "create", "staging", "--gateway-url", fixture.issuer + "/v1"
            )
            cli(env, "env", "use", "work")
            home, staging = selected_home(env), selected_home(env, "staging")
            (staging / "login-settings.json").write_bytes(
                (home / "login-settings.json").read_bytes()
            )
            (home / "history.jsonl").write_text("preserved-owner-history\n")
            registry_path = Path(env["AIRS_HARNESS_HOME"]) / "environments.json"
            original_registry = registry_path.read_bytes()
            terminal = Preview(
                binary, arguments=["login"], environment=env, directory=root
            )
            try:
                terminal.expect("Choose another environment")
                terminal.send(b"3")
                terminal.expect("Choose an environment")
                terminal.send(b"1")
                company(terminal)
                authorize_device(terminal)
                terminal.expect("You're ready to use AIRS")
                terminal.send(b"\r")
                terminal.finish("")
                assert (staging / "credential-binding.json").exists()
                assert not (home / "credential-binding.json").exists()
                assert (
                    home / "history.jsonl"
                ).read_text() == "preserved-owner-history\n"
                assert registry_path.read_bytes() == original_registry
                checks.append(
                    "environment picker uses the selected identity home and preserves another history and the saved default"
                )
            finally:
                terminal.close()

            env = state("workspace-key")
            terminal = Preview(
                binary, arguments=["login"], environment=env, directory=root
            )
            try:
                terminal.expect("Sign in to continue")
                terminal.send(b"2")
                terminal.expect("Workspace API key (input hidden")
                key = "synthetic-macos-workspace-key"
                fixture.access_tokens.add(key)
                terminal.send(b"\x1b[200~" + key.encode() + b"\x1b[201~\r")
                terminal.expect("You're ready to use AIRS")
                assert key.encode() not in terminal.transcript
                terminal.send(b"\r")
                terminal.finish("No individual user identity is asserted")
                assert (
                    json.loads(
                        (selected_home(env) / "credential-binding.json").read_text()
                    )["source"]["kind"]
                    == "keyring-v2"
                )
                checks.append(
                    "hidden workspace-key paste persists in Keychain and authorizes a gateway request"
                )
            finally:
                terminal.close()

            for env in states:
                for path in Path(env["AIRS_HARNESS_HOME"]).rglob("*"):
                    if path.is_file() and path.stat().st_size < 2_000_000:
                        assert all(
                            token.encode() not in path.read_bytes()
                            for token in fixture.access_tokens
                        )
            checks.append("no bearer credential appears in application files")
        finally:
            failures = []
            accounts = []
            for env in states:
                registry = Path(env["AIRS_HARNESS_HOME"]) / "environments.json"
                if not registry.exists():
                    continue
                for name in json.loads(registry.read_text())["environments"]:
                    if (selected_home(env, name) / "credential-binding.json").exists():
                        binding = json.loads(
                            (
                                selected_home(env, name) / "credential-binding.json"
                            ).read_text()
                        )
                        accounts.append({"id": binding["id"], "environment": name})
                        result = cli(env, "--environment", name, "logout", check=False)
                        if result.returncode:
                            failures.append(
                                {"environment": name, "error": result.stderr}
                            )
                            # Retain only public recovery metadata if native
                            # deletion fails; never copy stored credentials.
                            recovery = (
                                args.output
                                / "cleanup-recovery"
                                / Path(env["AIRS_HARNESS_HOME"]).name
                                / name
                            )
                            recovery.mkdir(parents=True, exist_ok=True)
                            for filename in (
                                "credential-binding.json",
                                "credential-pending-logout.json",
                                "credential-pending-cleanup.json",
                                "logged-out",
                            ):
                                source = selected_home(env, name) / filename
                                if source.is_file():
                                    (recovery / filename).write_bytes(
                                        source.read_bytes()
                                    )
            fixture.close()
            (args.output / "CREDENTIAL-CLEANUP.json").write_text(
                json.dumps(
                    {
                        "passed": not failures,
                        "failures": failures,
                        "owned_accounts": accounts,
                    },
                    indent=2,
                )
                + "\n"
            )
            assert not failures, (
                "Owned fixture credential cleanup failed; inspect CREDENTIAL-CLEANUP.json"
            )
        checks.append(
            "all owned fixture identities are logged out and temporary state is removed"
        )
        receipt = {
            "passed": True,
            "platform": "darwin",
            "architecture": os.uname().machine,
            "binary_sha256": hashlib.sha256(native.read_bytes()).hexdigest(),
            "entrypoint": str(args.binary.absolute()),
            "checks": checks,
            "local_https_oidc": True,
            "native_os_store": "Keychain",
            "default_browser_automated": False,
            "browser_state_changed": False,
            "production_sso": False,
            "production_servicenow": False,
            "oauth_exchanges": fixture.exchanges,
            "gateway_requests": sum(
                path == "/v1/responses" for _, path in fixture.requests
            ),
        }
    (args.output / "ONBOARDING-ACCEPTANCE.json").write_text(
        json.dumps(receipt, indent=2) + "\n"
    )
    write_gallery(args.output, captures, [next(iter(captures.values()))], preview=False)
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
