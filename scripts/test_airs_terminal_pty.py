#!/usr/bin/env python3
"""Linux interactive startup/logout regression; no inference or real key required."""

import fcntl
import json
import os
from pathlib import Path
import pty
import select
import shutil
import struct
import subprocess
import tempfile
import termios
import time
import unittest

BINARY = Path(
    os.environ.get("AIRS_TERMINAL_BIN", "codex-rs/target/debug/airs-terminal")
).resolve()


class InteractiveTerminal(unittest.TestCase):
    def test_logout_keeps_running_environment_after_default_and_binary_change(self):
        with tempfile.TemporaryDirectory(prefix="airs-terminal-pty-") as directory:
            root = Path(directory)
            home = root / "state"
            work = root / "work"
            work.mkdir()
            binary = root / "airs-terminal"
            shutil.copy2(BINARY, binary)
            key = root / "credential"
            key.write_text("synthetic-interactive-test-key")
            key.chmod(0o600)
            env = dict(os.environ, AIRS_TERMINAL_HOME=str(home), TERM="xterm-256color")

            def cli(*args):
                result = subprocess.run(
                    [str(binary), *args],
                    env=env,
                    cwd=work,
                    capture_output=True,
                    text=True,
                    timeout=30,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                return result

            for name in ("work", "other"):
                cli(
                    "setup",
                    "--environment",
                    name,
                    "--gateway-url",
                    "https://gateway.invalid/v1",
                )
                cli("login", "--credential-file", str(key))
            cli("env", "use", "work")
            master, slave = pty.openpty()
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 120, 0, 0))

            def controlling_terminal():
                os.setsid()
                fcntl.ioctl(slave, termios.TIOCSCTTY, 0)

            process = subprocess.Popen(
                [str(binary), "--no-alt-screen"],
                env=env,
                cwd=work,
                stdin=slave,
                stdout=slave,
                stderr=slave,
                preexec_fn=controlling_terminal,
            )
            os.close(slave)
            transcript = bytearray()

            def read_until(marker, timeout=15):
                deadline = time.monotonic() + timeout
                while marker not in transcript and time.monotonic() < deadline:
                    ready, _, _ = select.select([master], [], [], 0.1)
                    if ready:
                        try:
                            chunk = os.read(master, 65536)
                        except OSError:
                            break
                        if not chunk:
                            break
                        transcript.extend(chunk)
                self.assertTrue(
                    marker in transcript, transcript.decode(errors="replace")[-2500:]
                )

            try:
                read_until(b"Yes, continue")
                # Onboarding deliberately drains typeahead after its first draw.
                time.sleep(0.3)
                os.write(master, b"\r")
                read_until(b"workspace credential")
                self.assertIn(b"Prisma AIRS Terminal", transcript)
                self.assertIn(b"environment:", transcript)
                self.assertNotIn(b"Ask Codex", transcript)
                cli("env", "use", "other")
                replacement = root / "replacement"
                shutil.copy2(BINARY, replacement)
                os.replace(replacement, binary)
                os.write(master, b"/logout")
                time.sleep(0.25)
                os.write(master, b"\r")
                deadline = time.monotonic() + 15
                while process.poll() is None and time.monotonic() < deadline:
                    ready, _, _ = select.select([master], [], [], 0.1)
                    if ready:
                        try:
                            transcript.extend(os.read(master, 65536))
                        except OSError:
                            break
                self.assertEqual(
                    process.wait(timeout=5),
                    0,
                    transcript.decode(errors="replace")[-2500:],
                )
                registry = json.loads((home / "environments.json").read_text())
                work_home = (
                    home / "environments" / registry["environments"]["work"]["id"]
                )
                other_home = (
                    home / "environments" / registry["environments"]["other"]["id"]
                )
                self.assertTrue((work_home / "logged-out").exists())
                self.assertIsNone(
                    json.loads((work_home / "credential-binding.json").read_text())[
                        "source"
                    ]
                )
                self.assertFalse((other_home / "logged-out").exists())
                self.assertIsNotNone(
                    json.loads((other_home / "credential-binding.json").read_text())[
                        "source"
                    ]
                )
                self.assertNotIn(b"synthetic-interactive-test-key", transcript)
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=10)
                os.close(master)


if __name__ == "__main__":
    unittest.main()
