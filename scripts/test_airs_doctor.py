"""Installed-binary connection doctor acceptance; fixtures are not live SSO proof."""

import base64
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
import socket
import threading
import time
import tomllib
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

    def test_catalog_worker_does_not_inspect_credentials_or_contact_gateway(self):
        fixture = self.fixture
        catalog = tomllib.loads((fixture.home / "config.toml").read_text())[
            "model_catalog_json"
        ]
        fixture.env["AIRS_DOCTOR_CATALOG_PROBE"] = catalog
        # If dispatch accidentally reaches native diagnostics it must fail.
        fixture.env["AIRS_DOCTOR_STORAGE_PROBE"] = "1"
        fixture.env["DBUS_SESSION_BUS_ADDRESS"] = "unix:path=/nonexistent/airs-probe"
        for contents, expected in [("{}", 0), ("PRIVATE-CATALOG-CANARY", 2)]:
            with self.subTest(expected=expected):
                Path(catalog).write_text(contents)
                result = fixture.run_cli("doctor", "--json")
                self.assertEqual(result.returncode, expected, result.stderr)
                self.assertEqual(result.stdout, "")
                self.assertEqual(fixture.health_requests, [])
                self.assertEqual(fixture.requests, [])
                self.assertEqual(fixture.mcp_requests, [])
                self.assertNotIn("PRIVATE-CATALOG-CANARY", result.stderr)

    def test_access_timestamp_describes_only_the_explicit_inference_observation(self):
        fixture = self.fixture
        passive = json.loads(fixture.run_cli("doctor", "--json").stdout)
        self.assertIsNone(passive["gateway_access_checked_at"])
        self.assertEqual(fixture.requests, [])
        for status, body, passed in [
            (
                200,
                {
                    "object": "response",
                    "status": "completed",
                    "output": [{"type": "message"}],
                },
                True,
            ),
            (446, {"error": {"message": "PRIVATE-POLICY-CANARY"}}, False),
        ]:
            with self.subTest(status=status):
                fixture.probe_response = (status, body)
                started = int(time.time())
                result = fixture.run_cli("doctor", "--json", "--verify-access")
                report = json.loads(result.stdout)
                checked = report["gateway_access_checked_at"]
                self.assertGreaterEqual(checked, started)
                self.assertLessEqual(checked, int(time.time()))
                access = next(
                    row for row in report["checks"] if row["name"] == "gateway_access"
                )
                self.assertEqual(access["passed"], passed)
                expected = datetime.fromtimestamp(checked, timezone.utc).strftime(
                    "%Y-%m-%d %H:%M:%S UTC"
                )
                self.assertIn("Checked at: " + expected, access["detail"])
                self.assertNotIn("PRIVATE-POLICY-CANARY", result.stdout + result.stderr)
        self.assertEqual(len(fixture.requests), 2)
        self.assertEqual(fixture.mcp_requests, [])

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

    def test_redacted_report_preview_copy_and_save_reuse_one_inspection(self):
        fixture = self.fixture
        signed_in = fixture.run_cli("login", "--credential-env", "AIRS_TEST_CREDENTIAL")
        self.assertEqual(signed_in.returncode, 0, signed_in.stderr)
        # Exercise terminal-mediated copy without changing the host clipboard.
        fixture.env["SSH_CONNECTION"] = "127.0.0.1 1234 127.0.0.1 22"
        fixture.env.pop("TMUX", None)
        fixture.env.pop("TMUX_PANE", None)
        private = b"PRIVATE-REPORT-RESPONSE-CANARY"

        def failed_health(request):
            fixture.health_requests.append(request.path)
            request.send_response(503)
            request.send_header("Content-Length", str(len(private)))
            request.end_headers()
            request.wfile.write(private)

        fixture.server.RequestHandlerClass.do_GET = failed_health
        protected = [
            fixture.home / "config.toml",
            fixture.home / "credential-binding.json",
        ]
        with TerminalSession(harness.BINARY, fixture.env, fixture.work) as terminal:
            terminal.start()
            before = {path: path.read_bytes() for path in protected}
            terminal.send_line("/doctor")
            terminal.wait_for(b"Connection health", timeout=45)
            counts = (
                len(fixture.health_requests),
                len(fixture.requests),
                len(fixture.mcp_requests),
            )
            self.assertGreater(counts[0], 0)
            self.assertEqual(counts[1:], (0, 0))
            offset = len(terminal.transcript)
            # Report precedes the final MCP-manager action, independent of check count.
            os.write(terminal.master, b"\x1b[F\x1b[A\r")
            terminal.wait_for(b"Preview report", offset)
            self.assertEqual(list(fixture.home.glob("diagnostic-report-*.txt")), [])
            offset = len(terminal.transcript)
            self.choose(terminal)
            # The pager advances the cursor over spaces instead of writing them.
            terminal.wait_for(b"gateway_access:", offset)
            offset = len(terminal.transcript)
            os.write(terminal.master, b"\x1b")
            terminal.wait_for(b"Preview report", offset)
            offset = len(terminal.transcript)
            self.choose(terminal, 1)
            terminal.wait_for(b"\x1b]52;c;", offset)
            clipboard = re.compile(rb"\x1b\]52;c;([A-Za-z0-9+/=]+)\x07")
            terminal.wait_until(
                lambda: clipboard.search(terminal.transcript[offset:]) is not None
            )
            copied = base64.b64decode(
                clipboard.search(terminal.transcript[offset:])[1], validate=True
            )
            # Each refreshed report-action view starts at its first action.
            offset = len(terminal.transcript)
            self.choose(terminal, 2)
            terminal.wait_for(b"Saved locally:", offset)
            reports = list(fixture.home.glob("diagnostic-report-*.txt"))
            self.assertEqual(len(reports), 1)
            saved = reports[0].read_bytes()
            self.assertEqual(saved, copied)
            self.assertTrue(saved.startswith(b"AIRS diagnostic report v1\n"))
            self.assertLessEqual(len(saved), 8192)
            self.assertEqual(reports[0].stat().st_mode & 0o777, 0o600)
            self.assertIn(b"Needs attention", saved)
            self.assertIn(b"Not verified", saved)
            for canary in (
                private,
                b"test-only-credential",
                fixture.url.encode(),
                str(fixture.home).encode(),
                str(fixture.work).encode(),
            ):
                self.assertNotIn(canary, saved)
            self.assertNotIn(b"http://", saved)
            self.assertNotIn(b"\x1b", saved)
            self.assertEqual(
                counts,
                (
                    len(fixture.health_requests),
                    len(fixture.requests),
                    len(fixture.mcp_requests),
                ),
            )
        self.assertEqual({path: path.read_bytes() for path in protected}, before)
        for rollout in fixture.home.rglob("*.jsonl"):
            self.assertNotIn("AIRS diagnostic report", rollout.read_text())
            self.assertNotIn(private.decode(), rollout.read_text())

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
