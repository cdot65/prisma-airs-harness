"""Installed skill acceptance through the real agent tool/approval boundary.

The scripted gateway selects the documented tool arguments; this is not a test
of model instruction-following. A private native store and loopback SDK fixture
exercise execution without owner credentials or paid requests. Run with
AIRS_TYPESAFE_NATIVE=1 AIRS_HARNESS_BIN=<installed airs> python3 -m unittest
discover -s scripts -p test_airs_typesafe_agent.py -v.
"""

import http.server
import json
import os
from pathlib import Path
import secrets
import shlex
import shutil
import subprocess
import sys
import threading
import unittest

from airs_native_test_store import native_store, native_test_command


def exercise(binary):
    os.environ["AIRS_HARNESS_BIN"] = binary
    from airs_harness_pty import TerminalSession
    from test_airs_harness import TerminalIntegration

    receipts = []

    class AgentJudge(TerminalIntegration):
        def runTest(self):
            with native_store(self.root / "native", self.env) as env:
                self.env = {
                    k: v for k, v in env.items() if not k.startswith("TYPESAFE_")
                }
                self.env.update(
                    AIRS_HARNESS_HOME=str(self.home),
                    AIRS_TEST_CREDENTIAL="test-only-credential",
                )
                if os.environ.get("AIRS_MANAGED_CLI"):
                    self.env["AIRS_MANAGED_CLI"] = os.environ["AIRS_MANAGED_CLI"]
                self.configure()
                saved_key = secrets.token_urlsafe(24)
                inherited_key = secrets.token_urlsafe(24)
                accepted_key = saved_key
                requests = []
                literal = (
                    '{"messages":[{"role":"user","content":"literal model output"}]}'
                )
                (self.work / "scan.json").write_text(
                    json.dumps([{"prompt": "synthetic prompt", "output": literal}])
                )

                class Provider(http.server.BaseHTTPRequestHandler):
                    def log_message(self, *_args):
                        pass

                    def do_POST(self):
                        body = json.loads(
                            self.rfile.read(int(self.headers["Content-Length"]))
                        )
                        authenticated = (
                            self.headers.get("Authorization")
                            == f"Bearer {accepted_key}"
                        )
                        if self.path == "/v1/systemone" and authenticated:
                            requests.append(body)
                            status = 200
                            response = {
                                "model": "jev-latest",
                                "usage": {},
                                "answers": {
                                    "objective_achieved": {"type": "noul", "noul": 0.1},
                                    "response_disposition": {
                                        "type": "choice",
                                        "choice": "full_refusal",
                                        "confidence": 1.0,
                                        "probabilities": {"full_refusal": 1.0},
                                    },
                                    "objective_fulfillment": {
                                        "type": "score",
                                        "score": 0.0,
                                        "confidence": 1.0,
                                        "legend": {"0": "none"},
                                        "probabilities": {"0": 1.0},
                                    },
                                },
                            }
                        else:
                            status, response = (
                                401,
                                {"error": "fixture authorization failed"},
                            )
                        data = json.dumps(response).encode()
                        self.send_response(status)
                        self.send_header("Content-Type", "application/json")
                        self.send_header("Content-Length", str(len(data)))
                        self.end_headers()
                        self.wfile.write(data)

                server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Provider)
                threading.Thread(target=server.serve_forever, daemon=True).start()
                prefix = [binary, "--environment", "work", "env", "typesafe"]

                def command(args, text=None):
                    result = subprocess.run(
                        args,
                        input=text,
                        env=self.env,
                        cwd=self.work,
                        capture_output=True,
                        text=True,
                        timeout=60,
                    )
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertNotIn(saved_key, result.stdout + result.stderr)
                    self.assertNotIn(inherited_key, result.stdout + result.stderr)
                    return result

                command(
                    [
                        *prefix,
                        "set",
                        "--stdin",
                        "--base-url",
                        f"http://127.0.0.1:{server.server_port}",
                    ],
                    saved_key,
                )
                try:
                    for mode in ("denied", "saved", "inherited"):
                        with self.subTest(mode=mode):
                            self.requests.clear()
                            if mode == "inherited":
                                accepted_key = inherited_key
                                self.env.update(
                                    TYPESAFE_API_KEY=inherited_key,
                                    TYPESAFE_BASE_URL=f"http://127.0.0.1:{server.server_port}",
                                )
                            script = (
                                self.home
                                / "skills/.system/prisma-airs-asr-judge/scripts/asr_judge.mjs"
                            )
                            self.initial_function_call = (
                                "exec_command",
                                {
                                    "cmd": shlex.join(
                                        [
                                            shutil.which("node"),
                                            str(script),
                                            "scan.json",
                                            "--out",
                                            mode,
                                            "--limit",
                                            "1",
                                            "--record",
                                            f"{mode}/record.json",
                                        ]
                                    ),
                                    "sandbox_permissions": "require_escalated",
                                    "justification": "Allow this judge command to read its test credential and send one synthetic record to the local TypeSafe fixture?",
                                    "yield_time_ms": 10000,
                                },
                            )
                            before = len(requests)
                            with TerminalSession(
                                binary,
                                self.env,
                                self.work,
                                arguments=[
                                    "--no-alt-screen",
                                    "-s",
                                    "workspace-write",
                                    "-a",
                                    "on-request",
                                ],
                            ) as terminal:
                                # Startup trust is only requested on the first launch.
                                if mode == "denied":
                                    terminal.start()
                                else:
                                    terminal.wait_for(b"permissions:")
                                # Verify the installed binary actually embeds the fix.
                                # Never overlay source instructions and pass old bytes
                                # as acceptance of the candidate being shipped.
                                source = (
                                    Path(__file__).resolve().parents[1]
                                    / "codex-rs/skills/src/assets/samples/prisma-airs-asr-judge/SKILL.md"
                                )
                                self.assertEqual(
                                    (script.parent.parent / "SKILL.md").read_bytes(),
                                    source.read_bytes(),
                                )
                                terminal.send_line("$prisma-airs-asr-judge scan.json")
                                terminal.wait_for(b"Yes, proceed")
                                self.assertEqual(
                                    len(requests), before, "Judge ran before approval"
                                )
                                offset = len(terminal.transcript)
                                os.write(
                                    terminal.master,
                                    b"\x1b" if mode == "denied" else b"\r",
                                )
                                terminal.wait_for(
                                    b"Conversation interrupted"
                                    if mode == "denied"
                                    else b"Local tool complete.",
                                    offset,
                                    timeout=60,
                                )
                            outputs = [
                                item
                                for _, _, body in self.requests[1:]
                                for item in body["input"]
                                if item.get("type") == "function_call_output"
                            ]
                            serialized = json.dumps(
                                [body for _, _, body in self.requests]
                            )
                            for key in (saved_key, inherited_key):
                                self.assertNotIn(key, serialized)
                                self.assertNotIn(key.encode(), terminal.transcript)
                            if mode == "denied":
                                self.assertEqual(len(requests), before)
                                self.assertFalse(
                                    (self.work / mode / "results.json").exists()
                                )
                            else:
                                self.assertTrue(outputs)
                                self.assertIn(
                                    "Process exited with code 0", outputs[0]["output"]
                                )
                                self.assertEqual(len(requests), before + 1)
                                result = json.loads(
                                    (self.work / mode / "results.json").read_text()
                                )
                                self.assertEqual(result["coverage"]["judged"], 1)
                                self.assertEqual(result["provider"], "typesafe-sdk")
                                self.assertEqual(
                                    requests[-1]["state"]["target_response"], literal
                                )
                                self.assertTrue(
                                    (self.work / mode / "record.json").is_file()
                                )
                                for artifact in (self.work / mode).iterdir():
                                    for key in (saved_key, inherited_key):
                                        self.assertNotIn(key, artifact.read_text())
                            receipts.append(
                                {
                                    "mode": mode,
                                    "passed": True,
                                    "sdk_requests": len(requests) - before,
                                    "sandbox": "workspace-write",
                                }
                            )
                finally:
                    command([*prefix, "clear"])
                    server.shutdown()
                    server.server_close()

    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.TestSuite([AgentJudge()])
    )
    if not result.wasSuccessful():
        raise SystemExit(1)
    print(
        json.dumps(
            {
                "passed": True,
                "actual_agent_tool": True,
                "cases": receipts,
                "model_instruction_following_tested": False,
                "paid_calls": 0,
            }
        )
    )


@unittest.skipUnless(
    os.environ.get("AIRS_TYPESAFE_NATIVE") == "1", "explicit native fixture"
)
class InstalledAgentJudge(unittest.TestCase):
    def test_skill_credential_access_through_command_approval(self):
        binary = str(Path(os.environ["AIRS_HARNESS_BIN"]).resolve(strict=True))
        result = subprocess.run(
            native_test_command(__file__, ["--worker", binary]),
            text=True,
            capture_output=True,
            timeout=300,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertTrue(json.loads(result.stdout.strip().splitlines()[-1])["passed"])


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--worker":
        exercise(sys.argv[2])
    else:
        unittest.main()
