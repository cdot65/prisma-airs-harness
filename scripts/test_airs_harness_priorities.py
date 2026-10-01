"""Exercise selective Codex adoption through the installed AIRS agent and a real PTY.

The gateway is a deterministic loopback fixture. These checks inspect actual tool
results, pending input and clipboard payloads without inference credentials or
changes to the host's clipboard.
"""

import base64
import codecs
import json
import os
import re
import threading
import time
import unittest

import pyte

import test_airs_harness as harness
from airs_harness_pty import TerminalSession
from test_airs_harness_transcript import conversation_requests


class AdoptionTerminal(unittest.TestCase):
    def fixture(self):
        fixture = harness.TerminalIntegration()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        fixture.configure()
        return fixture

    def test_completed_command_reports_early_output_and_final_tail_once(self):
        fixture = self.fixture()
        fixture.initial_function_call = (
            "exec_command",
            {
                "cmd": "printf 'airs-016-early\\n'; sleep 0.1; printf 'airs-016-tail\\n'",
                "yield_time_ms": 1000,
            },
        )
        with TerminalSession(harness.BINARY, fixture.env, fixture.work) as terminal:
            terminal.start()
            terminal.send_line("Run the bounded command output fixture.")
            terminal.wait_for(b"Local tool complete.")
        requests = conversation_requests(fixture)
        outputs = [
            item["output"]
            for _, _, body in requests
            for item in body["input"]
            if item.get("type") == "function_call_output"
            and item.get("call_id") == "local-tool"
        ]
        self.assertEqual(len(outputs), 1)
        self.assertEqual(outputs[0].count("airs-016-early"), 1)
        self.assertEqual(outputs[0].count("airs-016-tail"), 1)
        self.assertIn("Process exited with code 0", outputs[0])
        self.assertEqual(len(requests), 2)

    def test_opt_in_steering_preempts_stream_without_replaying_completed_tool(self):
        self.exercise_steering(enabled=True)

    def test_default_steering_waits_for_stream_completion(self):
        self.exercise_steering(enabled=False)

    def exercise_steering(self, *, enabled):
        fixture = self.fixture()
        first = "Run the first steering fixture."
        steer = "Continue with the second steering fixture."
        fixture.phase_replies = {
            first: "Initial response held.",
            steer: "Steer received.",
        }
        fixture.question_turn_release = threading.Event()
        self.addCleanup(fixture.question_turn_release.set)
        args = ["--no-alt-screen"]
        if enabled:
            args += ["--enable", "instant_interrupt"]
        with TerminalSession(
            harness.BINARY, fixture.env, fixture.work, arguments=args
        ) as terminal:
            terminal.start()
            terminal.send_line(first)
            terminal.wait_for(b"Initial response held.")
            self.assertEqual(len(conversation_requests(fixture)), 2)
            offset = len(terminal.transcript)
            terminal.send_line(steer)
            if not enabled:
                until = time.monotonic() + 1.5
                terminal.wait_until(lambda: time.monotonic() >= until, timeout=3)
                self.assertEqual(len(conversation_requests(fixture)), 2)
                fixture.question_turn_release.set()
            terminal.wait_for(b"Steer received.", offset)
            self.assertEqual(
                fixture.question_turn_release.is_set(),
                not enabled,
                "Opt-in steering must proceed while the original stream is still held",
            )
            fixture.question_turn_release.set()
        requests = conversation_requests(fixture)
        self.assertEqual(len(requests), 3)
        self.assertEqual(harness.latest_user_text(requests[-1][2]), steer)
        calls = [
            item
            for item in requests[-1][2]["input"]
            if item.get("type") == "function_call"
            and item.get("call_id") == "local-tool"
        ]
        self.assertLessEqual(len(calls), 1)
        self.assertEqual(
            (fixture.work / "result.txt").read_text(), "local tool worked\n"
        )
        for path, _, body in requests:
            self.assertEqual(path, "/prefix/v1/responses")
            self.assertNotIn("model", body)

    def test_fullscreen_selection_copies_markdown_and_keeps_unsent_draft(self):
        fixture = self.fixture()
        prompt = "Render the clipboard fixture."
        draft = "Unsent protected draft"
        fixture.phase_replies = {
            prompt: "**Priority bold** and `priority_code`\n\n- first fixture item\n- second fixture item",
            draft: "Protected draft received.",
        }
        (fixture.home / "settings.toml").write_text(
            '[tui]\nfullscreen_transcript = true\ncopy_on_select = "always"\n'
            'right_click_paste = "off"\n'
        )
        # Terminal-mediated copy is observable without writing to the host clipboard.
        environment = dict(fixture.env, SSH_CONNECTION="127.0.0.1 1 127.0.0.1 2")
        environment.pop("TMUX", None)
        environment.pop("ZELLIJ", None)
        with TerminalSession(
            harness.BINARY, environment, fixture.work, arguments=[]
        ) as terminal:
            terminal.start()
            terminal.send_line(prompt)
            terminal.wait_for(b"second fixture item")
            os.write(terminal.master, b"\x1b[200~" + draft.encode() + b"\x1b[201~")
            terminal.wait_for(draft.encode())
            screen = pyte.Screen(120, 40)
            stream = pyte.Stream(screen)
            decoder = codecs.getincrementaldecoder("utf-8")(errors="replace")
            consumed = 0

            def rendered():
                nonlocal consumed
                stream.feed(decoder.decode(terminal.transcript[consumed:]))
                consumed = len(terminal.transcript)
                return screen.display

            terminal.wait_until(lambda: any(draft in row for row in rendered()))
            rows = rendered()
            start_y = next(y for y, row in enumerate(rows) if "Priority bold" in row)
            start_x = rows[start_y].index("Priority bold")
            end_y = next(
                y for y, row in enumerate(rows) if "second fixture item" in row
            )
            end_x = rows[end_y].index("second fixture item") + len(
                "second fixture item"
            )
            offset = len(terminal.transcript)
            for button, x, y in [
                (0, start_x, start_y),
                (32, end_x, end_y),
                (0, end_x, end_y),
            ]:
                ending = "m" if button == 0 and y == end_y else "M"
                os.write(
                    terminal.master, f"\x1b[<{button};{x + 1};{y + 1}{ending}".encode()
                )
                # Separate actual gestures so the terminal can paint each selection state.
                until = time.monotonic() + 0.1
                terminal.wait_until(lambda: time.monotonic() >= until, timeout=2)
            terminal.wait_for(b"\x1b]52;", offset)
            matches = re.findall(
                rb"\x1b\]52;c;([A-Za-z0-9+/=]+)(?:\x07|\x1b\\)",
                terminal.transcript[offset:],
            )
            self.assertTrue(matches, "The selection must emit an actual OSC 52 payload")
            copied = base64.b64decode(matches[-1]).decode()
            self.assertEqual(
                copied,
                "**Priority bold** and `priority_code`\n\n- first fixture item\n- second fixture item",
            )
            self.assertEqual(len(conversation_requests(fixture)), 2)
            self.assertTrue(any(draft in row for row in rendered()))
            self.assertTrue(screen.buffer[start_y][start_x].reverse)
            # Unconfirmed clipboard delivery must retain the selection. Escape returns
            # typing to the composer, and the existing draft is submitted exactly once.
            os.write(terminal.master, b"\x1b")
            until = time.monotonic() + 0.2
            terminal.wait_until(lambda: time.monotonic() >= until, timeout=2)
            os.write(terminal.master, b"\r")
            terminal.wait_for(b"Protected draft received.")
        requests = conversation_requests(fixture)
        self.assertEqual(len(requests), 3)
        self.assertEqual(harness.latest_user_text(requests[-1][2]), draft)


if __name__ == "__main__":
    unittest.main()
