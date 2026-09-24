"""Exercise the installed AIRS transcript overlay with a draft and terminal resize."""

import fcntl
import os
import struct
import termios
import unittest

import test_airs_harness as harness
from airs_harness_pty import TerminalSession


class TranscriptTerminal(unittest.TestCase):
    def test_transcript_resize_restores_draft_without_replaying_inference(self):
        fixture = harness.TerminalIntegration()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        fixture.configure()
        draft = "Keep this unsent AIRS draft"
        with TerminalSession(
            harness.BINARY, fixture.env, fixture.work, arguments=[]
        ) as terminal:
            fixture.terminal_transcript = terminal.transcript
            terminal.start()
            terminal.send_line("Create result.txt using a local shell tool.")
            terminal.wait_for(b"Local tool complete.")
            os.write(terminal.master, b"\x1b[200~" + draft.encode() + b"\x1b[201~")
            terminal.wait_for(draft.encode())
            count = len(fixture.requests)
            offset = len(terminal.transcript)
            os.write(terminal.master, b"\x14")
            terminal.wait_for(b"\x1b[?1049h", offset)
            for rows, columns in [(16, 40), (40, 120)]:
                offset = len(terminal.transcript)
                fcntl.ioctl(
                    terminal.master,
                    termios.TIOCSWINSZ,
                    struct.pack("HHHH", rows, columns, 0, 0),
                )
                terminal.wait_until(lambda: len(terminal.transcript) > offset)
                offset = len(terminal.transcript)
            os.write(terminal.master, b"\x14")
            terminal.wait_for(draft.encode(), offset)
            self.assertEqual(len(fixture.requests), count)
            self.assertEqual(fixture.mcp_requests, [])
            for path, _, body in fixture.requests:
                self.assertEqual(path, "/prefix/v1/responses")
                self.assertNotIn("model", body)
