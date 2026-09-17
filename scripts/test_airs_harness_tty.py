"""Verify the macOS sandbox backport against the exact standalone executable.

The unsandboxed control and injected bytes stay inside disposable PTYs. A host
that cannot exercise Seatbelt fails this acceptance check rather than passing it.
"""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from airs_harness_pty import TerminalSession
from test_airs_harness import BINARY


PROBE = r"""
import errno, fcntl, os, select, sys, termios, tty
original = termios.tcgetattr(0)
try:
    tty.setraw(0)
    print('__ready__', flush=True)
    assert os.read(0, 1) == b'k'
    try:
        fcntl.ioctl(0, termios.TIOCSTI, b'x')
    except OSError as error:
        assert sys.argv[1] == 'deny', error
        assert error.errno == errno.EPERM, error
        assert not select.select([0], [], [], 0)[0]
    else:
        assert os.read(0, 1) == b'x'
        assert sys.argv[1] == 'allow', 'TIOCSTI unexpectedly succeeded'
finally:
    termios.tcsetattr(0, termios.TCSANOW, original)
print('__passed__', flush=True)
"""


@unittest.skipUnless(sys.platform == "darwin", "requires macOS Seatbelt")
class MacTerminalSandboxTests(unittest.TestCase):
    def test_sandbox_denies_injection_that_succeeds_in_control(self):
        with tempfile.TemporaryDirectory(prefix="airs-tty-acceptance-") as directory:
            root = Path(directory)
            env = dict(
                os.environ,
                AIRS_HARNESS_HOME=str(root / "state"),
                AIRS_TEST_CREDENTIAL="synthetic-unused-key",
            )
            subprocess.run(
                [
                    str(BINARY),
                    "env",
                    "create",
                    "work",
                    "--gateway-url",
                    "http://127.0.0.1:1/v1",
                    "--allow-http-loopback",
                    "--credential-env",
                    "AIRS_TEST_CREDENTIAL",
                ],
                env=env,
                cwd=root,
                capture_output=True,
                check=True,
                timeout=20,
            )
            for binary, arguments in (
                (Path("/usr/bin/python3"), ["-c", PROBE, "allow"]),
                (
                    BINARY,
                    [
                        "sandbox",
                        "-P",
                        ":read-only",
                        "--",
                        "/usr/bin/python3",
                        "-c",
                        PROBE,
                        "deny",
                    ],
                ),
            ):
                with TerminalSession(
                    binary, env, root, arguments=arguments
                ) as terminal:
                    terminal.wait_for(b"__ready__")
                    os.write(terminal.master, b"k")
                    terminal.wait_for(b"__passed__")
                    self.assertEqual(terminal.process.wait(timeout=10), 0)


if __name__ == "__main__":
    unittest.main()
