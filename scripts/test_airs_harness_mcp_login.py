"""Exercise manual MCP OAuth on the exact executable through a real terminal.

The authorization service and credentials are disposable loopback fixtures.
No browser, production token, or operating-system credential store is used.
"""

import json

from airs_fixture_config import set_mcp_store
import os
from pathlib import Path
import re
import subprocess
import tempfile
import termios
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlencode, urlsplit, urlunsplit
from urllib.request import urlopen

from airs_harness_pty import TerminalSession

BINARY = Path(
    os.environ.get("AIRS_HARNESS_BIN", "codex-rs/target/debug/airs-harness")
).resolve()


class ManualMcpLogin(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="airs-mcp-login-")
        self.addCleanup(self.directory.cleanup)
        self.home = Path(self.directory.name)
        self.tokens = []
        owner = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def reply(self, status, body):
                data = json.dumps(body).encode()
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)

            def do_GET(self):
                if self.path == "/.well-known/oauth-authorization-server/mcp":
                    self.reply(
                        200,
                        {
                            "issuer": owner.issuer,
                            "authorization_endpoint": owner.base + "/authorize",
                            "token_endpoint": owner.base + "/token",
                            "response_types_supported": ["code"],
                            "code_challenge_methods_supported": ["S256"],
                            "authorization_response_iss_parameter_supported": True,
                        },
                    )
                else:
                    self.reply(404, {})

            def do_POST(self):
                if self.path != "/token":
                    self.reply(404, {})
                    return
                body = self.rfile.read(int(self.headers["Content-Length"]))
                owner.tokens.append(parse_qs(body.decode()))
                self.reply(
                    200,
                    {
                        "access_token": "fixture-access-token",
                        "refresh_token": "fixture-refresh-token",
                        "token_type": "Bearer",
                    },
                )

        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        self.base = f"http://127.0.0.1:{server.server_port}"
        self.issuer = self.base + "/mcp"
        self.env = dict(
            os.environ,
            AIRS_HARNESS_HOME=str(self.home),
            CODEX_HOME=str(self.home),
            AIRS_MCP_FIXTURE_KEY="isolated-inference-key",
            NO_PROXY="127.0.0.1,localhost",
            no_proxy="127.0.0.1,localhost",
        )
        setup = subprocess.run(
            [
                str(BINARY),
                "env",
                "create",
                "work",
                "--gateway-url",
                self.base + "/v1",
                "--allow-http-loopback",
                "--credential-env",
                "AIRS_MCP_FIXTURE_KEY",
            ],
            env=self.env,
            cwd=self.home,
            capture_output=True,
            text=True,
            timeout=20,
        )
        self.assertEqual(setup.returncode, 0, setup.stderr)
        registry = json.loads((self.home / "environments.json").read_text())
        self.home = self.home / "environments" / registry["environments"]["work"]["id"]
        config = self.home / "config.toml"
        set_mcp_store(config, "file")
        config.write_text(
            config.read_text()
            + f'\n[mcp_servers.manual]\nurl = "{self.issuer}"\n'
            + '[mcp_servers.manual.oauth]\nclient_id = "fixture-client"\n'
        )

    def test_pasted_callback_is_hidden_and_persists_once(self):
        self.exercise("paste")

    def test_http_callback_restores_terminal_with_pending_input(self):
        self.exercise("http")

    def test_cancel_restores_terminal_without_token_exchange(self):
        self.exercise("cancel")

    def exercise(self, delivery):
        with TerminalSession(
            BINARY,
            self.env,
            self.home,
            arguments=["mcp", "login", "manual", "--no-browser"],
        ) as terminal:
            terminal.wait_for(b"Callback URL (input hidden):")
            self.assertFalse(termios.tcgetattr(terminal.master)[3] & termios.ECHO)
            authorization = re.search(
                rb"http://127\.0\.0\.1:[0-9]+/authorize\?[^\s]+",
                terminal.transcript,
            )
            self.assertIsNotNone(authorization)
            params = parse_qs(urlsplit(authorization[0].decode()).query)
            address = urlsplit(params["redirect_uri"][0])
            callback = urlunsplit(
                address._replace(
                    query=urlencode(
                        {
                            "state": params["state"][0],
                            "code": "private-fixture-code-never-echo",
                            "iss": self.issuer,
                        }
                    )
                )
            )
            if delivery == "cancel":
                os.write(terminal.master, b"private-unfinished-callback\x03")
                terminal.wait_for(b"OAuth login cancelled")
            elif delivery == "paste":
                terminal.send_line(callback)
                terminal.wait_for(b"Successfully logged in")
            else:
                with urlopen(callback, timeout=10) as response:
                    self.assertEqual(response.status, 200)
                terminal.wait_for(b"Successfully logged in")
            terminal.wait_until(lambda: terminal.process.poll() is not None)
            self.assertEqual(terminal.process.returncode == 0, delivery != "cancel")
            self.assertNotIn(b"private-fixture-code-never-echo", terminal.transcript)
            self.assertNotIn(b"private-unfinished-callback", terminal.transcript)
            mode = termios.tcgetattr(terminal.master)[3]
            self.assertTrue(mode & termios.ECHO)
            self.assertTrue(mode & termios.ICANON)
        saved = self.home / ".credentials.json"
        if delivery == "cancel":
            self.assertEqual(self.tokens, [])
            self.assertFalse(saved.exists())
        else:
            self.assertEqual(len(self.tokens), 1)
            self.assertEqual(self.tokens[0]["grant_type"], ["authorization_code"])
            self.assertEqual(self.tokens[0]["resource"], [self.issuer])
            self.assertEqual(self.tokens[0]["client_id"], ["fixture-client"])
            self.assertTrue(self.tokens[0]["code_verifier"])
            credentials = list(json.loads(saved.read_text()).values())
            self.assertEqual(len(credentials), 1)
            self.assertEqual(credentials[0]["server_url"], self.issuer)
            self.assertEqual(credentials[0]["access_token"], "fixture-access-token")
