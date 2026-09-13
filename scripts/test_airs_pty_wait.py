import unittest
from unittest.mock import patch

from airs_harness_pty import TerminalSession


class PtyExitOrdering(unittest.TestCase):
    def test_eof_before_process_exit_does_not_fail_exit_wait(self):
        terminal = TerminalSession.__new__(TerminalSession)
        terminal.master = 123
        terminal.transcript = bytearray()
        polls = iter([False, False, True, True])
        with (
            patch("airs_harness_pty.select.select", return_value=([123], [], [])),
            patch("airs_harness_pty.os.read", return_value=b""),
            patch("airs_harness_pty.time.sleep"),
        ):
            terminal.wait_until(lambda: next(polls), timeout=1)

    def test_eof_does_not_satisfy_an_unmet_condition(self):
        terminal = TerminalSession.__new__(TerminalSession)
        terminal.master = 123
        terminal.transcript = bytearray(b"child closed its terminal")
        with (
            patch("airs_harness_pty.select.select", return_value=([123], [], [])),
            patch("airs_harness_pty.os.read", return_value=b""),
        ):
            with self.assertRaisesRegex(AssertionError, "child closed its terminal"):
                terminal.wait_until(lambda: False, timeout=0.02)
