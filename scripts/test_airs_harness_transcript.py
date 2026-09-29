"""Exercise the installed AIRS transcript overlay with a draft and terminal resize."""

import codecs
import fcntl
import os
from pathlib import Path
import struct
import signal
import termios
import threading
import tomllib
import json
import unittest

import pyte

import test_airs_harness as harness
from airs_harness_pty import TerminalSession


def conversation_requests(fixture):
    # Automatic naming is independent of the conversation turn. It may finish
    # while the user edits a draft; do not mistake it for answer submission.
    return [
        row
        for row in fixture.requests
        if not harness.latest_user_text(row[2]).startswith(
            "Generate a concise, single-line task title"
        )
    ]


class PaletteTerminal(TerminalSession):
    """Answer the real startup color probe without modifying product configuration."""

    def __init__(self, *args, background, **kwargs):
        self.background = background
        self.answered = {}
        super().__init__(*args, **kwargs)

    def wait_until(self, predicate, timeout=30):
        def observe():
            replies = {
                b"\x1b]10;?\x1b\\": b"\x1b]10;rgb:eeee/eeee/eeee\x1b\\",
                b"\x1b]11;?\x1b\\": b"\x1b]11;rgb:" + self.background + b"\x1b\\",
                b"\x1b[6n": b"\x1b[1;1R",
                b"\x1b[?u": b"\x1b[?0u",
                b"\x1b[c": b"\x1b[?1;2c",
            }
            for query, reply in replies.items():
                count = self.transcript.count(query)
                if count > self.answered.get(query, 0):
                    os.write(self.master, reply * (count - self.answered.get(query, 0)))
                    self.answered[query] = count
            return predicate()

        super().wait_until(observe, timeout)


class TranscriptTerminal(unittest.TestCase):
    def test_airs_menus_paint_blue_selection_and_move_it_with_keyboard(self):
        for background, expected in [
            (b"1111/1111/1111", "63a8f8"),
            (b"ffff/ffff/ffff", "a4cdfb"),
        ]:
            with self.subTest(background=background):
                fixture = harness.TerminalIntegration()
                fixture.setUp()
                self.addCleanup(fixture.doCleanups)
                fixture.configure()
                environment = dict(fixture.env, COLORTERM="truecolor", FORCE_COLOR="3")
                environment.pop("NO_COLOR", None)
                with PaletteTerminal(
                    harness.BINARY, environment, fixture.work, background=background
                ) as terminal:
                    fixture.terminal_transcript = terminal.transcript
                    terminal.start()
                    screen = pyte.Screen(120, 40)
                    stream = pyte.Stream(screen)
                    decoder = codecs.getincrementaldecoder("utf-8")(errors="replace")
                    consumed = 0

                    def selections():
                        nonlocal consumed
                        stream.feed(decoder.decode(terminal.transcript[consumed:]))
                        consumed = len(terminal.transcript)
                        return [
                            (y, text.strip())
                            for y, text in enumerate(screen.display)
                            if text.strip()
                            and any(
                                screen.buffer[y][x].bg == expected for x in range(120)
                            )
                        ]

                    for command, heading in [
                        ("/mcp", b"MCP connections"),
                        ("/typesafe", b"TypeSafe Jev"),
                    ]:
                        offset = len(terminal.transcript)
                        terminal.send_line(command)
                        terminal.wait_for(heading, offset)
                        terminal.wait_until(lambda: bool(selections()))
                        # A selected choice may occupy several adjacent wrapped rows.
                        first = selections()
                        os.write(terminal.master, b"\x1b[B")
                        terminal.wait_until(
                            lambda: bool(selections()) and selections() != first
                        )
                        rows = [y for y, _text in selections()]
                        self.assertEqual(rows, list(range(rows[0], rows[-1] + 1)))
                        os.write(terminal.master, b"\x1b")
                        terminal.wait_until(lambda: not selections())
                self.assertEqual(fixture.requests, [])
                self.assertEqual(fixture.mcp_requests, [])

    def test_harness_starts_fullscreen_without_interface_settings(self):
        fixture = harness.TerminalIntegration()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        # An existing empty file keeps the acceptance pin away: harness defaults apply.
        (fixture.home / "settings.toml").write_text("")
        fixture.configure()
        with TerminalSession(
            harness.BINARY, fixture.env, fixture.work, arguments=[]
        ) as terminal:
            fixture.terminal_transcript = terminal.transcript
            terminal.start()
            terminal.wait_for(b"\x1b[?1049h")
        self.assertEqual(fixture.requests, [])

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
            count = len(conversation_requests(fixture))
            offset = len(terminal.transcript)
            os.write(terminal.master, b"\x14")
            terminal.wait_for(b"\x1b[?1049h", offset)
            terminal.wait_for(b"q close", offset)
            screen = pyte.Screen(120, 40)
            stream = pyte.Stream(screen)
            decoder = codecs.getincrementaldecoder("utf-8")(errors="replace")
            consumed = 0

            def rendered():
                nonlocal consumed
                stream.feed(decoder.decode(terminal.transcript[consumed:]))
                consumed = len(terminal.transcript)
                return "\n".join(screen.display)

            rendered()
            for rows, columns in [(16, 40), (40, 120)]:
                screen.resize(lines=rows, columns=columns)
                screen.reset()
                fcntl.ioctl(
                    terminal.master,
                    termios.TIOCSWINSZ,
                    struct.pack("HHHH", rows, columns, 0, 0),
                )
                # Explicitly deliver the window-change signal for this PTY's
                # process group; a heartbeat is not evidence of resized layout.
                os.killpg(terminal.process.pid, signal.SIGWINCH)
                terminal.wait_until(lambda: "q close" in rendered().splitlines()[-2])
            offset = len(terminal.transcript)
            os.write(terminal.master, b"\x14")
            terminal.wait_for(b"\x1b[?1049l", offset)
            terminal.wait_until(lambda: draft in rendered())
            self.assertEqual(len(conversation_requests(fixture)), count)
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
                len(conversation_requests(fixture)),
                2,
                "Recovery must not submit the answer",
            )
            self.assertTrue(
                all(answer not in json.dumps(body) for _, _, body in fixture.requests)
            )
            os.write(terminal.master, b"\r")
            terminal.wait_for(b"Recovered input received.", offset)
        self.assertEqual(
            harness.latest_user_text(conversation_requests(fixture)[-1][2]), combined
        )
        self.assertEqual(len(conversation_requests(fixture)), 3)
        self.assertEqual(fixture.mcp_requests, [])
        for path, _, body in fixture.requests:
            self.assertEqual(path, "/prefix/v1/responses")
            self.assertNotIn("model", body)
