"""Disposable HTTPS gateway/OAuth fixture for installed in-session MCP tests."""

import json
import ssl
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit

from airs_onboarding_fixture import IdentityFixture


class McpGatewayFixture:
    def __init__(
        self, directory, *, token_exchange=None, authorize=None, tool_response=None
    ):
        # Reuse the onboarding fixture's CA/leaf generator without changing trust globally.
        identity = IdentityFixture(directory)
        identity.close()
        self.certificate = identity.certificate
        self.tokens, self.requests, self.registrations = [], [], []
        self.network_requests = []
        self.reject_tools = False
        self.registration_failure = None
        owner = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def reply(self, status, body, headers=None):
                data = json.dumps(body).encode()
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(data)))
                for name, value in (headers or {}).items():
                    self.send_header(name, value)
                self.end_headers()
                self.wfile.write(data)

            def do_GET(self):
                path = urlsplit(self.path).path
                owner.network_requests.append(("GET", path))
                if path.startswith("/.well-known/oauth-protected-resource"):
                    self.reply(
                        200,
                        {
                            "resource": owner.endpoint,
                            "authorization_servers": [owner.base],
                            "scopes_supported": [
                                "mcp:servers:read",
                                "mcp:tools:list",
                                "mcp:tools:call",
                            ],
                        },
                    )
                elif path.startswith("/.well-known/oauth-authorization-server"):
                    self.reply(
                        200,
                        {
                            "issuer": owner.base,
                            "authorization_endpoint": owner.base + "/authorize",
                            "token_endpoint": owner.base + "/token",
                            "registration_endpoint": owner.base + "/register",
                            "response_types_supported": ["code"],
                            "code_challenge_methods_supported": ["S256"],
                            "authorization_response_iss_parameter_supported": True,
                        },
                    )
                else:
                    self.reply(405, {})

            def do_POST(self):
                owner.network_requests.append(("POST", urlsplit(self.path).path))
                raw = self.rfile.read(int(self.headers.get("Content-Length", 0)))
                if self.path == "/register":
                    request = json.loads(raw)
                    owner.registrations.append(request)
                    if owner.registration_failure is not None:
                        self.reply(*owner.registration_failure)
                        return
                    self.reply(
                        201,
                        {
                            **request,
                            "client_id": "manager-fixture",
                            "token_endpoint_auth_method": "none",
                        },
                    )
                elif self.path == "/token":
                    owner.tokens.append(parse_qs(raw.decode()))
                    if token_exchange is not None:
                        self.reply(*token_exchange(owner.tokens[-1]))
                        return
                    self.reply(
                        200,
                        {
                            "access_token": "fixture-mcp-access",
                            "refresh_token": "fixture-mcp-refresh",
                            "token_type": "Bearer",
                            "expires_in": 3600,
                        },
                    )
                elif self.path == "/gateway/service-now/mcp":
                    body = json.loads(raw)
                    authorized = (
                        authorize(self.headers.get("Authorization", ""), body)
                        if authorize is not None
                        else self.headers.get("Authorization")
                        == "Bearer fixture-mcp-access"
                    )
                    if not authorized:
                        self.reply(
                            401,
                            {},
                            {
                                "WWW-Authenticate": f'Bearer resource_metadata="{owner.base}/.well-known/oauth-protected-resource"'
                            },
                        )
                        return
                    owner.requests.append(body)
                    if "id" not in body:
                        self.reply(202, {})
                        return
                    method = body["method"]
                    if method == "tools/call" and tool_response is not None:
                        self.reply(
                            200,
                            {
                                "jsonrpc": "2.0",
                                "id": body["id"],
                                "result": tool_response(body),
                            },
                        )
                        return
                    if method == "tools/list" and owner.reject_tools:
                        self.reply(
                            200,
                            {
                                "jsonrpc": "2.0",
                                "id": body["id"],
                                "error": {
                                    "code": -32603,
                                    "message": "Fixture discovery failure",
                                },
                            },
                        )
                        return
                    result = {
                        "initialize": {
                            "protocolVersion": "2025-06-18",
                            "capabilities": {"tools": {}},
                            "serverInfo": {"name": "gateway-fixture", "version": "1"},
                        },
                        "tools/list": {
                            "tools": [
                                {
                                    "name": "incident_lookup",
                                    "description": "Read a fixture incident",
                                    "inputSchema": {"type": "object", "properties": {}},
                                }
                            ]
                        },
                        "resources/list": {"resources": []},
                        "resources/templates/list": {"resourceTemplates": []},
                        "tools/call": {
                            "content": [
                                {"type": "text", "text": "Fixture incident INC001"}
                            ]
                        },
                    }.get(method, {})
                    self.reply(
                        200, {"jsonrpc": "2.0", "id": body["id"], "result": result}
                    )
                else:
                    self.reply(404, {})

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.base = f"https://127.0.0.1:{self.server.server_port}"
        self.endpoint = self.base + "/gateway/service-now/mcp"
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(
            directory / "fixture-server.pem", directory / "fixture-server.key"
        )
        self.server.socket = context.wrap_socket(self.server.socket, server_side=True)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def close(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=5)
