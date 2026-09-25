"""Installed AIRS routing transactions; isolated gateway fixtures, no owner credentials."""

import json
import os
import threading
import time
import unittest

import pyte

import test_airs_harness as harness
from airs_harness_pty import TerminalSession


class GatewayRouting(unittest.TestCase):
    def setUp(self):
        self.fixture = harness.TerminalIntegration()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.fixture.configure()

    def choose(self, terminal):
        time.sleep(0.2)
        os.write(terminal.master, b"\r")

    def select(self, terminal, command, expected=b"Gateway routing applied"):
        offset = len(terminal.transcript)
        terminal.send_line(command)
        terminal.wait_for(b"Verify and use this routing?", offset)
        self.choose(terminal)
        terminal.wait_for(expected, offset, timeout=60)
        time.sleep(0.2)

    def assert_pair(self, request, config, model):
        _, headers, body = request
        headers = {name.lower(): value for name, value in headers.items()}
        self.assertEqual(headers.get("x-portkey-config"), config)
        self.assertEqual(body.get("model"), model)

    def test_config_model_default_denial_and_restart(self):
        fixture = self.fixture
        with TerminalSession(harness.BINARY, fixture.env, fixture.work) as terminal:
            terminal.start()
            self.select(terminal, "/config pc-first")
            self.assert_pair(fixture.requests[-1], "pc-first", None)
            self.select(terminal, "/model @fixture/override")
            self.assert_pair(fixture.requests[-1], "pc-first", "@fixture/override")
            self.select(terminal, "/config pc-first", b"Current routing accepted")
            self.assert_pair(fixture.requests[-1], "pc-first", "@fixture/override")
            # Denial must preserve both values and must not echo the response body.
            fixture.probe_response = (446, {"error": "PRIVATE-ROUTING-CANARY"})
            self.select(terminal, "/config pc-denied", b"Routing unchanged")
            self.assert_pair(fixture.requests[-1], "pc-denied", None)
            del fixture.probe_response
            self.select(terminal, "/config pc-first", b"Current routing accepted")
            self.assert_pair(fixture.requests[-1], "pc-first", "@fixture/override")
            self.select(terminal, "/config pc-second")
            self.assert_pair(fixture.requests[-1], "pc-second", None)
            offset = len(terminal.transcript)
            terminal.send_line("Confirm the selected route.")
            terminal.wait_for(b"Local tool complete.", offset)
            turns = [
                request
                for request in fixture.requests
                if request[2].get("stream") is not False
            ]
            self.assertTrue(turns)
            for request in turns:
                self.assert_pair(request, "pc-second", None)
            self.assertNotIn(b"PRIVATE-ROUTING-CANARY", terminal.transcript)
            terminal.send_line("/quit")
            terminal.wait_until(lambda: terminal.process.poll() is not None)
        rollouts = list(fixture.home.glob("sessions/**/*.jsonl"))
        self.assertEqual(len(rollouts), 1)
        rows = [json.loads(line) for line in rollouts[0].read_text().splitlines()]
        thread = next(
            row["payload"]["id"] for row in rows if row["type"] == "session_meta"
        )
        self.assertNotIn("connectivity check", rollouts[0].read_text())
        with TerminalSession(
            harness.BINARY,
            fixture.env,
            fixture.work,
            arguments=["--no-alt-screen", "resume", thread],
        ) as terminal:
            terminal.wait_for(b"permissions:")
            time.sleep(0.4)
            self.select(terminal, "/config pc-second", b"Current routing accepted")
            self.assert_pair(fixture.requests[-1], "pc-second", None)
            self.select(terminal, "/config default")
            self.assert_pair(fixture.requests[-1], None, None)
        for _, _, body in fixture.requests:
            if body.get("stream") is False:
                self.assertEqual(body["max_output_tokens"], 16)
                self.assertFalse(body["store"])
                self.assertNotIn("tools", body)
                self.assertIsInstance(body["input"], str)

    def test_open_cancel_and_invalid_selection_send_nothing(self):
        fixture = self.fixture
        with TerminalSession(harness.BINARY, fixture.env, fixture.work) as terminal:
            terminal.start()
            for command, title in [
                ("/config", b"AI Gateway configuration"),
                ("/model", b"AI Gateway model"),
                ("/config pc-cancel", b"Verify and use this routing?"),
            ]:
                offset = len(terminal.transcript)
                terminal.send_line(command)
                terminal.wait_for(title, offset)
                frame = terminal.transcript.index(title, offset) + len(title)
                terminal.wait_for(b"\x1b[?2026l", frame)
                # Match the other real-terminal key helpers: a partial header
                # write is not the end of the interactive-screen input boundary.
                time.sleep(0.2)
                os.write(terminal.master, b"\x1b")
                # Wait for the rendered composer, rather than sending another
                # command while the terminal is still processing cancellation.
                screen = pyte.Screen(120, 40)
                stream = pyte.ByteStream(screen)
                consumed = 0

                def composer_restored():
                    nonlocal consumed
                    stream.feed(bytes(terminal.transcript[consumed:]))
                    consumed = len(terminal.transcript)
                    visible = "\n".join(screen.display)
                    return "Ask AIRS Harness to work on your project" in visible

                terminal.wait_until(composer_restored)
            terminal.send_line('/config {"targets":[]}')
            terminal.wait_for(b"select a saved gateway config ID")
            self.assertEqual(fixture.requests, [])
            self.select(terminal, "/config pc-after-cancel")
            self.assertEqual(len(fixture.requests), 1)

    def test_cancel_slow_probe_ignores_its_eventual_success(self):
        fixture = self.fixture
        entered, release = threading.Event(), threading.Event()
        handler = fixture.server.RequestHandlerClass
        original = handler.do_POST

        def slow(request):
            entered.set()
            release.wait(15)
            try:
                original(request)
            except (BrokenPipeError, ConnectionResetError):
                pass

        handler.do_POST = slow
        self.addCleanup(release.set)
        with TerminalSession(harness.BINARY, fixture.env, fixture.work) as terminal:
            terminal.start()
            terminal.send_line("/config pc-cancelled")
            terminal.wait_for(b"Verify and use this routing?")
            self.choose(terminal)
            terminal.wait_for(b"Verifying gateway routing")
            self.assertTrue(entered.wait(5))
            os.write(terminal.master, b"\x1b")
            time.sleep(0.3)
            release.set()
            handler.do_POST = original
            time.sleep(0.3)
            self.select(terminal, "/config default", b"Current routing accepted")
            self.assert_pair(fixture.requests[-1], None, None)


if __name__ == "__main__":
    unittest.main()
