"""Installed onboarding recovery uses the selected environment, never another default.

The Linux subprocess owns a fresh D-Bus and disposable native store. Only local
HTTPS fixtures and synthetic credentials are used; owner state is never read.
"""

import json
import os
from pathlib import Path
import secrets
import signal
import shlex
import subprocess
import sys
import tempfile
import time
import shutil
import unittest

REPO = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO / "scripts"))
from airs_onboarding_fixture import IdentityFixture
from validate_airs_onboarding_preview import Preview

BINARY = Path(
    os.environ.get("AIRS_HARNESS_BIN", REPO / "codex-rs/target/debug/airs-harness")
).resolve()


def require(condition, message="Fixture invariant failed"):
    if not condition:
        raise AssertionError(message)


def worker(name):

    def stop(_signal, _frame):
        raise RuntimeError("Isolated fixture cancelled")

    signal.signal(signal.SIGTERM, stop)
    with tempfile.TemporaryDirectory(prefix="airs-onboarding-baseline-") as temporary:
        root = Path(temporary)
        fixture = IdentityFixture(root)
        fixture.gateway_status = 403
        env = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith(("AIRS_", "OPENAI_"))
            and key
            not in (
                "CODEX_HOME",
                "CODEX_SQLITE_HOME",
                "CODEX_CA_CERTIFICATE",
                "NO_COLOR",
            )
        }
        env.update(
            AIRS_HARNESS_HOME=str(root / "harness"),
            XDG_DATA_HOME=str(root / "data"),
            XDG_CONFIG_HOME=str(root / "config"),
            GNOME_KEYRING_CONTROL=str(root / "keyring"),
            XDG_RUNTIME_DIR=str(root / "runtime"),
            SSL_CERT_FILE=str(fixture.certificate),
        )
        for key in (
            "AIRS_HARNESS_HOME",
            "XDG_DATA_HOME",
            "XDG_CONFIG_HOME",
            "GNOME_KEYRING_CONTROL",
            "XDG_RUNTIME_DIR",
        ):
            Path(env[key]).mkdir(mode=0o700)
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
            owner_command,
            env=env,
            capture_output=True,
            text=True,
            check=True,
            timeout=5,
        )
        require(
            "boolean false" in owner.stdout, "Requires an empty private D-Bus session"
        )
        subprocess.run(
            [
                "dbus-update-activation-environment",
                "XDG_DATA_HOME",
                "XDG_CONFIG_HOME",
                "GNOME_KEYRING_CONTROL",
                "XDG_RUNTIME_DIR",
            ],
            env=env,
            check=True,
            capture_output=True,
            timeout=5,
        )
        with (root / "keyring.log").open("wb") as log:
            daemon = subprocess.Popen(
                [
                    "gnome-keyring-daemon",
                    "--foreground",
                    "--unlock",
                    "--components=secrets",
                    "--control-directory",
                    env["GNOME_KEYRING_CONTROL"],
                ],
                env=env,
                stdin=subprocess.PIPE,
                stdout=log,
                stderr=log,
            )
            daemon.stdin.write(secrets.token_urlsafe(32).encode())
            daemon.stdin.close()
        terminal = None
        try:
            for _ in range(100):
                owner = subprocess.run(
                    owner_command,
                    env=env,
                    capture_output=True,
                    text=True,
                    check=True,
                    timeout=5,
                )
                if "boolean true" in owner.stdout:
                    break
                time.sleep(0.05)
            require("boolean true" in owner.stdout)

            def cli(*arguments):
                return subprocess.run(
                    [str(BINARY), *arguments],
                    env=env,
                    cwd=root,
                    capture_output=True,
                    text=True,
                    check=True,
                    timeout=30,
                ).stdout

            for environment in ("work", name):
                cli(
                    f"--environment={environment}",
                    "env",
                    "create",
                    "--gateway-url",
                    fixture.issuer + "/v1",
                )
            cli("env", "use", "work")
            registry_path = Path(env["AIRS_HARNESS_HOME"]) / "environments.json"
            before = registry_path.read_bytes()
            registry = json.loads(before)
            homes = {
                name: registry_path.parent / "environments" / value["id"]
                for name, value in registry["environments"].items()
            }
            (homes["work"] / "history.jsonl").write_text("preserved fixture history\n")
            command = (
                f"airs --environment={name}"
                if name.startswith("-")
                else f"airs --environment {name}"
            )
            terminal = Preview(
                BINARY,
                arguments=[f"--environment={name}", "login"],
                environment=env,
                directory=root,
                columns=100,
                rows=40,
            )
            terminal.expect("Sign in to continue")
            terminal.send(b"2")
            terminal.expect("Workspace API key (input hidden")
            key = b"synthetic-onboarding-baseline-key"
            fixture.access_tokens.add(key.decode())
            terminal.send(b"\x1b[200~" + key + b"\x1b[201~\r")
            terminal.expect("gateway access needs attention", timeout=20)
            screen = "\n".join(terminal.screen.display)
            compact = " ".join(screen.split())
            require(f"Environment {name}" in compact)
            require("HTTP 403" in compact)
            require(f"Retry: {command} doctor --verify-access" in compact, compact)
            require(key not in terminal.transcript)
            terminal.send(b"\x1b")
            terminal.finish("", status=1)
            terminal_text = "\n".join(terminal.screen.display)
            require(
                f"{command} doctor --verify-access"
                in terminal.transcript.decode(errors="replace"),
                terminal_text,
            )
            require(
                f"Start {command}" in terminal.transcript.decode(errors="replace"),
                terminal_text,
            )
            require(registry_path.read_bytes() == before)
            require((homes[name] / "credential-binding.json").exists())
            require(not (homes["work"] / "credential-binding.json").exists())
            require(
                (homes["work"] / "history.jsonl").read_text()
                == "preserved fixture history\n"
            )
            probe = subprocess.run(
                [
                    str(BINARY),
                    *shlex.split(command)[1:],
                    "doctor",
                    "--verify-access",
                    "--json",
                ],
                env=env,
                cwd=root,
                capture_output=True,
                text=True,
                timeout=40,
            )
            report = json.loads(probe.stdout)
            require(Path(report["state_directory"]) == homes[name])
            access = next(
                (
                    check
                    for check in report["checks"]
                    if check["name"] == "gateway_access"
                )
            )
            require("HTTP 403" in access["detail"], access["detail"])
            require(registry_path.read_bytes() == before)
            terminal = Preview(
                BINARY,
                arguments=[f"--environment={name}", "login"],
                environment=env,
                directory=root,
                columns=100,
                rows=40,
            )
            terminal.expect("Sign in to continue")
            terminal.send(b"\x1b")
            terminal.finish("", status=1)
            require(
                f"resume with {command} login."
                in terminal.transcript.decode(errors="replace")
            )
            require(registry_path.read_bytes() == before)
            pending = homes[name] / "credential-pending-cleanup.json"
            pending.write_text('{"fixture":"read-only pending metadata"}')
            pending_before = pending.read_bytes()
            binding_before = (homes[name] / "credential-binding.json").read_bytes()
            inspection = subprocess.run(
                [str(BINARY), *shlex.split(command)[1:], "doctor", "--json"],
                env=env,
                cwd=root,
                capture_output=True,
                text=True,
                timeout=30,
            )
            cleanup = next(
                check
                for check in json.loads(inspection.stdout)["checks"]
                if check["name"] == "credential_cleanup"
            )
            require(f"{command} login" in cleanup["detail"], cleanup["detail"])
            require(f"{command} logout" in cleanup["detail"], cleanup["detail"])
            require(pending.read_bytes() == pending_before)
            require(
                (homes[name] / "credential-binding.json").read_bytes() == binding_before
            )
            require(registry_path.read_bytes() == before)
            print("Selected-environment recovery and preserved default verified.")
        finally:
            if terminal:
                terminal.close()
            fixture.close()
            daemon.terminate()
            daemon.wait(timeout=10)


@unittest.skipUnless(
    sys.platform.startswith("linux"), "Isolated Linux Secret Service fixture"
)
class SelectedEnvironmentRecovery(unittest.TestCase):
    def test_denied_login_and_cancel_preserve_nondefault_environment(self):
        for name in ("staging", "-staging"):
            with self.subTest(environment=name):
                for command in (
                    "dbus-run-session",
                    "gnome-keyring-daemon",
                    "dbus-send",
                    "dbus-update-activation-environment",
                ):
                    self.assertIsNotNone(
                        shutil.which(command), f"Required fixture tool: {command}"
                    )
                process = subprocess.Popen(
                    [
                        "dbus-run-session",
                        "--",
                        sys.executable,
                        *(["-O"] if sys.flags.optimize else []),
                        str(Path(__file__).resolve()),
                        "--worker",
                        name,
                    ],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    text=True,
                    start_new_session=True,
                )
                try:
                    output, error = process.communicate(timeout=90)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGTERM)
                    try:
                        process.communicate(timeout=10)
                    except subprocess.TimeoutExpired:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.communicate(timeout=5)
                    self.fail("Isolated native recovery fixture timed out")
                self.assertEqual(process.returncode, 0, output + error)


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--worker":
        worker(sys.argv[2])
    else:
        unittest.main()
