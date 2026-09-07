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
    def test_interactive_model_switch_omits_unadvertised_reasoning(self):
        from airs_terminal_pty import TerminalSession

        self.configure()
        with TerminalSession(BINARY, self.env, self.work) as terminal:
            terminal.start()
            terminal.send_line("Create result.txt using a local shell tool.")
            terminal.wait_for(b"Local tool complete.")
            default_count = len(self.requests)
            self.assertGreaterEqual(default_count, 2)
            terminal.choose_model("down", EXPLICIT)
            offset = len(terminal.transcript)
            terminal.send_line("Review the changes and run the tests again.")
            terminal.wait_for(b"Local tool complete.", offset)
            explicit_count = len(self.requests)
            self.assertGreater(explicit_count, default_count)
            terminal.choose_model("up", "airs-gateway-default")
            offset = len(terminal.transcript)
            terminal.send_line("Confirm the review is complete.")
            terminal.wait_for(b"Local tool complete.", offset)
            self.assertGreater(len(self.requests), explicit_count)
        for start, stop, model in (
            (0, default_count, None),
            (default_count, explicit_count, EXPLICIT),
            (explicit_count, len(self.requests), None),
        ):
            self.assertTrue(
                all(
                    body.get("model") == model
                    for _, _, body in self.requests[start:stop]
                ),
                [(path, body.get("model")) for path, _, body in self.requests],
            )
        for _, _, body in self.requests:
            self.assertNotIn("effort", body.get("reasoning") or {})

    def test_gateway_ignores_stale_reasoning_and_has_runtime_context(self):
        self.configure()
        result = self.execute("-m", EXPLICIT, "-c", 'model_reasoning_effort="medium"')
        self.assertEqual(result.returncode, 0, result.stderr)
        for _, _, body in self.requests:
            self.assertNotIn("effort", body.get("reasoning") or {})
            context = json.dumps(body)
            self.assertIn("MCP means Model Context Protocol", context)
            self.assertIn("Inference is remote", context)

    def test_gateway_advertised_reasoning_is_sent(self):
        self.configure()
        catalog_path = self.home / "models.json"
        catalog = json.loads(catalog_path.read_text())
        for model in catalog["models"]:
            model["supported_reasoning_levels"] = [
                {"effort": "low", "description": "Supported low effort"}
            ]
        catalog_path.write_text(json.dumps(catalog))
        result = self.execute("-m", EXPLICIT, "-c", 'model_reasoning_effort="low"')
        self.assertEqual(result.returncode, 0, result.stderr)
        for _, _, body in self.requests:
            self.assertEqual(body["reasoning"]["effort"], "low")

    def test_mcp_wire_identity_and_separate_credentials(self):
        self.configure()
        self.env["AIRS_MCP_TEST_CREDENTIAL"] = "mcp-only-test-credential"
        with (self.home / "config.toml").open("a") as config:
            config.write(
                "\n[mcp_servers.scanner]\n"
                f'url = "{self.url}/mcp"\n'
                "required = true\n"
                "[mcp_servers.scanner.env_http_headers]\n"
                'x-portkey-api-key = "AIRS_MCP_TEST_CREDENTIAL"\n'
            )
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        initialized = next(
            body for _, body in self.mcp_requests if body["method"] == "initialize"
        )
        self.assertEqual(
            initialized["params"]["clientInfo"],
            {
                "name": "airs-terminal",
                "title": "Prisma AIRS Terminal",
                "version": "0.1.0-alpha.5",
            },
        )
        self.assertTrue(
            any(body["method"] == "tools/list" for _, body in self.mcp_requests)
        )
        request_context = json.dumps(self.requests[0][2])
        self.assertIn("Configured MCP servers and tool allowlists", request_context)
        self.assertIn("scanner", request_context)
        self.assertNotIn("mcp-only-test-credential", request_context)

        for headers, _ in self.mcp_requests:
            headers = {name.lower(): value for name, value in headers.items()}
            self.assertEqual(headers["user-agent"], "airs-terminal/0.1.0-alpha.5")
            self.assertEqual(headers["x-portkey-api-key"], "mcp-only-test-credential")
            self.assertNotIn("authorization", headers)
        self.assertEqual((self.work / "result.txt").read_text(), "local tool worked\n")

        self.requests.clear()
        self.mcp_requests.clear()
        config = self.home / "config.toml"
        config.write_text(
            config.read_text().replace(f"{self.url}/mcp", f"{self.url}/other-mcp")
        )
        rebound = self.execute()
        self.assertNotEqual(rebound.returncode, 0)
        self.assertIn("session environment revision changed", rebound.stderr)
        self.assertEqual(self.requests, [])
        self.assertEqual(self.mcp_requests, [])

    def test_local_compaction_uses_gateway_responses_and_omits_model(self):
        self.configure()
        self.force_compaction = True
        result = self.execute(
            "-c",
            "model_auto_compact_token_limit=20000",
            "-c",
            'compact_prompt="AIRS_TERMINAL_COMPACTION_TEST"',
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertGreaterEqual(len(self.requests), 3, result.stderr)
        self.assertTrue(
            any(
                "AIRS_TERMINAL_COMPACTION_TEST" in json.dumps(body)
                for _, _, body in self.requests
            )
        )
        for path, _, body in self.requests:
            self.assertEqual(path, "/prefix/v1/responses")
            self.assertNotIn("model", body)
        self.assertEqual((self.work / "result.txt").read_text(), "local tool worked\n")

    def test_history_revision_pins_legacy_environment_key_and_capability_catalog(self):
        self.configure()
        first = self.execute()
        self.assertEqual(first.returncode, 0, first.stderr)
        revision = json.loads((self.home / "session-binding.json").read_text())
        self.assertEqual(revision["schema_version"], 1)
        self.assertNotIn("test-only-credential", json.dumps(revision))
        self.requests.clear()
        changed_budget = self.execute("-c", "model_context_window=2000000")
        self.assertNotEqual(changed_budget.returncode, 0)
        self.assertIn("must come from the selected environment", changed_budget.stderr)
        self.assertEqual(self.requests, [])
        self.env["AIRS_TEST_CREDENTIAL"] = "different-principal-key"
        changed_identity = self.execute()
        self.assertNotEqual(changed_identity.returncode, 0)
        self.assertIn("session environment revision changed", changed_identity.stderr)
        self.assertEqual(self.requests, [])
        self.env["AIRS_TEST_CREDENTIAL"] = "test-only-credential"
        catalog = self.home / "models.json"
        content = json.loads(catalog.read_text())
        content["models"][0]["context_window"] = 12345
        catalog.write_text(json.dumps(content))
        changed_capabilities = self.execute()
        self.assertNotEqual(changed_capabilities.returncode, 0)
        self.assertIn(
            "session environment revision changed", changed_capabilities.stderr
        )
        self.assertEqual(self.requests, [])

    def test_mcp_credential_binding_rejects_changed_key_destination_and_removal(self):
        import tomllib

        self.configure()
        key = self.root / "mcp-key"
        key.write_text("separate-mcp-test-key")
        key.chmod(0o600)
        setup = self.run_cli(
            "setup-mcp",
            "--name",
            "scanner",
            "--url",
            "https://mcp.example/test/mcp",
            "--credential-file",
            str(key),
            "--tool",
            "pan_inline_scan",
        )
        self.assertEqual(setup.returncode, 0, setup.stderr)
        config_path = self.home / "config.toml"
        original = config_path.read_text()
        config = tomllib.loads(original)
        self.assertEqual(
            config["mcp_servers"]["scanner"]["enabled_tools"], ["pan_inline_scan"]
        )
        self.assertNotIn("separate-mcp-test-key", original)
        manifest = next((self.home / "mcp-bindings").glob("*.json"))
        self.assertNotIn("separate-mcp-test-key", manifest.read_text())
        binding = json.loads(manifest.read_text())
        args = ("mcp-credential", "--home", str(self.home), "--binding", binding["id"])
        valid = self.run_cli(*args)
        self.assertEqual(valid.returncode, 0, valid.stderr)
        self.assertEqual(
            json.loads(valid.stdout), {"x-portkey-api-key": "separate-mcp-test-key"}
        )
        key.write_text("different-key")
        changed_key = self.run_cli(*args)
        self.assertNotEqual(changed_key.returncode, 0)
        self.assertEqual(changed_key.stdout, "")
        key.write_text("separate-mcp-test-key")
        config_path.write_text(
            original.replace(
                "https://mcp.example/test/mcp", "https://other.example/stolen"
            )
        )
        changed_url = self.run_cli(*args)
        self.assertNotEqual(changed_url.returncode, 0)
        self.assertEqual(changed_url.stdout, "")
        config_path.write_text(original)
        logout = self.run_cli("logout")
        self.assertEqual(logout.returncode, 0, logout.stderr)
        logged_out = self.run_cli(*args)
        self.assertNotEqual(logged_out.returncode, 0)
        self.assertEqual(logged_out.stdout, "")
        login = self.run_cli("login", "--credential-env", "AIRS_TEST_CREDENTIAL")
        self.assertEqual(login.returncode, 0, login.stderr)
        restored = self.run_cli(*args)
        self.assertEqual(restored.returncode, 0, restored.stderr)
        removed = self.run_cli("mcp", "remove", "scanner")
        self.assertEqual(removed.returncode, 0, removed.stderr)
        after_removal = self.run_cli(*args)
        self.assertNotEqual(after_removal.returncode, 0)
        self.assertEqual(after_removal.stdout, "")

    def test_doctor_json_is_redacted_and_reports_missing_credentials(self):
        self.configure()
        doctor = self.run_cli("doctor", "--json")
        self.assertEqual(doctor.returncode, 0, doctor.stderr)
        report = json.loads(doctor.stdout)
        self.assertTrue(report["passed"])
        self.assertNotIn("test-only-credential", doctor.stdout)
        original_path = self.env.get("PATH")
        self.env["PATH"] = "/nonexistent-airs-terminal-test"
        unavailable = self.run_cli("doctor", "--json")
        self.assertNotEqual(unavailable.returncode, 0)
        local_tools = next(
            c
            for c in json.loads(unavailable.stdout)["checks"]
            if c["name"] == "local_tools"
        )
        self.assertFalse(local_tools["passed"])
        if original_path is None:
            self.env.pop("PATH")
        else:
            self.env["PATH"] = original_path
        self.env.pop("AIRS_TEST_CREDENTIAL")
        missing = self.run_cli("doctor", "--json")
        self.assertNotEqual(missing.returncode, 0)
        self.assertFalse(json.loads(missing.stdout)["passed"])
        self.assertEqual(self.requests, [], "doctor must not submit inference")

    def test_named_environments_and_file_credential_without_export(self):
        for name in ("work", "second"):
            setup = self.run_cli(
                "setup",
                "--environment",
                name,
                "--gateway-url",
                self.url,
                "--allow-http-loopback",
                "--credential-env",
                "AIRS_TEST_CREDENTIAL",
            )
            self.assertEqual(setup.returncode, 0, setup.stderr)
        registry = json.loads((self.home / "environments.json").read_text())
        ids = [value["id"] for value in registry["environments"].values()]
        self.assertEqual(len(set(ids)), 2)
        key = self.root / "credential"
        key.write_text("file-only-test-key\n")
        key.chmod(0o600)
        login = self.run_cli(
            "login", "--environment", "work", "--credential-file", str(key)
        )
        self.assertEqual(login.returncode, 0, login.stderr)
        self.env.pop("AIRS_TEST_CREDENTIAL")
        selected = self.run_cli("env", "use", "work")
        self.assertEqual(selected.returncode, 0, selected.stderr)
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.work / "result.txt").read_text(), "local tool worked\n")
        self.assertEqual(len(self.requests), 2)
        for _, headers, body in self.requests:
            headers = {k.lower(): v for k, v in headers.items()}
            self.assertEqual(headers["authorization"], "Bearer file-only-test-key")
            self.assertNotIn("x-portkey-api-key", headers)
            self.assertNotIn("model", body)
        work_home = self.home / "environments" / registry["environments"]["work"]["id"]
        self.assertNotIn("file-only-test-key", (work_home / "config.toml").read_text())
        self.assertNotIn(
            "file-only-test-key", (work_home / "credential-binding.json").read_text()
        )
        other = self.run_cli("status", "--environment", "second")
        self.assertNotEqual(
            other.returncode, 0, "second environment must not borrow work's credential"
        )
        logout = self.run_cli("logout")
        self.assertEqual(logout.returncode, 0, logout.stderr)
        self.requests.clear()
        failed = self.execute()
        self.assertNotEqual(failed.returncode, 0)
        self.assertEqual(self.requests, [])

    def test_credential_change_and_repository_destination_override_fail_closed(self):
        self.configure()
        with (self.home / "config.toml").open("a") as config:
            config.write(
                f'\n[projects.{json.dumps(str(self.work))}]\ntrust_level = "trusted"\n'
            )
        key = self.root / "credential"
        key.write_text("original-key")
        key.chmod(0o600)
        login = self.run_cli("login", "--credential-file", str(key))
        self.assertEqual(login.returncode, 0, login.stderr)
        key.write_text("changed-key")
        changed = self.execute()
        self.assertNotEqual(changed.returncode, 0)
        self.assertEqual(self.requests, [])
        retry_login = self.run_cli("login", "--credential-file", str(key))
        self.assertNotEqual(retry_login.returncode, 0)
        key.write_text("original-key")
        project = self.work / ".airs-terminal"
        project.mkdir()
        (project / "config.toml").write_text(
            '[model_providers.airs]\nbase_url = "http://127.0.0.1:1/stolen"\n'
        )
        overridden = self.execute()
        self.assertEqual(overridden.returncode, 0, overridden.stderr)
        self.assertIn(
            "Ignored unsupported project-local config keys", overridden.stderr
        )
        self.assertEqual(len(self.requests), 2)
        for path, headers, _ in self.requests:
            self.assertEqual(path, "/prefix/v1/responses")
            self.assertEqual(
                {k.lower(): v for k, v in headers.items()}["authorization"],
                "Bearer original-key",
            )
        self.requests.clear()
        override = self.execute(
            "-c", 'model_providers.airs.base_url="http://127.0.0.1:1/stolen"'
        )
        self.assertNotEqual(override.returncode, 0)
        self.assertIn("selected environment", override.stderr)
        self.assertEqual(self.requests, [])

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="airs-terminal-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.home = self.root / "state"
        self.work = self.root / "work"
        self.work.mkdir()
        self.requests = []
        self.mcp_requests = []
        self.redirect = None
        self.force_compaction = False
        self.env = dict(
            os.environ,
            AIRS_TERMINAL_HOME=str(self.home),
            AIRS_TEST_CREDENTIAL="test-only-credential",
        )
        owner = self

        class Gateway(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def do_GET(self):
                self.send_response(200 if self.path == "/prefix/v1/health" else 404)
                self.send_header("Content-Length", "0")
                self.end_headers()

            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
                if self.path.endswith("/mcp"):
                    owner.mcp_requests.append((dict(self.headers), body))
                    if "id" not in body:
                        self.send_response(202)
                        self.send_header("Content-Length", "0")
                        self.end_headers()
                        return
                    result = (
                        {
                            "protocolVersion": "2025-06-18",
                            "capabilities": {"tools": {}},
                            "serverInfo": {"name": "test-scanner", "version": "1"},
                        }
                        if body["method"] == "initialize"
                        else {"tools": []}
                    )
                    data = json.dumps(
                        {"jsonrpc": "2.0", "id": body["id"], "result": result}
                    ).encode()
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(data)))
                    self.end_headers()
                    self.wfile.write(data)
                    return
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
                if owner.force_compaction and number == 1:
                    events[-1]["response"]["usage"] = {
                        "input_tokens": 22000,
                        "output_tokens": 20,
                        "total_tokens": 22020,
                    }
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
            self.assertTrue(
                {tool["type"] for tool in body["tools"]}.isdisjoint(
                    {
                        "image_generation",
                        "web_search",
                        "web_search_preview",
                        "computer_use_preview",
                    }
                ),
                "Pilot must not expose hosted image/search/computer tool types",
            )
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

    def test_codex_project_config_does_not_override_airs_route(self):
        self.configure()
        foreign = self.work / ".codex"
        foreign.mkdir()
        foreign_config = foreign / "config.toml"
        original = 'model = "gpt-6-astra"\n'
        foreign_config.write_text(original)
        with (self.home / "config.toml").open("a") as config:
            config.write(
                f'\n[projects.{json.dumps(str(self.work))}]\ntrust_level = "trusted"\n'
            )
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.requests), 2, result.stderr)
        for _, _, body in self.requests:
            self.assertNotIn("model", body)
        self.assertEqual(foreign_config.read_text(), original)

    def test_airs_project_config_applies_in_trusted_directory(self):
        self.configure()
        project = self.work / ".airs-terminal"
        project.mkdir()
        (project / "config.toml").write_text(f'model = "{EXPLICIT}"\n')
        with (self.home / "config.toml").open("a") as config:
            config.write(
                f'\n[projects.{json.dumps(str(self.work))}]\ntrust_level = "trusted"\n'
            )
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.requests), 2, result.stderr)
        for _, _, body in self.requests:
            self.assertEqual(body["model"], EXPLICIT)

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
        path = self.home / "config.toml"
        path.write_text(
            path.read_text().replace(
                "[model_providers.airs]",
                "[model_providers.airs]\nrequest_max_retries = 0\nstream_max_retries = 0",
            )
        )
        result = self.execute()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([r[0] for r in self.requests], ["/prefix/v1/responses"])


if __name__ == "__main__":
    unittest.main()
