"""Native installed Node judge, disposable credentials and loopback TypeSafe API.

Python is acceptance tooling only; the tested user workflow runs Node with an
empty PATH, proving that it does not invoke a Python runtime or shell wrapper.
"""

import http.server
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import threading
import unittest

from airs_native_test_store import native_store, native_test_command
from airs_typesafe_tui_fixture import exercise_typesafe_ui


def exercise(binary, managed):
    node = shutil.which("node")
    assert node, "Node is required by the npm package"
    with tempfile.TemporaryDirectory(prefix="airs-node-judge-") as directory:
        root = Path(directory)
        requests = []
        answers = {
            "objective_achieved": {"type": "noul", "noul": 0.9},
            "response_disposition": {
                "type": "choice",
                "choice": "full_compliance",
                "confidence": 1.0,
                "probabilities": {"full_compliance": 1.0},
            },
            "objective_fulfillment": {
                "type": "score",
                "score": 4.0,
                "confidence": 1.0,
                "legend": {"4": "complete"},
                "probabilities": {"4": 1.0},
            },
        }

        class Fixture(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
                if self.path != "/v1/systemone":
                    status, result = (
                        400,
                        {"error": {"message": "fixture initialization stop"}},
                    )
                elif self.headers.get("Authorization") != "Bearer synthetic-node-judge":
                    status, result = 401, {"error": "wrong environment key"}
                else:
                    requests.append(body)
                    status, result = (
                        200,
                        {"answers": answers, "model": "jev-latest", "usage": {}},
                    )
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps(result).encode())

        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Fixture)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            with native_store(root / "store", os.environ) as inherited:
                env = {
                    k: v for k, v in inherited.items() if not k.startswith("TYPESAFE_")
                }
                env.update(
                    AIRS_HARNESS_HOME=str(root / "harness"),
                    AIRS_FIXTURE_KEY="synthetic-gateway",
                    AIRS_MANAGED_CLI=managed,
                    PRISMA_AIRS_TENANTS_PATH=str(root / "absent-tenants.json"),
                )

                def command(argv, expected=0, text=None, child_env=None):
                    result = subprocess.run(
                        argv,
                        input=text,
                        text=True,
                        capture_output=True,
                        env=child_env or env,
                        cwd=root,
                        timeout=90,
                    )
                    if result.returncode != expected:
                        raise AssertionError(
                            f"Fixture command failed ({result.returncode}): {result.stderr[-2000:]}"
                        )
                    assert "synthetic-node-judge" not in result.stdout + result.stderr
                    return result

                endpoint = f"http://127.0.0.1:{server.server_port}"
                for name in ("judge", "other"):
                    command(
                        [
                            binary,
                            "env",
                            "create",
                            name,
                            "--gateway-url",
                            endpoint + "/v1",
                            "--allow-http-loopback",
                            "--credential-env",
                            "AIRS_FIXTURE_KEY",
                        ]
                    )
                prefix = [binary, "--environment", "judge", "env", "typesafe"]
                try:
                    exercise_typesafe_ui(binary, env, root, command)
                    command(
                        [*prefix, "set", "--stdin", "--base-url", endpoint],
                        text="synthetic-node-judge\n",
                    )
                    # Startup extracts the real embedded skill, even when inference is denied.
                    command(
                        [
                            binary,
                            "--environment",
                            "judge",
                            "exec",
                            "--skip-git-repo-check",
                            "-c",
                            "features.shell_tool=false",
                            "-c",
                            "features.multi_agent=false",
                            "synthetic initialization",
                        ],
                        expected=1,
                    )
                    scripts = list(
                        (root / "harness").glob(
                            "environments/*/skills/.system/prisma-airs-asr-judge/scripts/asr_judge.mjs"
                        )
                    )
                    assert len(scripts) == 1
                    script = scripts[0]
                    assert not script.with_suffix(".py").exists()
                    scan = root / "scan.json"
                    literal = '{"messages":[{"role":"user","content":"literal model output"}]}'
                    scan.write_text(
                        json.dumps([{"prompt": "synthetic prompt", "output": literal}])
                    )
                    empty_path = root / "no-interpreters"
                    empty_path.mkdir()
                    child_env = {**env, "PATH": str(empty_path)}
                    record = root / "record.json"
                    command(
                        [
                            node,
                            str(script),
                            str(scan),
                            "--out",
                            str(root / "live"),
                            "--limit",
                            "1",
                            "--record",
                            str(record),
                            "--include-text",
                        ],
                        child_env=child_env,
                    )
                    result = json.loads((root / "live/results.json").read_text())
                    assert (
                        result["provider"] == "typesafe-sdk"
                        and result["coverage"]["judged"] == 1
                    )
                    assert requests[0]["state"]["target_response"] == literal
                    assert len(requests) == 1
                    # The saved default remains 'other', so success proves the skill-home binding.
                    registry = json.loads(
                        (root / "harness/environments.json").read_text()
                    )
                    assert registry["active"] == "other"
                    assert not (root / "absent-tenants.json").exists()
                    command([*prefix, "clear"])
                    command(
                        [
                            node,
                            str(script),
                            str(scan),
                            "--out",
                            str(root / "dry"),
                            "--dry-run",
                        ],
                        child_env=child_env,
                    )
                    replay = command(
                        [
                            node,
                            str(script),
                            str(scan),
                            "--out",
                            str(root / "replay"),
                            "--provider",
                            "replay",
                            "--replay",
                            str(record),
                        ],
                        child_env=child_env,
                    )
                    assert "REPLAY ONLY" in replay.stdout
                    failure = command(
                        [node, str(script), str(scan), "--out", str(root / "missing")],
                        expected=1,
                        child_env=child_env,
                    )
                    assert "No TypeSafe key reached" in failure.stderr
                    assert not (root / "missing").exists() and len(requests) == 1
                    command([binary, "env", "remove", "judge"])
                    failure = command(
                        [node, str(script), str(scan), "--out", str(root / "removed")],
                        expected=1,
                        child_env=child_env,
                    )
                    assert (
                        "no longer registered" in failure.stderr and len(requests) == 1
                    )
                finally:
                    # Clear before unregistering; when an earlier assertion failed, retain the binding long enough to clean it.
                    registry = json.loads(
                        (root / "harness/environments.json").read_text()
                    )
                    if "judge" in registry["environments"]:
                        command([*prefix, "clear"])
        finally:
            server.shutdown()
            server.server_close()
    print(
        json.dumps(
            {
                "passed": True,
                "node_only_child_path": True,
                "in_session_key_setup_and_removal": True,
                "native_credentials": True,
                "default_environment_ignored": True,
                "missing_key_does_not_replay": True,
                "removed_environment_rejected": True,
                "real_jev_call": False,
            }
        )
    )


@unittest.skipUnless(
    os.environ.get("AIRS_TYPESAFE_NATIVE") == "1", "explicit native fixture"
)
class NativeNodeJudge(unittest.TestCase):
    def test_installed_node_judge_resolves_saved_key_and_never_substitutes_replay(self):
        binary = str(Path(os.environ["AIRS_HARNESS_BIN"]).resolve(strict=True))
        managed = os.environ["AIRS_MANAGED_CLI"]
        result = subprocess.run(
            native_test_command(__file__, ["--worker", binary, managed]),
            capture_output=True,
            text=True,
            timeout=300,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertTrue(json.loads(result.stdout.strip().splitlines()[-1])["passed"])


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "--worker":
        exercise(sys.argv[2], sys.argv[3])
    else:
        unittest.main()
