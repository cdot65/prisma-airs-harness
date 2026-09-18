#!/usr/bin/env python3
"""Exercise branded onboarding against local HTTPS OIDC and a real isolated OS store.

Linux: dbus-run-session -- uv run --with pyte==0.8.2 scripts/validate_airs_onboarding.py
  --binary /absolute/path/to/airs-harness --output /absolute/path/to/receipts
No production identity, gateway, tenant or user data is used.
"""

import argparse
import hashlib
import json
import os
import re
import secrets
import socket
import ssl
import subprocess
import tempfile
import time
from pathlib import Path
from urllib.request import urlopen

from airs_onboarding_fixture import IdentityFixture
from validate_airs_onboarding_preview import Preview, write_gallery


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    args.output.mkdir(parents=True, exist_ok=True)
    checks, captures, states = [], {}, []
    daemon = None
    with tempfile.TemporaryDirectory(prefix="airs-onboarding-fixture-") as temporary:
        root = Path(temporary)
        os.chmod(root, 0o700)
        fixture = IdentityFixture(root)
        environment = dict(
            os.environ,
            XDG_DATA_HOME=str(root / "data"),
            XDG_CONFIG_HOME=str(root / "config"),
            GNOME_KEYRING_CONTROL=str(root / "keyring"),
            SSL_CERT_FILE=str(fixture.certificate),
        )
        for name in ["AIRS_API_KEY", "AIRS_TERMINAL_HOME", "NO_COLOR"]:
            environment.pop(name, None)
        browser_record = root / "browser-url"
        browser = root / "browser"
        browser.write_text(
            '#!/usr/bin/env python3\nimport os,sys\np=os.environ["AIRS_FIXTURE_BROWSER_RECORD"]\nf=os.open(p,os.O_CREAT|os.O_WRONLY|os.O_TRUNC,0o600)\nos.write(f,sys.argv[1].encode())\nos.close(f)\n'
        )
        browser.chmod(0o700)
        environment.update(
            BROWSER=str(browser), AIRS_FIXTURE_BROWSER_RECORD=str(browser_record)
        )
        for key in ["XDG_DATA_HOME", "XDG_CONFIG_HOME", "GNOME_KEYRING_CONTROL"]:
            Path(environment[key]).mkdir(mode=0o700, parents=True, exist_ok=True)
        password = secrets.token_urlsafe(32)
        owner_command = [
            "dbus-send",
            "--session",
            "--print-reply",
            "--dest=org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus.NameHasOwner",
            "string:org.freedesktop.secrets",
        ]
        owner = subprocess.run(
            owner_command, env=environment, check=True, capture_output=True, text=True
        )
        assert "boolean false" in owner.stdout, (
            "Run this fixture in a fresh dbus-run-session"
        )
        subprocess.run(
            [
                "dbus-update-activation-environment",
                "XDG_DATA_HOME",
                "XDG_CONFIG_HOME",
                "GNOME_KEYRING_CONTROL",
            ],
            env=environment,
            check=True,
            capture_output=True,
        )
        keyring_log = root / "keyring.log"
        with keyring_log.open("wb") as log:
            daemon = subprocess.Popen(
                [
                    "gnome-keyring-daemon",
                    "--foreground",
                    "--unlock",
                    "--components=secrets",
                    "--control-directory",
                    str(root / "keyring"),
                ],
                env=environment,
                stdin=subprocess.PIPE,
                stdout=log,
                stderr=log,
            )
            daemon.stdin.write(password.encode())
            daemon.stdin.close()
        for _ in range(100):
            owner = subprocess.run(
                owner_command,
                env=environment,
                check=True,
                capture_output=True,
                text=True,
            )
            if "boolean true" in owner.stdout:
                break
            time.sleep(0.05)
        assert "boolean true" in owner.stdout, (
            "Private Secret Service did not acquire its bus name"
        )
        tls = ssl.create_default_context(cafile=str(fixture.certificate))

        def cli(env, *arguments):
            result = subprocess.run(
                [str(binary), *arguments],
                env=env,
                cwd=root,
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            assert result.returncode == 0, result.stderr
            return result.stdout

        def state(name):
            home = root / name
            home.mkdir()
            states.append(home)
            return dict(environment, AIRS_HARNESS_HOME=str(home))

        def configured(name, gateway=None):
            env = state(name)
            gateway = gateway or fixture.issuer + "/v1"
            cli(env, "env", "create", "work", "--gateway-url", gateway)
            registry = json.loads(
                (Path(env["AIRS_HARNESS_HOME"]) / "environments.json").read_text()
            )
            home = (
                Path(env["AIRS_HARNESS_HOME"])
                / "environments"
                / registry["environments"]["work"]["id"]
            )
            (home / "login-settings.json").write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "gateway_url": gateway,
                        "identity": {
                            "issuer": fixture.issuer,
                            "client_id": fixture.client,
                            "audience": fixture.audience,
                        },
                    }
                )
            )
            return env, home

        def session(env, arguments=None):
            return Preview(
                binary,
                arguments=arguments or ["login"],
                environment=env,
                directory=root,
            )

        def choose_company(terminal, method=b"1"):
            terminal.expect("Sign in to continue")
            terminal.send(b"1")
            terminal.expect("Continue with saved settings")
            terminal.send(b"1")
            terminal.expect("How would you like to sign in?")
            terminal.send(method)

        def authorize_browser(terminal):
            deadline = time.monotonic() + 10
            while not browser_record.exists() and time.monotonic() < deadline:
                terminal.pump(0.05)
            assert browser_record.exists(), "Browser launch did not receive a URL"
            url = browser_record.read_text()
            browser_record.unlink()
            assert url.startswith(fixture.issuer + "/authorize?")
            with urlopen(url, context=tls, timeout=15) as response:
                assert response.status == 200

        try:
            # Wait for the private Secret Service to own its bus name.
            for _ in range(50):
                probe = subprocess.run(
                    [
                        "secret-tool",
                        "store",
                        "--label=AIRS isolated fixture",
                        "service",
                        "airs-onboarding-fixture",
                        "account",
                        "probe",
                    ],
                    input="synthetic-probe",
                    text=True,
                    env=environment,
                    capture_output=True,
                    timeout=5,
                    check=False,
                )
                if probe.returncode == 0:
                    break
                time.sleep(0.1)
            assert probe.returncode == 0, (
                "Isolated Secret Service did not start: "
                + keyring_log.read_text()
                + probe.stderr
            )
            subprocess.run(
                [
                    "secret-tool",
                    "clear",
                    "service",
                    "airs-onboarding-fixture",
                    "account",
                    "probe",
                ],
                env=environment,
                check=True,
                capture_output=True,
            )
            checks.append(
                "isolated native Secret Service accepts and deletes credentials"
            )

            env = state("cancel-before-create")
            terminal = session(env, ["env", "create"])
            try:
                terminal.expect("Connect an environment")
                terminal.send(b"\x1b")
                terminal.finish("", status=1)
                assert not (
                    Path(env["AIRS_HARNESS_HOME"]) / "environments.json"
                ).exists()
                checks.append("cancel before creation leaves no registered environment")
            finally:
                terminal.close()

            env = state("fresh-session")
            terminal = Preview(
                binary,
                arguments=[],
                environment=env,
                directory=root,
                terminal_stdout=True,
            )
            try:
                terminal.expect("Connect an environment")
                terminal.send(b"1")
                terminal.expect("Environment name")
                terminal.send(b"\x15\x1b[200~bad name\x1b[201~\r")
                terminal.expect("letters, digits")
                assert not (
                    Path(env["AIRS_HARNESS_HOME"]) / "environments.json"
                ).exists()
                terminal.send(b"\x15\x1b[200~work\x1b[201~\r")
                terminal.expect("AI Gateway URL")
                terminal.send(
                    b"\x1b[200~https://gateway.example/v1/responses\x1b[201~\r"
                )
                terminal.expect("API root")
                terminal.send(
                    b"\x15\x1b[200~"
                    + (fixture.issuer + "/v1").encode()
                    + b"\x1b[201~\r"
                )
                terminal.expect("Create this environment?")
                captures["Confirm first environment"] = terminal.capture()
                assert not (
                    Path(env["AIRS_HARNESS_HOME"]) / "environments.json"
                ).exists()
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
                terminal.send(b"1")
                authorize_browser(terminal)
                terminal.expect("You're ready to use AIRS")
                terminal.send(b"\r")
                terminal.expect("Yes, continue")
                terminal.pump(
                    0.4
                )  # Respect the existing trust screen's typeahead guard.
                terminal.send(b"\r")
                terminal.expect("permissions:")
                terminal.pump(0.4)
                captures["First session after company SSO"] = terminal.capture()
                terminal.send(b"\x04")
                terminal.finish("")
                checks.append(
                    "fresh airs: validation, explicit creation, public SSO settings, saved identity and entry into the agent"
                )
            finally:
                terminal.close()

            env, home = configured("browser-success")
            terminal = session(env)
            try:
                choose_company(terminal)
                authorize_browser(terminal)
                terminal.expect("You're ready to use AIRS")
                captures["Company SSO complete"] = terminal.capture()
                terminal.send(b"\r")
                terminal.finish("")
                binding = json.loads((home / "credential-binding.json").read_text())
                assert binding["source"]["kind"] == "oidc"
                assert "fixture-user" not in (home / "config.toml").read_text()
                checks.append(
                    "browser OAuth callback, PKCE, verified JWT, OS persistence and gateway probe"
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
                terminal.expect("Yes, continue")
                assert b"Sign in to continue" not in terminal.transcript
                terminal.pump(0.4)
                terminal.send(b"\r")
                terminal.expect("permissions:")
                terminal.pump(0.4)
                terminal.send(b"\x04")
                terminal.finish("")
                assert len(fixture.requests) == before
                checks.append(
                    "returning signed-in startup bypasses onboarding and additional authentication"
                )
            finally:
                terminal.close()

            fixture.gateway_status = 403
            env, home = configured("device-denied-gateway")
            terminal = session(env)
            try:
                choose_company(terminal, b"2")
                terminal.expect("Authorize this device")
                assert fixture.devices
                code = list(fixture.devices.values())[-1]["user_code"]
                with urlopen(
                    fixture.issuer + "/verify?user_code=" + code,
                    context=tls,
                    timeout=10,
                ) as response:
                    assert response.status == 200
                terminal.expect("gateway access needs attention")
                captures["Credential saved but gateway denied"] = terminal.capture()
                terminal.send(b"\r")
                terminal.expect("Your credential is saved")
                terminal.send(b"1")
                terminal.finish("")
                assert (home / "credential-binding.json").exists()
                checks.append(
                    "device OAuth persists credentials while gateway denial stays distinct"
                )
            finally:
                terminal.close()
            fixture.gateway_status = 200

            env, home = configured("browser-launch-unavailable")
            env.update(
                PATH=str(root / "no-browser-tools"),
                BROWSER=str(root / "missing-browser"),
            )
            terminal = session(env)
            try:
                choose_company(terminal)
                terminal.expect("Open company sign-in")
                terminal.resize(100, 40)
                terminal.expect("browser could not open automatically")
                rendered = "".join(line.strip() for line in terminal.screen.display)
                match = re.search(
                    r"https://127\.0\.0\.1:[0-9]+/authorize\?[A-Za-z0-9%=&+./_~:-]+",
                    rendered,
                )
                assert match is not None, (
                    "Browser failure did not retain its usable authorization link"
                )
                with urlopen(match.group(), context=tls, timeout=15) as response:
                    assert response.status == 200
                terminal.expect("You're ready to use AIRS")
                terminal.send(b"\r")
                terminal.finish("")
                assert (home / "credential-binding.json").exists()
                checks.append(
                    "browser launch failure retains a displayed URL that completes real callback authentication"
                )
            finally:
                terminal.close()

            env, home = configured("cancel-browser")
            terminal = session(env)
            try:
                choose_company(terminal)
                terminal.expect("Waiting for company sign-in")
                terminal.send(b"\x1b")
                terminal.finish("", status=1)
                assert not (home / "credential-binding.json").exists()
                assert (home / "login-settings.json").exists()
                browser_record.unlink(missing_ok=True)
                checks.append(
                    "browser cancellation preserves public settings without publishing credentials"
                )
            finally:
                terminal.close()

            env, home = configured("storage-unavailable")
            env["DBUS_SESSION_BUS_ADDRESS"] = "unix:path=" + str(root / "missing-bus")
            before = len(fixture.requests)
            terminal = session(env)
            try:
                choose_company(terminal)
                terminal.expect("Sign-in needs your attention")
                captures["Native storage unavailable"] = terminal.capture()
                terminal.send(b"\x1b")
                terminal.finish("", status=1)
                assert len(fixture.requests) == before
                assert not (home / "credential-binding.json").exists()
                checks.append(
                    "storage failure precedes authorization and never claims sign-in success"
                )
            finally:
                terminal.close()

            fixture.invalid_nonce = True
            env, home = configured("retry-invalid-token")
            terminal = session(env)
            try:
                choose_company(terminal)
                authorize_browser(terminal)
                terminal.expect("Sign-in needs your attention")
                terminal.expect("nonce")
                assert not (home / "credential-binding.json").exists()
                fixture.invalid_nonce = False
                terminal.send(b"\r")
                terminal.expect("Choose your next step")
                terminal.send(b"1")
                terminal.expect("Continue with saved settings")
                terminal.send(b"1")
                terminal.expect("How would you like to sign in?")
                terminal.send(b"1")
                authorize_browser(terminal)
                terminal.expect("You're ready to use AIRS")
                terminal.send(b"\r")
                terminal.finish("")
                checks.append(
                    "invalid signed ID-token nonce cannot publish credentials; inline retry completes a fresh valid flow"
                )
            finally:
                fixture.invalid_nonce = False
                terminal.close()

            fixture.client = "c" * 2000
            env, home = configured("manual-long-url")
            terminal = session(env)
            try:
                choose_company(terminal, b"3")
                pattern = rb"https://127\.0\.0\.1:[0-9]+/authorize\?[^\r\n\x1b]+[\r\n]"
                deadline = time.monotonic() + 10
                match = None
                while match is None and time.monotonic() < deadline:
                    terminal.pump(0.05)
                    match = re.search(pattern, bytes(terminal.transcript))
                assert match is not None, (
                    "Manual flow did not print the complete authorization URL"
                )
                url = match.group().strip().decode()
                assert len(url) > 2048 and not browser_record.exists()
                with urlopen(url, context=tls, timeout=15) as response:
                    assert response.status == 200
                terminal.expect("You're ready to use AIRS")
                terminal.send(b"\r")
                terminal.finish("Credentials stored in the OS store")
                checks.append(
                    "manual browser flow prints a complete long URL and persists the resulting token bundle"
                )
            finally:
                fixture.client = "airs-fixture-client"
                terminal.close()

            with socket.socket() as unused:
                unused.bind(("127.0.0.1", 0))
                offline_gateway = f"https://127.0.0.1:{unused.getsockname()[1]}/v1"
            env, home = configured("offline-access", gateway=offline_gateway)
            terminal = session(env)
            try:
                choose_company(terminal)
                authorize_browser(terminal)
                terminal.expect("gateway access needs attention")
                terminal.expect("connection failed")
                count = len(fixture.exchanges)
                terminal.send(b"\r")
                terminal.expect("Your credential is saved")
                terminal.send(b"2")
                terminal.expect("gateway access needs attention")
                terminal.send(b"\r")
                terminal.expect("Your credential is saved")
                terminal.send(b"1")
                terminal.finish("")
                assert (
                    len(fixture.exchanges) == count
                    and (home / "credential-binding.json").exists()
                )
                checks.append(
                    "offline gateway retry preserves credentials and does not repeat SSO"
                )
            finally:
                terminal.close()

            env, home = configured("choose-environment")
            cli(
                env, "env", "create", "staging", "--gateway-url", fixture.issuer + "/v1"
            )
            cli(env, "env", "use", "work")
            registry_path = Path(env["AIRS_HARNESS_HOME"]) / "environments.json"
            original_registry = registry_path.read_bytes()
            registry = json.loads(original_registry)
            staging = (
                registry_path.parent
                / "environments"
                / registry["environments"]["staging"]["id"]
            )
            (staging / "login-settings.json").write_bytes(
                (home / "login-settings.json").read_bytes()
            )
            (home / "history.jsonl").write_text("preserved-work-history\n")
            terminal = session(env)
            try:
                terminal.expect("Choose another environment")
                terminal.send(b"3")
                terminal.expect("Choose an environment")
                terminal.send(b"1")
                choose_company(terminal)
                authorize_browser(terminal)
                terminal.expect("You're ready to use AIRS")
                captures["Selected environment sign-in"] = terminal.capture()
                terminal.send(b"\r")
                terminal.finish("")
                assert (staging / "credential-binding.json").exists()
                assert not (home / "credential-binding.json").exists()
                assert (
                    home / "history.jsonl"
                ).read_text() == "preserved-work-history\n"
                assert registry_path.read_bytes() == original_registry
                checks.append(
                    "environment picker authenticates the chosen home without changing the default or another history"
                )
            finally:
                terminal.close()

            env, home = configured("saved-workspace-key")
            terminal = session(env)
            try:
                terminal.expect("Sign in to continue")
                terminal.send(b"2")
                terminal.expect("Workspace API key (input hidden")
                secret = b"synthetic-persisted-workspace-key"
                fixture.access_tokens.add(secret.decode())
                terminal.send(b"\x1b[200~" + secret + b"\x1b[201~\r")
                terminal.expect("You're ready to use AIRS")
                assert secret not in terminal.transcript
                terminal.send(b"\r")
                terminal.finish("No individual user identity is asserted")
                binding = json.loads((home / "credential-binding.json").read_text())
                assert binding["source"]["kind"] == "keyring-v2"
                checks.append(
                    "workspace key remains hidden, persists in the native store and authorizes the gateway probe"
                )
            finally:
                terminal.close()

            env, home = configured("hidden-workspace-key")
            terminal = session(env)
            try:
                terminal.expect("Sign in to continue")
                terminal.send(b"2")
                terminal.expect("Workspace API key (input hidden")
                secret = b"synthetic-key-must-never-be-visible"
                terminal.send(b"\x1b[200~" + secret + b"\x1b[201~")
                assert secret not in terminal.transcript
                terminal.send(b"\x1b")
                terminal.expect("Sign-in needs your attention")
                assert secret not in terminal.transcript
                terminal.send(b"\x1b")
                terminal.finish("", status=1)
                assert not (home / "credential-binding.json").exists()
                checks.append(
                    "workspace key stays hidden through paste, cancellation and recovery"
                )
            finally:
                terminal.close()

            for state_root in states:
                for path in state_root.rglob("*"):
                    if path.is_file() and path.stat().st_size < 2_000_000:
                        contents = path.read_bytes()
                        assert all(
                            token.encode() not in contents
                            for token in fixture.access_tokens
                        ), "A bearer token was written outside the OS store"
            checks.append("no issued bearer token appears in environment files")

            receipt = {
                "passed": True,
                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                "checks": checks,
                "local_https_oidc": True,
                "native_os_store": True,
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
            write_gallery(
                args.output, captures, [next(iter(captures.values()))], preview=False
            )
            print(json.dumps(receipt, indent=2))
        finally:
            fixture.close()
            if daemon is not None and daemon.poll() is None:
                daemon.terminate()
                daemon.wait(timeout=10)


if __name__ == "__main__":
    main()
