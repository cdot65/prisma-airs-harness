"""Actual agent request-count checks against a local denying gateway fixture."""

import json
import unittest

import test_airs_harness as harness


class GatewayPolicyRetry(unittest.TestCase):
    def test_terminal_gateway_denials_are_not_replayed(self):
        for status, streamed in (
            (401, False),
            (403, False),
            (446, False),
            (200, False),
            (200, True),
        ):
            with self.subTest(status=status, streamed=streamed):
                fixture = harness.TerminalIntegration()
                fixture.setUp()
                try:
                    fixture.configure()
                    config = fixture.home / "config.toml"
                    config.write_text(
                        config.read_text().replace(
                            "[model_providers.airs]",
                            "[model_providers.airs]\nstream_max_retries = 2",
                        )
                    )

                    def deny(request):
                        body = json.loads(
                            request.rfile.read(int(request.headers["Content-Length"]))
                        )
                        fixture.requests.append((request.path, {}, body))
                        body = {
                            "error": {
                                "type": "hooks_failed",
                                "message": "PRIVATE-DENIAL-CANARY",
                            }
                        }
                        if streamed:
                            data = (
                                "data: "
                                + json.dumps(
                                    {"type": "response.failed", "response": body}
                                )
                                + "\n\n"
                            ).encode()
                        else:
                            data = json.dumps(body).encode()
                        request.send_response(status)
                        # Retry advice must not override terminal denial semantics.
                        request.send_header("Retry-After", "0")
                        request.send_header(
                            "Content-Type",
                            "text/event-stream" if streamed else "application/json",
                        )
                        request.send_header("Content-Length", str(len(data)))
                        request.end_headers()
                        request.wfile.write(data)

                    fixture.server.RequestHandlerClass.do_POST = deny
                    result = fixture.execute()
                    self.assertNotEqual(result.returncode, 0)
                    self.assertEqual(
                        [path for path, _, _ in fixture.requests],
                        ["/prefix/v1/responses"],
                        result.stderr,
                    )
                    if status == 200:
                        self.assertIn("policy denial", result.stderr)
                        self.assertNotIn("PRIVATE-DENIAL-CANARY", result.stderr)
                    self.assertFalse((fixture.work / "result.txt").exists())
                    self.assertEqual(fixture.mcp_requests, [])
                finally:
                    fixture.doCleanups()


if __name__ == "__main__":
    unittest.main()
