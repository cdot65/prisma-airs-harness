"""Real-terminal MCP manager acceptance using isolated inference and HTTPS OAuth.

AIRS_HARNESS_BIN selects the development/installed executable. These fixtures do
not establish production gateway or upstream ServiceNow OAuth acceptance.
"""

import json

from airs_fixture_config import set_mcp_store
import os
import re
import time
import unittest
from urllib.parse import parse_qs, urlencode, urlsplit, urlunsplit
from urllib.request import urlopen

import test_airs_harness as harness
from airs_harness_pty import TerminalSession
from airs_mcp_manager_fixture import McpGatewayFixture


class McpManager(unittest.TestCase):
    def setUp(self):
        self.inference = harness.TerminalIntegration()
        self.inference.setUp()
        self.addCleanup(self.inference.doCleanups)
        self.inference.configure()
        self.home = self.inference.home
        self.gateway = McpGatewayFixture(self.inference.root)
        self.addCleanup(self.gateway.close)
        self.env = dict(
            self.inference.env,
            SSL_CERT_FILE=str(self.gateway.certificate),
            NO_PROXY="127.0.0.1,localhost",
            no_proxy="127.0.0.1,localhost",
        )
        self.env.pop("CODEX_CA_CERTIFICATE", None)
        config = self.home / "config.toml"
        set_mcp_store(config, "file")
        self.before = config.read_text()

    def key(self, terminal, keys):
        offset = len(terminal.transcript)
        os.write(terminal.master, keys)
        return offset

    def choose(self, terminal, down=0):
        # Pause between transition and selection to avoid startup key debouncing.
        time.sleep(0.15)
        return self.key(terminal, b"\x1b[B" * down + b"\r")

    def add(self, terminal, name="service-now"):
        terminal.send_line("/mcp")
        terminal.wait_for(b"Add gateway MCP server")
        self.choose(terminal)
        terminal.wait_for(b"Name this MCP connection")
        terminal.send_line(name)
        terminal.wait_for(b"https://gateway-mcp.example.com/service-now/mcp")
        time.sleep(0.15)
        terminal.send_line(self.gateway.endpoint)
        terminal.wait_for(b"Sign in to gateway MCP", timeout=45)
        terminal.wait_for(b"Ctrl+O")

    def callback(self, terminal):
        # Ctrl+Y's OSC52 carries the full URL even if the terminal preview wraps.
        import base64

        offset = self.key(terminal, b"\x19")
        terminal.wait_until(
            lambda: (
                re.search(
                    rb"\x1b\]52;[^;]*;([A-Za-z0-9+/=]+)\x07",
                    terminal.transcript[offset:],
                )
                is not None
            )
        )
        encoded = re.search(
            rb"\x1b\]52;[^;]*;([A-Za-z0-9+/=]+)\x07", terminal.transcript[offset:]
        )[1]
        authorization = base64.b64decode(encoded).decode()
        query = parse_qs(urlsplit(authorization).query)
        self.assertEqual(
            set(query["scope"][0].split()),
            {"mcp:servers:read", "mcp:tools:list", "mcp:tools:call"},
        )
        self.assertEqual(query["code_challenge_method"], ["S256"])
        return urlunsplit(
            urlsplit(query["redirect_uri"][0])._replace(
                query=urlencode(
                    {
                        "state": query["state"][0],
                        "code": "PRIVATE-MANAGER-CODE",
                        "iss": self.gateway.base,
                    }
                )
            )
        )

    def test_remote_callback_add_verify_logout_remove_and_environment_pinning(self):
        self.env["SSH_CONNECTION"] = "fixture"  # deterministic OSC52 clipboard
        with TerminalSession(harness.BINARY, self.env, self.inference.work) as terminal:
            terminal.start()
            # Change the default after launch; UI mutations must remain bound to work.
            other = self.inference.run_cli(
                "env",
                "create",
                "other",
                "--gateway-url",
                self.inference.url,
                "--allow-http-loopback",
                "--credential-env",
                "AIRS_TEST_CREDENTIAL",
            )
            self.assertEqual(other.returncode, 0, other.stderr)
            self.add(terminal)
            callback = self.callback(terminal)
            self.key(terminal, b"\x1b[200~" + callback.encode() + b"\x1b[201~\r")
            terminal.wait_for(b"MCP connection updated", timeout=45)
            terminal.wait_for(b"connected")
            self.choose(terminal)  # explicit new conversation
            terminal.wait_for(b"Review your draft")
            terminal.send_line("/mcp")
            offset = len(terminal.transcript)
            terminal.wait_for(b"MCP connections", offset)
            self.choose(terminal)  # service-now
            terminal.wait_for(b"Reconnect and verify")
            self.choose(terminal, 2)
            terminal.wait_for(b"MCP connection updated", offset, timeout=45)
            self.choose(terminal)
            offset = len(terminal.transcript)
            terminal.wait_for(b"Review your draft", offset)
            terminal.send_line("/mcp")
            terminal.wait_for(b"MCP connections", offset)
            self.choose(terminal)
            terminal.wait_for(b"Sign out", offset)
            self.choose(terminal, 1)
            terminal.wait_for(b"Sign out of service-now?", offset)
            self.choose(terminal, 1)
            terminal.wait_for(b"signed out", offset, timeout=45)
            self.choose(terminal, 1)  # manage connections, no inferred retry
            offset = len(terminal.transcript)
            terminal.wait_for(b"MCP connections", offset)
            self.choose(terminal)
            terminal.wait_for(b"Remove connection", offset)
            self.choose(terminal, 3)
            terminal.wait_for(b"Remove service-now?", offset)
            self.choose(terminal, 1)
            terminal.wait_for(b"removed from this environment", offset, timeout=45)
            self.assertNotIn(b"PRIVATE-MANAGER-CODE", terminal.transcript)
            self.assertEqual(self.inference.requests, [])
            self.choose(terminal)
            terminal.wait_for(b"Review your draft", offset)
            # Start at the fixture's message-only response, without requesting a shell tool.
            self.inference.requests.append(("fixture-bootstrap", {}, {}))
            self.inference.phase_replies = {
                "Confirm inference still works": "Inference preserved"
            }
            terminal.send_line("Confirm inference still works")
            terminal.wait_for(b"Inference preserved", timeout=45)
        self.assertEqual(len(self.gateway.tokens), 1)
        self.assertTrue(self.gateway.tokens[0]["code_verifier"])
        self.assertTrue(any(r["method"] == "tools/list" for r in self.gateway.requests))
        self.assertNotIn(
            "mcp_servers.service-now", (self.home / "config.toml").read_text()
        )
        registry = json.loads(
            (self.inference.root / "state/environments.json").read_text()
        )
        other_home = (
            self.inference.root
            / "state/environments"
            / registry["environments"]["other"]["id"]
        )
        self.assertNotIn("mcp_servers", (other_home / "config.toml").read_text())
        requests = self.inference.requests[1:]
        self.assertTrue(requests)
        for path, headers, body in requests:
            self.assertEqual(path, "/prefix/v1/responses")
            self.assertNotIn("PRIVATE-MANAGER-CODE", json.dumps(body))
            self.assertNotIn("fixture-mcp-access", json.dumps(headers))
        credentials = self.home / ".credentials.json"
        self.assertNotIn(
            "fixture-mcp-access",
            credentials.read_text() if credentials.exists() else "",
        )
        for rollout in self.home.rglob("*.jsonl"):
            self.assertNotIn("PRIVATE-MANAGER-CODE", rollout.read_text())

    def test_cancel_keeps_connection_and_never_exchanges_or_submits_callback(self):
        with TerminalSession(harness.BINARY, self.env, self.inference.work) as terminal:
            terminal.start()
            self.add(terminal)
            self.key(terminal, b"\x1b[200~PRIVATE-UNFINISHED\x1b[201~\x1b")
            terminal.wait_for(b"MCP sign-in cancelled", timeout=45)
            self.assertNotIn(b"PRIVATE-UNFINISHED", terminal.transcript)
        self.assertEqual(self.gateway.tokens, [])
        self.assertEqual(self.inference.requests, [])
        self.assertIn("service-now", (self.home / "config.toml").read_text())

    def test_desktop_callback_and_failed_discovery_do_not_report_success(self):
        self.env["SSH_CONNECTION"] = "fixture"
        with TerminalSession(harness.BINARY, self.env, self.inference.work) as terminal:
            terminal.start()
            self.add(terminal)
            callback = self.callback(terminal)
            self.gateway.reject_tools = True
            offset = len(terminal.transcript)
            with urlopen(callback, timeout=10) as response:
                self.assertEqual(response.status, 200)
            terminal.wait_for(b"did not confirm the MCP connection", offset, timeout=45)
            self.assertNotIn(b"MCP connection updated", terminal.transcript[offset:])
            self.assertNotIn(b"PRIVATE-MANAGER-CODE", terminal.transcript)
        self.assertEqual(len(self.gateway.tokens), 1)
        self.assertEqual(self.inference.requests, [])

    def test_private_adapter_rejects_duplicate_name_without_replacing_it(self):
        import subprocess

        config = self.home / "config.toml"
        config.write_text(
            config.read_text()
            + f'\n[mcp_servers.existing]\nurl = "{self.gateway.endpoint}"\n'
        )
        before = config.read_bytes()
        result = subprocess.run(
            [
                str(harness.BINARY),
                "mcp",
                "add",
                "--no-browser",
                "--url",
                self.gateway.endpoint,
                "--",
                "existing",
            ],
            env=dict(
                self.env,
                AIRS_HARNESS_HOME=str(self.home),
                AIRS_MCP_INTERACTION="json-v1",
            ),
            cwd=self.inference.work,
            input="",
            capture_output=True,
            text=True,
            timeout=30,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("already exists", result.stderr)
        self.assertEqual(config.read_bytes(), before)
        self.assertEqual(self.gateway.tokens, [])
