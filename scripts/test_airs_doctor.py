"""Installed-binary connection doctor acceptance; fixtures are not live SSO proof."""

import json
import os
import socket
import threading
import time
import unittest

import test_airs_harness as harness
from airs_harness_pty import TerminalSession


class SessionDoctor(unittest.TestCase):
    def setUp(self):
        self.fixture = harness.TerminalIntegration()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.fixture.configure()

    def choose(self, terminal, down=0):
        time.sleep(0.15)
        os.write(terminal.master, b"\x1b[B" * down + b"\r")

    def native_binding(self):
        fixture = self.fixture
        result = fixture.run_cli("login", "--credential-env", "AIRS_TEST_CREDENTIAL")
        self.assertEqual(result.returncode, 0, result.stderr)
        path = fixture.home / "credential-binding.json"
        binding = json.loads(path.read_text())
        binding["source"] = {"kind": "keyring-v2"}
        path.write_text(json.dumps(binding))
        fixture.env["AIRS_DOCTOR_CONNECTION_HEALTH"] = "1"
        return path

    def test_environment_pinning_explicit_probe_and_failed_report(self):
        fixture = self.fixture
        with TerminalSession(harness.BINARY, fixture.env, fixture.work) as terminal:
            terminal.start()
            other = fixture.run_cli(
                "env",
                "create",
                "other",
                "--gateway-url",
                fixture.url.replace("/prefix/", "/other/"),
                "--allow-http-loopback",
                "--credential-env",
                "AIRS_TEST_CREDENTIAL",
            )
            self.assertEqual(other.returncode, 0, other.stderr)
            terminal.send_line("/doctor")
            terminal.wait_for(b"Connection health", timeout=45)
            self.assertEqual(fixture.requests, [])
            self.choose(terminal, 1)
            terminal.wait_for(b"Verify gateway access?")
            self.assertEqual(fixture.requests, [])
            self.choose(terminal)
            offset = len(terminal.transcript)
            terminal.wait_for(b"Connection health", offset, timeout=45)
            self.assertEqual(len(fixture.requests), 1)
            path, _, body = fixture.requests[0]
            self.assertEqual(path, "/prefix/v1/responses")
            self.assertEqual(body["max_output_tokens"], 16)
            self.assertFalse(body["store"])
            self.assertNotIn("tools", body)
            fixture.probe_response = (403, {"error": "PRIVATE-DOCTOR-CANARY"})
            self.choose(terminal, 1)
            terminal.wait_for(b"Verify gateway access?", offset)
            self.choose(terminal)
            offset = len(terminal.transcript)
            terminal.wait_for(b"Connection health", offset, timeout=45)
            # Scroll through diagnostic rows to the failed gateway access check.
            time.sleep(0.15)
            os.write(terminal.master, b"\x1b[F")
            terminal.wait_for(b"403", offset)
            self.assertEqual(len(fixture.requests), 2)
            self.assertNotIn(b"PRIVATE-DOCTOR-CANARY", terminal.transcript)
            self.assertNotIn(b"test-only-credential", terminal.transcript)
            self.choose(terminal)  # last row hands off to the existing MCP manager
            terminal.wait_for(b"MCP connections", offset)
            terminal.wait_for(b"Add gateway MCP server", offset)
            self.assertEqual(len(fixture.requests), 2)
        self.assertTrue(
            all(path == "/prefix/v1/health" for path in fixture.health_requests)
        )
        for rollout in fixture.home.rglob("*.jsonl"):
            self.assertNotIn("PRIVATE-DOCTOR-CANARY", rollout.read_text())
            self.assertNotIn("connectivity check", rollout.read_text())

    def test_cancel_slow_diagnostics_then_retry(self):
        fixture = self.fixture
        # The fixture's server is unique to this test. Hold just its health request.
        handler = fixture.server.RequestHandlerClass
        original = handler.do_GET
        entered, release = threading.Event(), threading.Event()

        def slow_health(request):
            entered.set()
            release.wait(15)
            original(request)

        handler.do_GET = slow_health
        self.addCleanup(release.set)
        with TerminalSession(harness.BINARY, fixture.env, fixture.work) as terminal:
            terminal.start()
            terminal.send_line("/doctor")
            terminal.wait_for(b"Checking this environment")
            self.assertTrue(entered.wait(5))
            os.write(terminal.master, b"\x1b")
            time.sleep(0.3)
            release.set()
            handler.do_GET = original
            offset = len(terminal.transcript)
            terminal.send_line("/doctor")
            terminal.wait_for(b"Connection health", offset, timeout=45)
            self.assertEqual(fixture.requests, [])

    @unittest.skipUnless(
        os.name == "posix" and os.uname().sysname == "Linux",
        "Linux native service diagnostics",
    )
    def test_missing_native_service_leaves_binding_and_cleanup_untouched(self):
        fixture = self.fixture
        path = self.native_binding()
        before = path.read_bytes()
        cleanup = fixture.home / "credential-pending-cleanup.json"
        cleanup.write_text("PRIVATE-PENDING-CLEANUP")
        fixture.env["DBUS_SESSION_BUS_ADDRESS"] = (
            "unix:path=/nonexistent/airs-doctor-test"
        )
        started = time.monotonic()
        result = fixture.run_cli("doctor", "--json")
        self.assertLess(time.monotonic() - started, 20)
        self.assertNotEqual(result.returncode, 0)
        report = json.loads(result.stdout)
        checks = {row["name"]: row for row in report["checks"]}
        self.assertEqual(report["authentication"], "Workspace API key")
        self.assertFalse(checks["credential_service"]["passed"])
        self.assertFalse(checks["credential_cleanup"]["passed"])
        self.assertEqual(path.read_bytes(), before)
        self.assertEqual(cleanup.read_text(), "PRIVATE-PENDING-CLEANUP")
        self.assertNotIn("PRIVATE-PENDING-CLEANUP", result.stdout + result.stderr)
        self.assertNotIn("test-only-credential", result.stdout + result.stderr)
        self.assertEqual(fixture.requests, [])

    @unittest.skipUnless(
        os.name == "posix" and os.uname().sysname == "Linux",
        "Linux native service diagnostics",
    )
    def test_stalled_native_service_probe_is_bounded(self):
        fixture = self.fixture
        path = self.native_binding()
        before = path.read_bytes()
        endpoint = fixture.root / "stalled-bus.sock"
        server = socket.socket(socket.AF_UNIX)
        server.bind(str(endpoint))
        server.listen(1)
        server.settimeout(10)
        self.addCleanup(server.close)
        connected, release = threading.Event(), threading.Event()
        self.addCleanup(release.set)

        def stall():
            connection, _ = server.accept()
            with connection:
                connected.set()
                release.wait(15)

        thread = threading.Thread(target=stall, daemon=True)
        thread.start()
        fixture.env["DBUS_SESSION_BUS_ADDRESS"] = f"unix:path={endpoint}"
        started = time.monotonic()
        result = fixture.run_cli("doctor", "--json")
        self.assertTrue(connected.is_set())
        self.assertLess(time.monotonic() - started, 12)
        checks = {row["name"]: row for row in json.loads(result.stdout)["checks"]}
        self.assertFalse(checks["credential_service"]["passed"])
        self.assertIn("bounded check", checks["credential_service"]["detail"])
        self.assertEqual(path.read_bytes(), before)
        self.assertEqual(fixture.requests, [])
        release.set()
        thread.join(timeout=2)


if __name__ == "__main__":
    unittest.main()
