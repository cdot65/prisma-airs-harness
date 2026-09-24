"""Exercise the installed AIRS transcript overlay with a draft and terminal resize."""

import codecs
import fcntl
import os
from pathlib import Path
import struct
import termios
import threading
import tomllib
import json
import unittest

import pyte

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


class QuestionRecoveryTerminal(unittest.TestCase):
    def test_completed_turn_recovers_unsent_answer_into_existing_draft(self):
        fixture = harness.TerminalIntegration()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        fixture.configure()
        catalog_path = Path(
            tomllib.loads((fixture.home / "config.toml").read_text())[
                "model_catalog_json"
            ]
        )
        catalog = json.loads(catalog_path.read_text())
        for model in catalog["models"]:
            model["experimental_supported_tools"] = ["request_user_input_async"]
        catalog_path.write_text(json.dumps(catalog))
        fixture.question_turn_release = threading.Event()
        self.addCleanup(fixture.question_turn_release.set)
        fixture.initial_function_call = (
            "request_user_input_async",
            {"questions": [{"title": "Which environment should we inspect?"}]},
        )
        prompt = "Ask which environment to inspect."
        draft = "Preserve my independent instruction."
        answer = "Use the read-only fixture environment."
        combined = draft + "\n" + answer
        fixture.phase_replies = {
            prompt: "Question ready.",
            combined: "Recovered input received.",
        }
        with TerminalSession(harness.BINARY, fixture.env, fixture.work) as terminal:
            fixture.terminal_transcript = terminal.transcript
            terminal.start()
            terminal.send_line(prompt)
            terminal.wait_for(b"Question ready.")
            os.write(terminal.master, b"\x1b[200~" + draft.encode() + b"\x1b[201~")
            terminal.wait_for(draft.encode())
            offset = len(terminal.transcript)
            os.write(terminal.master, b"\x1b[1;3A")
            terminal.wait_for(b"enter submit", offset)
            os.write(terminal.master, b"\x1b[200~" + answer.encode() + b"\x1b[201~")
            screen = pyte.Screen(120, 40)
            stream = pyte.Stream(screen)
            decoder = codecs.getincrementaldecoder("utf-8")(errors="replace")
            consumed = 0

            def rendered():
                nonlocal consumed
                stream.feed(decoder.decode(terminal.transcript[consumed:]))
                consumed = len(terminal.transcript)
                return "\n".join(screen.display)

            terminal.wait_until(lambda: answer in rendered())
            offset = len(terminal.transcript)
            fixture.question_turn_release.set()
            terminal.wait_until(
                lambda: (
                    draft in rendered()
                    and answer in rendered()
                    and "enter submit" not in rendered()
                )
            )
            self.assertEqual(
                len(fixture.requests), 2, "Recovery must not submit the answer"
            )
            os.write(terminal.master, b"\r")
            terminal.wait_for(b"Recovered input received.", offset)
        self.assertEqual(harness.latest_user_text(fixture.requests[-1][2]), combined)
        self.assertEqual(len(fixture.requests), 3)
        self.assertEqual(fixture.mcp_requests, [])
        for path, _, body in fixture.requests:
            self.assertEqual(path, "/prefix/v1/responses")
            self.assertNotIn("model", body)
