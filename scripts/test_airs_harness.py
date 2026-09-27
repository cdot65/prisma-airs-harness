#!/usr/bin/env python3
"""Exercise the built standalone agent against a deterministic Responses gateway.

Run: python3 -m unittest discover -s scripts -p test_airs_harness.py -v
AIRS_HARNESS_BIN optionally selects an installed executable. No inference key,
PAH service, network beyond loopback, or Python third-party package is required.
"""

import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import threading
import tomllib
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

BINARY = Path(
    os.environ.get("AIRS_HARNESS_BIN", "codex-rs/target/debug/airs-harness")
).resolve()
EXPLICIT = "@test/org/model:version"


def latest_user_text(body):
    for item in reversed(body.get("input", [])):
        if item.get("role") == "user":
            return "".join(part.get("text", "") for part in item.get("content", []))
    return ""


class TerminalIntegration(unittest.TestCase):
    def test_inline_question_preserves_draft_and_gateway_route(self):
        from airs_harness_pty import TerminalSession

        self.configure()
        catalog_path = Path(
            tomllib.loads((self.home / "config.toml").read_text())["model_catalog_json"]
        )
        catalog = json.loads(catalog_path.read_text())
        for model in catalog["models"]:
            model["experimental_supported_tools"] = ["request_user_input_async"]
        catalog_path.write_text(json.dumps(catalog))
        self.question_turn_release = threading.Event()
        self.addCleanup(self.question_turn_release.set)
        self.initial_function_call = (
            "request_user_input_async",
            {
                "questions": [
                    {"title": "Choose the fixture color", "options": ["Blue", "Green"]}
                ]
            },
        )
        prompt = "Ask me to choose the fixture color."
        draft = "Preserve this independent draft."
        self.phase_replies = {prompt: "Question queued.", draft: "Draft preserved."}
        with TerminalSession(BINARY, self.env, self.work) as terminal:
            self.terminal_transcript = terminal.transcript
            terminal.start()
            terminal.send_line(prompt)
            terminal.wait_for(b"Question queued.")
            os.write(terminal.master, b"\x1b[200~" + draft.encode() + b"\x1b[201~")
            terminal.wait_for(draft.encode())
            offset = len(terminal.transcript)
            os.write(terminal.master, b"\x1b[1;3A")
            terminal.wait_for(b"enter submit", offset)
            offset = len(terminal.transcript)
            os.write(terminal.master, b"\r")
            terminal.wait_for(draft.encode(), offset)
            self.question_turn_release.set()
            terminal.wait_for(b"Create result file", offset)
            os.write(terminal.master, b"\r")
            terminal.wait_for(b"Draft preserved.", offset)
        self.assertTrue(
            any(latest_user_text(body) == draft for _, _, body in self.requests)
        )
        self.assertTrue(
            any("Blue" in json.dumps(body["input"]) for _, _, body in self.requests[1:])
        )
        for path, _, body in self.requests:
            self.assertEqual(path, "/prefix/v1/responses")
            self.assertNotIn("model", body)

    def test_worktree_preserves_gateway_binding_and_isolates_local_edits(self):
        from airs_harness_pty import TerminalSession

        self.configure()
        for args in (
            ["init", "-q"],
            [
                "-c",
                "user.name=AIRS fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "Fixture baseline",
            ],
        ):
            subprocess.run(
                ["git", *args], cwd=self.work, check=True, capture_output=True
            )
        protected = tomllib.loads((self.home / "config.toml").read_text())
        with TerminalSession(
            BINARY,
            self.env,
            self.work,
            arguments=[
                "--no-alt-screen",
                "--enable",
                "worktrees",
                "--worktree",
                "-s",
                "workspace-write",
            ],
        ) as terminal:
            self.terminal_transcript = terminal.transcript
            terminal.start()
            terminal.send_line("Create result.txt using a local shell tool.")
            terminal.wait_for(b"Local tool complete.")
        listing = subprocess.check_output(
            ["git", "worktree", "list", "--porcelain"],
            cwd=self.work,
            text=True,
        )
        paths = [
            Path(line.removeprefix("worktree "))
            for line in listing.splitlines()
            if line.startswith("worktree ")
        ]
        managed = [path for path in paths if path != self.work]
        self.assertEqual(len(managed), 1, listing)
        self.assertEqual((managed[0] / "result.txt").read_text(), "local tool worked\n")
        self.assertFalse((self.work / "result.txt").exists())
        current = tomllib.loads((self.home / "config.toml").read_text())
        trust = current.pop("projects")
        self.assertEqual(trust, {str(self.work): {"trust_level": "trusted"}})
        self.assertEqual(current, protected)
        histories = [
            json.loads(line)["payload"]
            for path in (self.home / "sessions").rglob("*.jsonl")
            for line in path.read_text().splitlines()
            if json.loads(line).get("type") == "session_meta"
        ]
        parent = next(row for row in histories if row.get("cwd") == str(managed[0]))
        resumed = self.run_cli(
            "exec",
            "resume",
            "--skip-git-repo-check",
            parent["id"],
            "Review the worktree result again.",
        )
        self.assertEqual(resumed.returncode, 0, resumed.stderr)
        self.assertIn(str(managed[0]), json.dumps(self.requests[-1][2]["input"]))
        forked = self.run_cli(
            "--enable",
            "worktrees",
            "exec",
            "fork",
            "--worktree",
            "--skip-git-repo-check",
            parent["id"],
            "Review this isolated fork.",
        )
        self.assertEqual(forked.returncode, 0, forked.stderr)
        final_listing = subprocess.check_output(
            ["git", "worktree", "list", "--porcelain"],
            cwd=self.work,
            text=True,
        )
        self.assertEqual(
            sum(line.startswith("worktree ") for line in final_listing.splitlines()), 3
        )
        self.assertGreaterEqual(len(self.requests), 2)
        for path, headers, body in self.requests:
            self.assertEqual(path, "/prefix/v1/responses")
            self.assertNotIn("model", body)
            normalized = {key.lower(): value for key, value in headers.items()}
            self.assertEqual(normalized["x-portkey-api-key"], "test-only-credential")

    def test_saved_environment_cannot_enable_unvalidated_upstream_services(self):
        self.configure()
        blocked = [
            "apps",
            "plugins",
            "recommended_plugins",
            "image_generation",
            "remote_control",
            "code_mode",
            "code_mode_only",
            "code_mode_prewarm",
            "code_mode_host",
            "realtime_conversation",
            "guardianv2",
        ]
        # Omit setup's feature defaults to represent an older environment, then
        # request each unsupported capability through the public CLI overrides.
        config_path = self.home / "config.toml"
        original = config_path.read_text()
        config_path.write_text(
            re.sub(r"(?ms)^\[features\]\n.*?(?=^\[|\Z)", "", original)
        )
        args = []
        for feature in blocked:
            args.extend(["-c", f"features.{feature}=true"])
        result = self.run_cli(*args, "features", "list")
        self.assertEqual(result.returncode, 0, result.stderr)
        states = {
            line.split()[0]: line.split()[-1] for line in result.stdout.splitlines()
        }
        self.assertEqual(
            {feature: states[feature] for feature in blocked},
            {feature: "false" for feature in blocked},
        )
        self.assertFalse(self.requests)

    @classmethod
    def setUpClass(cls):
        result = subprocess.run(
            [str(BINARY), "--version"],
            capture_output=True,
            text=True,
            timeout=10,
            check=True,
        )
        product, cls.version = result.stdout.strip().split()
        if product not in ("airs", "airs-harness"):
            raise ValueError("Expected a Prisma AIRS Harness executable")

    @unittest.skipIf(sys.platform == "win32", "POSIX subprocess lock/PTY acceptance")
    def test_running_client_stays_revoked_after_subprocess_logout_and_relogin(self):
        import fcntl
        from airs_harness_pty import TerminalSession

        self.configure()
        login = self.run_cli("login", "--credential-env", "AIRS_TEST_CREDENTIAL")
        self.assertEqual(login.returncode, 0, login.stderr)
        self.phase_replies = {"Create result.txt. Warm this client.": "Client warmed."}
        with TerminalSession(BINARY, self.env, self.work) as terminal:
            self.terminal_transcript = terminal.transcript
            terminal.start()
            terminal.send_line("Create result.txt. Warm this client.")
            terminal.wait_for(b"Client warmed.")
            self.assertTrue(self.requests)
            self.assertEqual(
                (self.work / "result.txt").read_text(), "local tool worked\n"
            )
            # Simulate another process holding a long refresh/configuration lock.
            # Logout must invalidate the warm client before it can acquire this lock.
            with (self.home / ".configuration.lock").open("r+") as lock:
                fcntl.flock(lock, fcntl.LOCK_EX)
                logout = subprocess.Popen(
                    [str(BINARY), "logout"],
                    cwd=self.work,
                    env=self.env,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    text=True,
                )
                try:
                    terminal.wait_until(
                        lambda: (
                            (self.home / "auth-generation")
                            .read_text()
                            .startswith("v1 revoked ")
                        ),
                        timeout=5,
                    )
                    self.assertIsNone(
                        logout.poll(), "cleanup should still wait for the lock"
                    )
                    count = len(self.requests)
                    offset = len(terminal.transcript)
                    terminal.send_line("Try another request after sign-out.")
                    # The trusted helper can reject the revoked epoch before
                    # transport preparation reaches its own generation guard.
                    terminal.wait_until(
                        lambda: any(
                            message in terminal.transcript[offset:]
                            for message in (
                                b"authentication changed",
                                b"credential helper could not supply the bound credential",
                            )
                        ),
                        timeout=5,
                    )
                    self.assertEqual(len(self.requests), count)
                finally:
                    fcntl.flock(lock, fcntl.LOCK_UN)
                    stdout, stderr = logout.communicate(timeout=10)
                self.assertEqual(logout.returncode, 0, stdout + stderr)
            login = self.run_cli("login", "--credential-env", "AIRS_TEST_CREDENTIAL")
            self.assertEqual(login.returncode, 0, login.stderr)
            offset = len(terminal.transcript)
            terminal.send_line("Try again after another process signed in.")
            terminal.wait_for(b"authentication changed", offset, timeout=5)
            self.assertEqual(len(self.requests), count)
            self.assertEqual(
                (self.work / "result.txt").read_text(), "local tool worked\n"
            )
        # A fresh process may use the newly activated epoch and the same identity.
        fresh = self.execute()
        self.assertEqual(fresh.returncode, 0, fresh.stderr)
        self.assertGreater(len(self.requests), count)

    def test_invalid_binding_errors_do_not_echo_record_contents(self):
        self.configure()
        canary = "private-invalid-binding-value"
        (self.home / "credential-binding.json").write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "id": "0e7e417a-d7b6-4dc7-8ae3-72d5c2f3a096",
                    "gateway_url": self.url,
                    "credential_fingerprint": "0" * 64,
                    "source": {"kind": canary},
                }
            )
        )
        for arguments in (
            ["env", "status"],
            ["doctor", "--json"],
            ["login", "--credential-env", "AIRS_TEST_CREDENTIAL"],
        ):
            with self.subTest(arguments=arguments):
                result = self.run_cli(*arguments)
                self.assertNotEqual(result.returncode, 0)
                self.assertNotIn(canary, result.stdout + result.stderr)
        self.assertEqual(self.requests, [])

    @unittest.skipIf(
        sys.platform == "win32", "POSIX PTY; Windows uses native console acceptance"
    )
    def test_plain_terminal_setup_retains_public_settings_after_cancelled_login(self):
        from airs_harness_pty import TerminalSession

        self.env.pop("AIRS_API_KEY", None)
        with TerminalSession(
            BINARY,
            self.env,
            self.work,
            arguments=["env", "create", "--allow-http-loopback"],
            terminal_type="dumb",
        ) as terminal:
            terminal.wait_for(b"Environment name [work]")
            os.write(terminal.master, b"review\r")
            terminal.wait_for(b"AI Gateway URL:")
            os.write(terminal.master, self.url.encode() + b"\r")
            terminal.wait_for(b"Choose 1 or 2")
            os.write(terminal.master, b"2\r")
            terminal.wait_for(b"Workspace API key (input hidden")
            os.write(terminal.master, b"\x1b")
            terminal.wait_for(b"Sign-in cancelled")
            terminal.process.wait(timeout=5)
        registry = json.loads((self.home / "environments.json").read_text())
        self.assertEqual(registry["active"], "review")
        selected = self.home / "environments" / registry["environments"]["review"]["id"]
        config = tomllib.loads((selected / "config.toml").read_text())
        self.assertEqual(config["model_context_window"], 1_000_000)
        self.assertFalse((selected / "credential-binding.json").exists())
        for arguments in ([], ["resume"], ["fork"]):
            with self.subTest(arguments=arguments):
                with TerminalSession(
                    BINARY,
                    self.env,
                    self.work,
                    arguments=arguments,
                    terminal_type="dumb",
                ) as terminal:
                    terminal.wait_for(b"Choose 1 or 2")
                    self.assertNotIn(b"AI Gateway URL:", terminal.transcript)
                    os.write(terminal.master, b"\x03")
                    terminal.process.wait(timeout=5)
        self.assertEqual(self.requests, [])

    def test_existing_home_is_reused_without_moving_credentials_or_history(self):
        self.env["HOME"] = str(self.root)
        self.env.pop("AIRS_TERMINAL_HOME", None)
        self.home = self.root / ".airs-terminal"
        self.env["AIRS_HARNESS_HOME"] = str(self.home)
        self.configure()
        sentinel = self.home / "history.jsonl"
        sentinel.write_text('{"legacy_history":true}\n')
        original = tomllib.loads((self.home / "config.toml").read_text())
        del self.env["AIRS_HARNESS_HOME"]
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        actual = tomllib.loads((self.home / "config.toml").read_text())
        actual.pop("projects", None)  # Runtime can persist the ordinary trust decision.
        self.assertEqual(actual, original)
        self.assertEqual(sentinel.read_text(), '{"legacy_history":true}\n')
        self.assertFalse((self.root / ".airs-harness").exists())

    def test_legacy_home_override_still_selects_existing_state(self):
        self.configure()
        self.env["AIRS_TERMINAL_HOME"] = self.env.pop("AIRS_HARNESS_HOME")
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_new_home_override_takes_precedence_over_legacy(self):
        self.configure()
        self.env["AIRS_TERMINAL_HOME"] = str(self.root / "unselected")
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse((self.root / "unselected").exists())

    def test_saved_legacy_user_agent_uses_current_wire_branding(self):
        self.configure()
        config = self.home / "config.toml"
        before = config.read_text()
        self.assertIn(f"airs-harness/{self.version}", before)
        config.write_text(
            before.replace(
                f"airs-harness/{self.version}", "airs-terminal/0.1.0-alpha.7"
            )
        )
        self.assertNotEqual(config.read_text(), before)
        original = tomllib.loads(config.read_text())
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(self.requests)
        for _, headers, _ in self.requests:
            headers = {key.lower(): value for key, value in headers.items()}
            self.assertEqual(headers["user-agent"], f"airs-harness/{self.version}")
        actual = tomllib.loads(config.read_text())
        actual.pop("projects", None)
        self.assertEqual(actual, original)

    def test_saved_legacy_catalog_uses_new_identity_without_rewriting_history(self):
        self.configure()
        catalog = self.home / "models.json"
        catalog.write_text(
            catalog.read_text().replace("Prisma AIRS Harness", "Prisma AIRS Terminal")
        )
        original = catalog.read_bytes()
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(self.requests)
        for _, _, body in self.requests:
            self.assertIn(
                "You are Prisma AIRS Harness, a local coding assistant.",
                body["instructions"],
            )
            self.assertNotIn("You are Prisma AIRS Terminal", body["instructions"])
        self.assertEqual(catalog.read_bytes(), original)

    @unittest.skipUnless(sys.platform.startswith("linux"), "Linux session-bus recovery")
    def test_oidc_login_without_keyring_explains_recovery_before_network(self):
        self.configure()
        self.env["DBUS_SESSION_BUS_ADDRESS"] = "unix:path=" + str(
            self.work / "missing-bus"
        )
        result = self.run_cli(
            "login",
            "--issuer-url",
            "https://idp.invalid/realms/test",
            "--oidc-client-id",
            "terminal",
            "--audience",
            "inference",
            "--device-auth",
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "Credential storage failed during check credential storage", result.stderr
        )
        self.assertIn("Linux credential service", result.stderr)
        self.assertIn("category:", result.stderr)
        self.assertIn("OS status:", result.stderr)
        self.assertEqual(self.requests, [])
        # A missing session bus establishes unavailability, not a locked store.
        self.assertNotIn("Unlock your native credential store", result.stderr)
        self.assertIn("airs doctor", result.stderr)
        self.assertIn("No plaintext credential was written", result.stderr)

    @unittest.skipIf(
        sys.platform == "win32", "POSIX PTY; Windows uses native console acceptance"
    )
    def test_workspace_login_hides_paste_and_restores_terminal_on_cancel(self):
        import termios
        from airs_harness_pty import TerminalSession

        self.configure()
        for arguments in (["login"], ["login", "--with-api-key"]):
            with self.subTest(arguments=arguments):
                with TerminalSession(
                    BINARY, self.env, self.work, arguments=arguments
                ) as terminal:
                    if arguments == ["login"]:
                        terminal.wait_for(b"Sign in to continue")
                        os.write(terminal.master, b"2")
                    terminal.wait_for(b"Workspace API key (input hidden")
                    self.assertFalse(
                        termios.tcgetattr(terminal.master)[3] & termios.ECHO
                    )
                    secret = b"private-test-key-never-echo"
                    os.write(terminal.master, b"\x1b[200~" + secret + b"\x1b[201~")
                    os.write(terminal.master, b"\x03")
                    if arguments == ["login"]:
                        terminal.wait_for(b"Sign-in needs your attention")
                        os.write(terminal.master, b"\x1b")
                    terminal.wait_for(b"Sign-in cancelled")
                    terminal.process.wait(timeout=5)
                    self.assertNotIn(secret, terminal.transcript)
                    self.assertTrue(
                        termios.tcgetattr(terminal.master)[3] & termios.ECHO
                    )
                    self.assertTrue(
                        termios.tcgetattr(terminal.master)[3] & termios.ICANON
                    )
                self.assertFalse((self.home / "credential-binding.json").exists())
        self.assertEqual(self.requests, [])

    @unittest.skipIf(
        sys.platform == "win32", "POSIX PTY; Windows uses native console acceptance"
    )
    def test_workspace_login_rejects_multiline_paste_without_echo_or_binding(self):
        from airs_harness_pty import TerminalSession

        self.configure()
        with TerminalSession(
            BINARY, self.env, self.work, arguments=["login", "--with-api-key"]
        ) as terminal:
            terminal.wait_for(b"Workspace API key (input hidden")
            os.write(
                terminal.master, b"\x1b[200~private-line-one\nprivate-line-two\x1b[201~"
            )
            terminal.wait_for(b"Paste a single workspace API key")
            terminal.process.wait(timeout=5)
            self.assertNotIn(b"private-line-one", terminal.transcript)
            self.assertNotIn(b"private-line-two", terminal.transcript)
        self.assertFalse((self.home / "credential-binding.json").exists())
        self.assertEqual(self.requests, [])

    @unittest.skipIf(
        sys.platform == "win32", "POSIX signals; Windows uses native console acceptance"
    )
    def test_workspace_login_restores_terminal_when_terminated(self):
        import signal
        import termios
        from airs_harness_pty import TerminalSession

        self.configure()
        with TerminalSession(
            BINARY, self.env, self.work, arguments=["login", "--with-api-key"]
        ) as terminal:
            terminal.wait_for(b"Workspace API key (input hidden")
            secret = b"private-key-at-interruption"
            os.write(terminal.master, b"\x1b[200~" + secret + b"\x1b[201~")
            terminal.process.send_signal(signal.SIGTERM)
            terminal.wait_until(lambda: terminal.process.poll() is not None)
            self.assertNotIn(secret, terminal.transcript)
            mode = termios.tcgetattr(terminal.master)[3]
            self.assertTrue(mode & termios.ECHO)
            self.assertTrue(mode & termios.ICANON)
        self.assertFalse((self.home / "credential-binding.json").exists())
        self.assertEqual(self.requests, [])

    def test_interactive_model_switch_omits_unadvertised_reasoning(self):
        from airs_harness_pty import TerminalSession

        self.configure()
        config = self.home / "config.toml"
        before = config.read_text()
        self.assertIn(f"airs-harness/{self.version}", before)
        config.write_text(
            before.replace(
                f"airs-harness/{self.version}", "airs-terminal/0.1.0-alpha.7"
            )
        )
        self.assertNotEqual(config.read_text(), before)
        prompts = {
            "Create result.txt using a local shell tool. Phase one.": (
                None,
                "First phase finished.",
            ),
            "Review the changes and run the tests again. Phase two.": (
                EXPLICIT,
                "Second phase finished.",
            ),
            "Confirm the review is complete. Phase three.": (
                None,
                "Third phase finished.",
            ),
        }
        self.phase_replies = {prompt: reply for prompt, (_, reply) in prompts.items()}
        with TerminalSession(BINARY, self.env, self.work) as terminal:
            self.terminal_transcript = terminal.transcript
            terminal.start()
            for index, (prompt, (model, reply)) in enumerate(prompts.items()):
                if index:
                    terminal.choose_model(
                        "down" if model else "up", model or "airs-gateway-default"
                    )
                offset = len(terminal.transcript)
                terminal.send_line(prompt)
                terminal.wait_for(reply.encode(), offset)
        counts = dict.fromkeys(prompts, 0)
        for _, _, body in self.requests:
            if body.get("stream") is False:
                self.assertEqual(body["max_output_tokens"], 16)
                continue
            prompt = latest_user_text(body)
            if prompt in prompts:
                counts[prompt] += 1
                model = prompts[prompt][0]
            else:
                # An untitled session may retry naming on a later user turn.
                # The originating prompt, not arrival order, determines its route.
                self.assertTrue(
                    prompt.startswith("Generate a concise, single-line task title"),
                    prompt,
                )
                source_prompt = prompt.partition("User prompt:\n")[2]
                self.assertIn(source_prompt, prompts)
                model = prompts[source_prompt][0]
            if model is None:
                self.assertNotIn("model", body)
            else:
                self.assertEqual(body.get("model"), model)
        self.assertTrue(all(counts.values()), counts)
        self.assertGreaterEqual(counts[next(iter(prompts))], 2)
        self.assertEqual((self.work / "result.txt").read_text(), "local tool worked\n")
        for _, headers, body in self.requests:
            self.assertNotIn("effort", body.get("reasoning") or {})
            self.assertEqual(
                {key.lower(): value for key, value in headers.items()}["user-agent"],
                f"airs-harness/{self.version}",
            )

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
                "name": "airs-harness",
                "title": "Prisma AIRS Harness",
                "version": self.version,
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
            self.assertEqual(headers["user-agent"], f"airs-harness/{self.version}")
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
            'compact_prompt="AIRS_HARNESS_COMPACTION_TEST"',
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertGreaterEqual(len(self.requests), 3, result.stderr)
        self.assertTrue(
            any(
                "AIRS_HARNESS_COMPACTION_TEST" in json.dumps(body)
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
        inventory = self.run_cli("mcp", "list", "--json")
        self.assertEqual(inventory.returncode, 0, inventory.stderr)
        self.assertEqual(
            json.loads(inventory.stdout)[0]["auth_status"], "credential_helper"
        )
        self.assertNotIn("different-key", inventory.stdout)
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
        missing_tools = self.root / "missing-local-tools"
        missing_tools.mkdir()
        # Keep the npm launcher's prerequisite while hiding Git/ripgrep/Bubblewrap.
        if node := shutil.which("node", path=original_path):
            (missing_tools / Path(node).name).symlink_to(Path(node).resolve())
        self.env["PATH"] = str(missing_tools)
        unavailable = self.run_cli("doctor", "--json")
        self.assertNotEqual(unavailable.returncode, 0)
        self.assertTrue(unavailable.stdout, unavailable.stderr)
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

    def test_doctor_verify_access_uses_bounded_authenticated_wire_and_selected_route(
        self,
    ):
        self.configure()
        config = self.home / "config.toml"
        # Setup advertises --model in the catalog; choose it explicitly for
        # the first probe, then return to the gateway default for the second.
        config.write_text(
            config.read_text().replace(
                'model = "airs-gateway-default"', f'model = "{EXPLICIT}"', 1
            )
        )
        for bound in (False, True):
            with self.subTest(bound=bound):
                if bound:
                    login = self.run_cli(
                        "login", "--credential-env", "AIRS_TEST_CREDENTIAL"
                    )
                    self.assertEqual(login.returncode, 0, login.stderr)
                    config = self.home / "config.toml"
                    config.write_text(
                        config.read_text().replace(
                            f'model = "{EXPLICIT}"', 'model = "airs-gateway-default"', 1
                        )
                    )
                self.requests.clear()
                doctor = self.run_cli("doctor", "--verify-access", "--json")
                self.assertEqual(doctor.returncode, 0, doctor.stderr)
                access = next(
                    item
                    for item in json.loads(doctor.stdout)["checks"]
                    if item["name"] == "gateway_access"
                )
                self.assertTrue(access["passed"])
                self.assertIn("Gateway access verified", access["detail"])
                self.assertIn("MCP permissions were not tested", access["detail"])
                self.assertIn("one minimal inference request", doctor.stderr)
                self.assertNotIn("test-only-credential", doctor.stdout + doctor.stderr)
                self.assertEqual(len(self.requests), 1)
                path, headers, body = self.requests[0]
                headers = {name.lower(): value for name, value in headers.items()}
                self.assertEqual(path, "/prefix/v1/responses")
                self.assertEqual(
                    headers.get("authorization" if bound else "x-portkey-api-key"),
                    "Bearer test-only-credential" if bound else "test-only-credential",
                )
                self.assertNotIn(
                    "x-portkey-api-key" if bound else "authorization", headers
                )
                expected = {
                    "input": "Reply only with OK. This is a Prisma AIRS Harness connectivity check.",
                    "max_output_tokens": 16,
                    "store": False,
                    "stream": False,
                }
                if not bound:
                    expected["model"] = EXPLICIT
                self.assertEqual(body, expected)
                self.assertEqual(self.mcp_requests, [])

    def test_doctor_accepts_reasoning_only_bounded_probe(self):
        self.configure()
        login = self.run_cli("login", "--credential-env", "AIRS_TEST_CREDENTIAL")
        self.assertEqual(login.returncode, 0, login.stderr)
        for status in ("completed", "incomplete"):
            with self.subTest(status=status):
                self.probe_response = (
                    200,
                    {
                        "object": "response",
                        "status": status,
                        "error": None,
                        "output": [{"type": "reasoning", "summary": []}],
                    },
                )
                self.requests.clear()
                result = self.run_cli("doctor", "--verify-access", "--json")
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                access = next(
                    item
                    for item in json.loads(result.stdout)["checks"]
                    if item["name"] == "gateway_access"
                )
                self.assertTrue(access["passed"])
                self.assertEqual(len(self.requests), 1)
                path, _, body = self.requests[0]
                self.assertEqual(path, "/prefix/v1/responses")
                self.assertEqual(body["max_output_tokens"], 16)
                self.assertNotIn("model", body)
                self.assertFalse(body["store"])

    def test_native_status_and_plain_doctor_inspect_configuration_without_store_access(
        self,
    ):
        self.configure()
        login = self.run_cli("login", "--credential-env", "AIRS_TEST_CREDENTIAL")
        self.assertEqual(login.returncode, 0, login.stderr)
        path = self.home / "credential-binding.json"
        binding = json.loads(path.read_text())
        # No native entry exists for this reference binding. Native resolution
        # would fail; metadata inspection must succeed without prompting.
        self.env["DBUS_SESSION_BUS_ADDRESS"] = "unix:path=/nonexistent/airs-status-test"
        for kind in ("keyring", "keyring-v2"):
            with self.subTest(kind=kind):
                binding["source"] = {"kind": kind}
                path.write_text(json.dumps(binding))
                before = path.read_bytes()
                for arguments in (("env", "status"), ("login", "status")):
                    status = self.run_cli(*arguments)
                    self.assertEqual(status.returncode, 0, status.stderr)
                    self.assertIn(
                        "availability and gateway access not checked", status.stdout
                    )
                    self.assertNotIn(
                        "test-only-credential", status.stdout + status.stderr
                    )
                doctor = self.run_cli("doctor", "--json")
                self.assertEqual(doctor.returncode, 0, doctor.stderr)
                row = next(
                    item
                    for item in json.loads(doctor.stdout)["checks"]
                    if item["name"] == "credential_configuration"
                )
                self.assertTrue(row["passed"])
                self.assertIn(
                    "availability and gateway access not checked", row["detail"]
                )
                self.assertEqual(path.read_bytes(), before)
                self.assertEqual(self.requests, [])
                self.assertEqual(self.mcp_requests, [])

    def test_doctor_verify_access_denial_preserves_binding_and_redacts_response(self):
        self.configure()
        login = self.run_cli("login", "--credential-env", "AIRS_TEST_CREDENTIAL")
        self.assertEqual(login.returncode, 0, login.stderr)
        binding = (self.home / "credential-binding.json").read_bytes()
        self.probe_response = (
            403,
            {"error": "PRIVATE-DENIAL-CANARY test-only-credential"},
        )
        doctor = self.run_cli("doctor", "--verify-access", "--json")
        self.assertNotEqual(doctor.returncode, 0)
        access = next(
            item
            for item in json.loads(doctor.stdout)["checks"]
            if item["name"] == "gateway_access"
        )
        self.assertFalse(access["passed"])
        self.assertIn("Gateway access not yet verified", access["detail"])
        self.assertNotIn("PRIVATE-DENIAL-CANARY", doctor.stdout + doctor.stderr)
        self.assertNotIn("test-only-credential", doctor.stdout + doctor.stderr)
        self.assertEqual((self.home / "credential-binding.json").read_bytes(), binding)
        self.assertEqual(len(self.requests), 1)
        self.assertEqual(self.mcp_requests, [])

    def test_doctor_rejects_unsafe_gateway_before_display_or_any_request(self):
        self.configure()
        login = self.run_cli("login", "--credential-env", "AIRS_TEST_CREDENTIAL")
        self.assertEqual(login.returncode, 0, login.stderr)
        binding_path = self.home / "credential-binding.json"
        binding = binding_path.read_bytes()
        path = self.home / "config.toml"
        original = path.read_text()
        for gateway in (
            self.url.replace("http://", "http://user:GATEWAY-URL-CANARY@"),
            self.url + "?token=GATEWAY-URL-CANARY",
            self.url + "#GATEWAY-URL-CANARY",
        ):
            for options in ((), ("--verify-access",)):
                with self.subTest(
                    gateway_type=gateway.split("CANARY")[0][-8:], options=options
                ):
                    path.write_text(original.replace(self.url, gateway))
                    doctor = self.run_cli("doctor", "--json", *options)
                    self.assertNotEqual(doctor.returncode, 0)
                    self.assertNotIn(
                        "GATEWAY-URL-CANARY", doctor.stdout + doctor.stderr
                    )
                    self.assertNotIn(
                        "test-only-credential", doctor.stdout + doctor.stderr
                    )
                    # Named environments reject a changed gateway binding before
                    # doctor can load or display the tampered configuration.
                    self.assertEqual(doctor.stdout, "")
                    self.assertIn("gateway binding changed", doctor.stderr)
                    self.assertEqual(binding_path.read_bytes(), binding)
                    self.assertEqual(self.health_requests, [])
                    self.assertEqual(self.requests, [])
                    self.assertEqual(self.mcp_requests, [])

    def test_doctor_rejects_oversized_or_fifo_catalog_without_blocking(self):
        self.configure()
        catalog = Path(
            tomllib.loads((self.home / "config.toml").read_text())["model_catalog_json"]
        )
        cases = ["oversized"] + (["fifo"] if hasattr(os, "mkfifo") else [])
        for kind in cases:
            with self.subTest(kind=kind):
                catalog.unlink()
                if kind == "fifo":
                    os.mkfifo(catalog, 0o600)
                else:
                    catalog.write_text(
                        json.dumps(
                            {"padding": "CATALOG-PRIVATE-CANARY" + "x" * (1024 * 1024)}
                        )
                    )
                self.health_requests.clear()
                doctor = subprocess.run(
                    [str(BINARY), "doctor", "--json"],
                    env=self.env,
                    cwd=self.work,
                    capture_output=True,
                    text=True,
                    timeout=10,
                )
                self.assertNotEqual(doctor.returncode, 0)
                checks = {
                    row["name"]: row for row in json.loads(doctor.stdout)["checks"]
                }
                self.assertTrue(checks["configuration"]["passed"])
                self.assertFalse(checks["capabilities"]["passed"])
                self.assertIn("at most 1 MiB", checks["capabilities"]["detail"])
                self.assertTrue(checks["gateway_health"]["passed"])
                self.assertEqual(self.health_requests, ["/prefix/v1/health"])
                self.assertEqual(self.requests, [])
                self.assertEqual(self.mcp_requests, [])
                self.assertNotIn(
                    "CATALOG-PRIVATE-CANARY", doctor.stdout + doctor.stderr
                )
                self.assertNotIn("test-only-credential", doctor.stdout + doctor.stderr)

    def test_named_environments_and_file_credential_without_export(self):
        for name in ("work", "second"):
            setup = self.run_cli(
                "env",
                "create",
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
        other = self.run_cli("env", "status", "--environment", "second")
        self.assertNotEqual(
            other.returncode, 0, "second environment must not borrow work's credential"
        )
        logout = self.run_cli("logout")
        self.assertEqual(logout.returncode, 0, logout.stderr)
        repeated = self.run_cli("logout")
        self.assertEqual(repeated.returncode, 0, repeated.stderr)
        self.assertIn("Already logged out locally", repeated.stdout)
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
        project = self.work / ".airs-harness"
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
        # Release binaries refuse executable helper aliases under the OS temp
        # directory. Exercise the same user-owned state layout as real installs.
        state_root = Path.home() / ".cache" / "airs-harness-tests"
        state_root.mkdir(mode=0o700, parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(
            prefix="airs-harness-test-", dir=state_root
        )
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.home = self.root / "state"
        self.work = self.root / "work"
        self.work.mkdir()
        self.requests = []
        self.health_requests = []
        self.mcp_requests = []
        self.redirect = None
        self.force_compaction = False
        self.phase_replies = {}
        self.env = dict(
            os.environ,
            AIRS_HARNESS_HOME=str(self.home),
            AIRS_TEST_CREDENTIAL="test-only-credential",
        )
        owner = self

        class Gateway(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def do_GET(self):
                owner.health_requests.append(self.path)
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
                if body.get("stream") is False:
                    status, response = getattr(
                        owner,
                        "probe_response",
                        (
                            200,
                            {
                                "object": "response",
                                "status": "completed",
                                "error": None,
                                "output": [
                                    {
                                        "type": "message",
                                        "role": "assistant",
                                        "content": [
                                            {"type": "output_text", "text": "OK"}
                                        ],
                                    }
                                ],
                            },
                        ),
                    )
                    data = json.dumps(response).encode()
                    self.send_response(status)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(data)))
                    self.end_headers()
                    self.wfile.write(data)
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
                    command = getattr(owner, "tool_command", command)
                    args = (
                        {"cmd": command}
                        if name == "exec_command"
                        else {"command": command}
                    )
                    name, args = getattr(owner, "initial_function_call", (name, args))
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
                                        "text": (
                                            owner.phase_replies.get(
                                                latest_user_text(body),
                                                "Create result file",
                                            )
                                            if owner.phase_replies
                                            else "Local tool complete."
                                        ),
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
                release = getattr(owner, "question_turn_release", None)
                if (
                    number > 1
                    and release is not None
                    and latest_user_text(body) in owner.phase_replies
                ):
                    # Background naming can arrive between the initial tool call
                    # and its follow-up, so identify the conversation by its prompt.
                    # Questions are actionable during a live turn. Keep it live until
                    # the fixture submits the answer; terminal completion now recovers drafts.
                    completion = ("data: " + json.dumps(events[-1]) + "\n\n").encode()
                    self.wfile.write(data[: -len(completion)])
                    self.wfile.flush()
                    if not release.wait(30):
                        return
                    try:
                        self.wfile.write(completion)
                    except (BrokenPipeError, ConnectionResetError):
                        pass  # Steering may have already cancelled this stream.
                else:
                    self.wfile.write(data)

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Gateway)
        thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        thread.start()
        self.addCleanup(self.server.server_close)
        self.addCleanup(self.server.shutdown)
        self.url = f"http://127.0.0.1:{self.server.server_port}/prefix/v1"

    def tearDown(self):
        evidence = os.environ.get("AIRS_HARNESS_TEST_EVIDENCE")
        if evidence:
            directory = Path(evidence)
            directory.mkdir(parents=True, exist_ok=True)
            # Only synthetic fixture bodies and the public User-Agent are recorded.
            # Never include credential-bearing request headers.
            (directory / f"{self._testMethodName}.json").write_text(
                json.dumps(
                    {
                        "requests": [
                            {
                                "path": path,
                                "body": body,
                                "user_agent": next(
                                    (
                                        v
                                        for k, v in headers.items()
                                        if k.lower() == "user-agent"
                                    ),
                                    None,
                                ),
                            }
                            for path, headers, body in self.requests
                        ],
                        "phase_counts": getattr(self, "phase_counts", {}),
                        "transcript": getattr(self, "terminal_transcript", b"").decode(
                            errors="replace"
                        ),
                    },
                    indent=2,
                )
                + "\n"
            )

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
            "env",
            "create",
            "work",
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
        state = Path(self.env["AIRS_HARNESS_HOME"])
        registry = json.loads((state / "environments.json").read_text())
        self.home = state / "environments" / registry["environments"]["work"]["id"]
        self.assertNotIn(
            "test-only-credential", (self.home / "config.toml").read_text()
        )

    def execute(self, *extra):
        return self.run_cli(
            "exec",
            "--skip-git-repo-check",
            "--ephemeral",
            "-s",
            os.environ.get("AIRS_HARNESS_TEST_SANDBOX", "workspace-write"),
            *extra,
            getattr(
                self,
                "prompt",
                "Create result.txt using a local shell tool and verify its contents.",
            ),
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
            self.assertTrue(headers["user-agent"].startswith("airs-harness/"))
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

    @unittest.skipUnless(
        os.environ.get("AIRS_MANAGED_CLI_ACCEPTANCE"), "requires npm-managed CLI"
    )
    def test_embedded_prisma_skill_runs_managed_doctor_in_agent_shell(self):
        for key in list(self.env):
            if key.startswith(("PANW_", "PRISMA_AIRS_", "DOTENV_")):
                del self.env[key]
        config = self.work / "trusted-cli.json"
        config.write_text("{}")
        self.env["PRISMA_AIRS_TENANTS_PATH"] = str(self.work / "isolated-tenants.json")
        (self.work / ".env").write_text("PANW_AI_SEC_API_KEY=untrusted-project-key\n")
        self.prompt = "$prisma-airs-cli Check the managed CLI version and diagnose missing credentials."
        self.tool_command = (
            '"$AIRS_MANAGED_CLI" --version > managed-version.txt && '
            '("$AIRS_MANAGED_CLI" doctor --output json > managed-doctor.json; test $? -eq 1)'
        )
        self.configure()
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            (self.work / "managed-version.txt").read_text().strip(), "7.2.0"
        )
        statuses = {
            row["name"]: row["status"]
            for row in json.loads((self.work / "managed-doctor.json").read_text())
        }
        self.assertEqual(statuses["Tenant"], "fail")
        self.assertEqual(statuses["Scanner credentials"], "skip")
        self.assertEqual(statuses["Management credentials"], "skip")
        self.assertEqual(statuses["Scanner API"], "skip")
        context = json.dumps(self.requests[0][2]["input"])
        for name in [
            "prisma-airs-cli",
            "prisma-airs-runtime",
            "prisma-airs-guardrails",
            "prisma-airs-redteam",
            "prisma-airs-gateway",
            "prisma-airs-model-security",
            "prisma-airs-dlp-testing",
            "prisma-airs-dlp-management",
        ]:
            self.assertIn(name, context)
        self.assertIn(
            "Doctor is not proof", context, "invoked skill body must reach the agent"
        )

    @unittest.skipUnless(
        os.environ.get("AIRS_MANAGED_CLI_ACCEPTANCE"), "requires npm-managed CLI"
    )
    def test_embedded_gateway_skill_runs_admin_help_in_agent_shell(self):
        self.prompt = "$prisma-airs-gateway Inspect the admin guardrail list options without calling the gateway."
        self.tool_command = (
            '"$AIRS_MANAGED_CLI" aigateway admin-guardrails list --help > admin-help.txt'
        )
        self.configure()
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        help_text = (self.work / "admin-help.txt").read_text()
        for option in ("--workspace", "--page-size", "--current-page"):
            self.assertIn(option, help_text)
        context = json.dumps(self.requests[0][2]["input"])
        self.assertIn("Official API contract and admin guardrails", context)
        self.assertIn("Never switch scopes", context)

    def test_gateway_default_local_tool_and_continuation(self):
        self.assert_tool_loop(None)
        for _, _, body in self.requests:
            self.assertIs(body["parallel_tool_calls"], False)

    def test_explicit_route_local_tool_and_continuation(self):
        self.assert_tool_loop(EXPLICIT)
        for _, _, body in self.requests:
            self.assertIs(body["parallel_tool_calls"], False)

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
        project = self.work / ".airs-harness"
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

    def test_legacy_project_config_is_read_until_new_directory_exists(self):
        self.configure()
        legacy = self.work / ".airs-terminal"
        legacy.mkdir()
        (legacy / "config.toml").write_text(f'model = "{EXPLICIT}"\n')
        with (self.home / "config.toml").open("a") as config:
            config.write(
                f'\n[projects.{json.dumps(str(self.work))}]\ntrust_level = "trusted"\n'
            )
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.requests), 2)
        self.assertTrue(
            all(body.get("model") == EXPLICIT for _, _, body in self.requests)
        )
        current = self.work / ".airs-harness"
        current.mkdir()
        (current / "config.toml").write_text('model = "airs-gateway-default"\n')
        self.requests.clear()
        result = self.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.requests), 2)
        self.assertTrue(all("model" not in body for _, _, body in self.requests))

    def test_unconfigured_and_invalid_routes_fail_before_inference(self):
        result = self.execute()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("airs env create", result.stderr)
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
            "env",
            "create",
            "work",
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
            "env", "create", "work", "--gateway-url", self.url, "--allow-http-loopback"
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        state = Path(self.env["AIRS_HARNESS_HOME"])
        registry = json.loads((state / "environments.json").read_text())
        self.home = state / "environments" / registry["environments"]["work"]["id"]
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
