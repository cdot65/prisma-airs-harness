#!/usr/bin/env python3
"""Exercise the built standalone agent against a deterministic Responses gateway.

Run: python3 -m unittest discover -s scripts -p test_airs_terminal.py -v
AIRS_TERMINAL_BIN optionally selects an installed executable. No inference key,
PAH service, network beyond loopback, or Python third-party package is required.
"""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

BINARY = Path(
    os.environ.get("AIRS_TERMINAL_BIN", "codex-rs/target/debug/airs-terminal")
).resolve()
EXPLICIT = "@test/org/model:version"


class TerminalIntegration(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="airs-terminal-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.home = self.root / "state"
        self.work = self.root / "work"
        self.work.mkdir()
        self.requests = []
        self.redirect = None
        self.env = dict(
            os.environ,
            AIRS_TERMINAL_HOME=str(self.home),
            AIRS_TEST_CREDENTIAL="test-only-credential",
        )
        owner = self

        class Gateway(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
                owner.requests.append((self.path, dict(self.headers), body))
                if owner.redirect:
                    self.send_response(307)
                    self.send_header("Location", owner.redirect)
                    self.end_headers()
                    return
                number = len(owner.requests)
                events = [
                    {"type": "response.created", "response": {"id": f"resp-{number}"}}
                ]
                if number == 1:
                    names = [tool.get("name") for tool in body["tools"]]
                    name = (
                        "exec_command" if "exec_command" in names else "shell_command"
                    )
                    command = "printf 'local tool worked\\n' > result.txt && test -z \"${AIRS_TEST_CREDENTIAL+x}\" && cat result.txt"
                    args = (
                        {"cmd": command}
                        if name == "exec_command"
                        else {"command": command}
                    )
                    events.append(
                        {
                            "type": "response.output_item.done",
                            "item": {
                                "type": "function_call",
                                "call_id": "local-tool",
                                "name": name,
                                "arguments": json.dumps(args),
                            },
                        }
                    )
                else:
                    events.append(
                        {
                            "type": "response.output_item.done",
                            "item": {
                                "type": "message",
                                "role": "assistant",
                                "id": "msg-done",
                                "content": [
                                    {
                                        "type": "output_text",
                                        "text": "Local tool complete.",
                                    }
                                ],
                            },
                        }
                    )
                events.append(
                    {"type": "response.completed", "response": {"id": f"resp-{number}"}}
                )
                data = "".join(
                    f"event: {e['type']}\ndata: {json.dumps(e)}\n\n" for e in events
                ).encode()
                self.send_response(200)
                self.send_header("Content-Type", "text/event-stream")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Gateway)
        thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        thread.start()
        self.addCleanup(self.server.server_close)
        self.addCleanup(self.server.shutdown)
        self.url = f"http://127.0.0.1:{self.server.server_port}/prefix/v1"

    def run_cli(self, *args):
        return subprocess.run(
            [str(BINARY), *args],
            cwd=self.work,
            env=self.env,
            capture_output=True,
            text=True,
            timeout=60,
        )

    def configure(self):
        result = self.run_cli(
            "setup",
            "--gateway-url",
            self.url,
            "--allow-http-loopback",
            "--credential-env",
            "AIRS_TEST_CREDENTIAL",
            "--context-window",
            "32768",
            "--model",
            EXPLICIT,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn(
            "test-only-credential", (self.home / "config.toml").read_text()
        )

    def execute(self, *extra):
        return self.run_cli(
            "exec",
            "--skip-git-repo-check",
            "--ephemeral",
            "-s",
            os.environ.get("AIRS_TERMINAL_TEST_SANDBOX", "workspace-write"),
            *extra,
            "Create result.txt using a local shell tool and verify its contents.",
        )

    def assert_tool_loop(self, model):
        self.configure()
        legacy_state = self.root / "codex-state"
        self.env["CODEX_SQLITE_HOME"] = str(legacy_state)
        result = self.execute(*(["-m", model] if model else []))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.work / "result.txt").read_text(), "local tool worked\n")
        self.assertEqual(len(self.requests), 2, result.stderr)
        self.assertFalse(
            legacy_state.exists(), "standalone state must not use Codex's directory"
        )
        for path, headers, body in self.requests:
            self.assertEqual(path, "/prefix/v1/responses")
            headers = {k.lower(): v for k, v in headers.items()}
            self.assertEqual(headers["x-portkey-api-key"], "test-only-credential")
            self.assertTrue(headers["user-agent"].startswith("airs-terminal/"))
            self.assertNotIn("authorization", headers)
            if model:
                self.assertEqual(body["model"], model)
            else:
                self.assertNotIn("model", body)
        outputs = [
            item
            for item in self.requests[1][2]["input"]
            if item.get("type") == "function_call_output"
        ]
        self.assertEqual(len(outputs), 1)
        self.assertIn("local tool worked", outputs[0]["output"])
        self.assertIn("Process exited with code 0", outputs[0]["output"])

    def test_gateway_default_local_tool_and_continuation(self):
        self.assert_tool_loop(None)

    def test_explicit_route_local_tool_and_continuation(self):
        self.assert_tool_loop(EXPLICIT)

    def test_unconfigured_and_invalid_routes_fail_before_inference(self):
        result = self.execute()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("airs-terminal setup", result.stderr)
        self.configure()
        for args in [
            ("-m", "unqualified-model"),
            ("-m", "@test/unknown-capabilities"),
            ("-c", 'model_provider="openai"'),
        ]:
            result = self.execute(*args)
            self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.requests, [])

    def test_setup_refuses_overwrite(self):
        self.configure()
        before = (self.home / "config.toml").read_bytes()
        result = self.run_cli(
            "setup",
            "--gateway-url",
            self.url,
            "--allow-http-loopback",
            "--context-window",
            "8192",
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual((self.home / "config.toml").read_bytes(), before)
        if os.name == "posix":
            self.assertEqual(self.home.stat().st_mode & 0o777, 0o700)
            self.assertEqual((self.home / "config.toml").stat().st_mode & 0o777, 0o600)

    def test_setup_without_context_flag_persists_default(self):
        result = self.run_cli(
            "setup", "--gateway-url", self.url, "--allow-http-loopback"
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        catalog = json.loads((self.home / "models.json").read_text())
        self.assertEqual(catalog["models"][0]["context_window"], 1_000_000)
        self.assertIn(
            "model_context_window = 1000000", (self.home / "config.toml").read_text()
        )

    def test_missing_credential_fails_without_inference(self):
        self.configure()
        self.env.pop("AIRS_TEST_CREDENTIAL")
        result = self.execute()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("AIRS_TEST_CREDENTIAL", result.stderr)
        self.assertEqual(self.requests, [])

    def test_inference_redirect_is_not_followed(self):
        self.configure()
        self.redirect = self.url + "/credential-leak"
        result = self.execute(
            "-c",
            "model_providers.airs.request_max_retries=0",
            "-c",
            "model_providers.airs.stream_max_retries=0",
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([r[0] for r in self.requests], ["/prefix/v1/responses"])


if __name__ == "__main__":
    unittest.main()
