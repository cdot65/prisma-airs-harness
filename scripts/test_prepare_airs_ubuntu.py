#!/usr/bin/env python3
"""Exercise Ubuntu host readiness against a disposable encrypted GNOME keyring.

Run on a prepared Ubuntu host, passing the preparation script path. A private
D-Bus and XDG data/runtime directories keep the owner's keyring out of the test.
The installed AIRS package and network readiness checks run unchanged.
"""

import os
from pathlib import Path
import pty
import re
import select
import signal
import subprocess
import sys
import tempfile
import time


def dbus(destination, path, method, *args):
    return subprocess.check_output(
        [
            "gdbus",
            "call",
            "--session",
            "--dest",
            destination,
            "--object-path",
            path,
            "--method",
            method,
            *args,
        ],
        text=True,
        timeout=10,
    )


def owner():
    result = dbus(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus.GetConnectionUnixProcessID",
        "org.freedesktop.secrets",
    )
    return int(re.search(r"uint32 (\d+)", result).group(1))


def run_preparation(script, mode, password=None):
    master, slave = pty.openpty()
    process = subprocess.Popen(
        ["bash", script, mode],
        stdin=slave,
        stdout=slave,
        stderr=slave,
        start_new_session=True,
    )
    os.close(slave)
    output = b""
    answered = set()
    prompts = (b"Keyring password (hidden): ", b"Confirm new keyring password: ")
    deadline = time.monotonic() + 120
    try:
        while time.monotonic() < deadline:
            if select.select([master], [], [], 0.1)[0]:
                try:
                    chunk = os.read(master, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                output += chunk
                for prompt in prompts:
                    if prompt in output and prompt not in answered:
                        assert password is not None, "Unexpected password prompt"
                        os.write(master, password.encode() + b"\n")
                        answered.add(prompt)
            elif process.poll() is not None:
                break
        else:
            raise AssertionError("Preparation timed out")
        code = process.wait(timeout=5)
    finally:
        os.close(master)
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
            process.wait(timeout=5)
    result = output.decode(errors="replace")
    if password is not None:
        assert password not in result, "Password leaked into output"
    return code, result, answered


def exercise(script):
    password = "disposable-readiness-fixture-password"
    try:
        code, output, prompts = run_preparation(script, "--unlock", password)
        assert code == 0, output
        assert len(prompts) == 2, "New encrypted keyring must require confirmation"
        assert "PASS Native credential store write/read passed." in output, output
        assert "PASS Disposable credential removed." in output, output
        keyring = Path(os.environ["XDG_DATA_HOME"]) / "keyrings/login.keyring"
        assert keyring.is_file(), "Keyring was not created in the isolated directory"
        first_owner = owner()
        print("PASS new encrypted keyring and native write/read/delete", flush=True)

        code, output, prompts = run_preparation(script, "--unlock")
        assert code == 0 and not prompts, output
        assert owner() == first_owner, "An unlocked service must not restart"
        print("PASS already-unlocked service is reused", flush=True)

        subprocess.run(
            [
                "secret-tool",
                "store",
                "--label=Preservation fixture",
                "application",
                "airs-readiness-fixture",
            ],
            input="disposable-saved-credential",
            text=True,
            check=True,
            timeout=10,
        )

        dbus(
            "org.freedesktop.secrets",
            "/org/freedesktop/secrets",
            "org.freedesktop.Secret.Service.Lock",
            "[objectpath '/org/freedesktop/secrets/collection/login']",
        )
        code, output, prompts = run_preparation(script, "--check")
        assert code == 1 and not prompts and "locked/unavailable" in output, output
        assert owner() == first_owner, "Read-only check must not replace the service"
        original = keyring.read_bytes()
        code, output, prompts = run_preparation(
            script, "--unlock", "wrong-fixture-password"
        )
        assert code == 1 and len(prompts) == 1, output
        assert "password originally chosen" in output, output
        assert keyring.read_bytes() == original, (
            "Failed unlock changed encrypted storage"
        )
        print(
            "PASS locked check and wrong-password diagnosis preserve the keyring",
            flush=True,
        )

        code, output, prompts = run_preparation(script, "--unlock", password)
        assert code == 0 and len(prompts) == 1, output
        assert "PASS Native credential store write/read passed." in output, output
        assert owner() != first_owner, "Unlock did not replace the locked daemon"
        saved = subprocess.check_output(
            ["secret-tool", "lookup", "application", "airs-readiness-fixture"],
            text=True,
            timeout=10,
        )
        assert saved.strip() == "disposable-saved-credential", (
            "Saved credential was not preserved"
        )
        print(
            "PASS correct password unlocks the D-Bus service after a failed attempt",
            flush=True,
        )
    finally:
        try:
            os.kill(owner(), signal.SIGTERM)
        except (ProcessLookupError, subprocess.SubprocessError):
            pass


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--isolated":
        exercise(sys.argv[2])
    elif len(sys.argv) == 2:
        script = str(Path(sys.argv[1]).resolve())
        with tempfile.TemporaryDirectory(
            prefix="airs-keyring-regression-"
        ) as temporary:
            environment = os.environ.copy()
            for name in (
                "GNOME_KEYRING_CONTROL",
                "DBUS_SESSION_BUS_ADDRESS",
                "DISPLAY",
                "WAYLAND_DISPLAY",
            ):
                environment.pop(name, None)
            for name, subdirectory in (
                ("XDG_DATA_HOME", "data"),
                ("XDG_RUNTIME_DIR", "run"),
            ):
                directory = Path(temporary) / subdirectory
                directory.mkdir(mode=0o700)
                environment[name] = str(directory)
            result = subprocess.run(
                [
                    "dbus-run-session",
                    "--",
                    sys.executable,
                    str(Path(__file__).resolve()),
                    "--isolated",
                    script,
                ],
                env=environment,
                timeout=600,
            )
            sys.exit(result.returncode)
    else:
        sys.exit("Usage: test_prepare_airs_ubuntu.py /path/to/prepare_airs_ubuntu.sh")
