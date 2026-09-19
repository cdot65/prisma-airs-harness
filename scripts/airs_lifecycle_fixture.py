"""Synthetic signed OIDC inference and independently rotating gateway MCP."""

import json
import base64
import hashlib
import secrets
import threading
import time
from urllib.parse import parse_qs, urlsplit

from airs_lifecycle_tokens import RotatingTokens, opaque_response
from airs_mcp_manager_fixture import McpGatewayFixture
from airs_onboarding_fixture import IdentityFixture
from test_airs_harness import latest_user_text


class LifecycleIdentity(IdentityFixture):
    def __init__(self, directory, tokens, gateway):
        self.rotating = tokens
        super().__init__(
            directory, refresh_exchange=self.refresh, gateway_response=gateway
        )

    def build_tokens(self, generation, expires, nonce=None):
        claims = {
            "iss": self.issuer,
            "sub": self.subject,
            "iat": int(time.time()),
            "exp": expires,
            "azp": self.client,
            "jti": f"generation-{generation}",
        }
        access = self.jwt({**claims, "aud": self.audience})
        identity = {**claims, "aud": self.client, "preferred_username": "fixture-user"}
        if nonce is not None:
            identity["nonce"] = nonce
        refresh = "fixture-oidc-refresh-" + secrets.token_hex(24)
        self.access_tokens.add(access)
        self.refresh_tokens.add(refresh)
        return {
            "token_type": "Bearer",
            "expires_in": self.rotating.lifetime,
            "access_token": access,
            "id_token": self.jwt(identity),
            "refresh_token": refresh,
        }

    def tokens(self, nonce):
        return self.rotating.issue(
            lambda generation, expires: self.build_tokens(generation, expires, nonce)
        )

    def refresh(self, values):
        if values.get("client_id") != [self.client]:
            return 400, {"error": "invalid_client"}
        return self.rotating.exchange(values, self.build_tokens)


class LifecycleFixture:
    def __init__(self, root, lifetime, emit):
        self.emit, self.lock = emit, threading.RLock()
        self.turns = {}
        self.failure = None
        self.mcp_authorization = None
        self.inference = RotatingTokens("inference", lifetime, emit)
        self.mcp = RotatingTokens("mcp", lifetime, emit)
        identity_root, mcp_root = root / "identity", root / "mcp"
        identity_root.mkdir()
        mcp_root.mkdir()
        self.identity = LifecycleIdentity(identity_root, self.inference, self.responses)
        try:
            self.gateway = McpGatewayFixture(
                mcp_root,
                token_exchange=self.exchange_mcp,
                authorize=self.authorize_mcp,
                tool_response=self.tool_call,
            )
        except BaseException:
            self.identity.close()
            raise
        self.certificate = root / "combined-ca.pem"
        self.certificate.write_bytes(
            self.identity.certificate.read_bytes()
            + self.gateway.certificate.read_bytes()
        )

    def close(self):
        self.gateway.close()
        self.identity.close()

    def exchange_mcp(self, values):
        build = lambda generation, expires: opaque_response(
            generation, expires, self.mcp.lifetime
        )
        if values.get("grant_type") == ["refresh_token"]:
            if values.get("client_id") != ["manager-fixture"]:
                return 400, {"error": "invalid_client"}
            return self.mcp.exchange(values, build)
        expected, self.mcp_authorization = self.mcp_authorization, None
        challenge = (
            base64.urlsafe_b64encode(
                hashlib.sha256(values.get("code_verifier", [""])[0].encode()).digest()
            )
            .rstrip(b"=")
            .decode()
        )
        if (
            values.get("grant_type") != ["authorization_code"]
            or expected is None
            or values.get("code") != [expected["code"]]
            or values.get("client_id") != expected["client_id"]
            or values.get("redirect_uri") != expected["redirect_uri"]
            or values.get("resource") != [self.gateway.endpoint]
            or challenge != expected["code_challenge"][0]
        ):
            return 400, {"error": "invalid_grant"}
        return 200, self.mcp.issue(build)

    def approve_mcp(self, authorization, callback):
        query = parse_qs(urlsplit(authorization).query)
        returned = parse_qs(urlsplit(callback).query)
        if query.get("state") != returned.get("state") or query.get(
            "code_challenge_method"
        ) != ["S256"]:
            raise ValueError("Synthetic consent did not preserve state and PKCE")
        self.mcp_authorization = dict(query, code=returned["code"][0])

    def authorize_mcp(self, header, body):
        details = {}
        if body.get("method") == "tools/call":
            label = body.get("params", {}).get("arguments", {}).get("turn")
            if label in self.turns:
                details = {"turn": label, "call_id": self.turns[label]["call_id"]}
        return self.mcp.authorize(
            header.removeprefix("Bearer "), body.get("method", "unknown"), **details
        )

    def prepare_turn(self, label):
        with self.lock:
            if label in self.turns:
                raise ValueError("Lifecycle turn labels must be unique")
            nonce = secrets.token_hex(12)
            self.turns[label] = {
                "nonce": nonce,
                "call_id": "lifecycle-" + nonce,
                "requests": 0,
                "tool_calls": 0,
                "output_verified": False,
            }
            return "AIRS_LIFECYCLE_" + label

    def tool_call(self, body):
        with self.lock:
            args = body.get("params", {}).get("arguments", {})
            label = args.get("turn")
            turn = self.turns.get(label)
            if not turn or args.get("nonce") != turn["nonce"] or turn["tool_calls"]:
                self.failure = "MCP call was unknown, duplicate, or changed its nonce"
                return {
                    "isError": True,
                    "content": [{"type": "text", "text": "Invalid fixture lookup"}],
                }
            turn["tool_calls"] += 1
            return {"content": [{"type": "text", "text": "LOOKUP_" + turn["nonce"]}]}

    def responses(self, handler, body):
        with self.lock:
            label = latest_user_text(body).removeprefix("AIRS_LIFECYCLE_")
            details = {"turn": label} if label in self.turns else {}
            token = handler.headers.get(
                "x-portkey-api-key",
                handler.headers.get("Authorization", "").removeprefix("Bearer "),
            )
            if not self.inference.authorize(token, "responses", **details):
                handler.answer(401, {"error": {"message": "Expired synthetic access"}})
                return
            if body.get("stream") is False:
                # Login's explicit bounded access verification remains synthetic.
                handler.answer(
                    200,
                    {
                        "object": "response",
                        "status": "completed",
                        "output": [
                            {
                                "type": "message",
                                "role": "assistant",
                                "content": [{"type": "output_text", "text": "OK"}],
                            }
                        ],
                    },
                )
                return
            turn = self.turns.get(label)
            if turn is None:
                self.failure = (
                    "Inference request did not match a prepared lifecycle turn"
                )
                handler.answer(400, {"error": {"message": self.failure}})
                return
            turn["requests"] += 1
            if turn["requests"] == 1:
                name = next(
                    (
                        tool.get("name")
                        for tool in body.get("tools", [])
                        if tool.get("name", "").endswith("incident_lookup")
                    ),
                    None,
                )
                if name is None:
                    self.failure = "MCP lookup was absent from actual inference tools"
                    handler.answer(400, {"error": {"message": self.failure}})
                    return
                item = {
                    "type": "function_call",
                    "call_id": turn["call_id"],
                    "name": name,
                    "arguments": json.dumps({"turn": label, "nonce": turn["nonce"]}),
                }
            else:
                outputs = [
                    item
                    for item in body.get("input", [])
                    if item.get("type") == "function_call_output"
                    and item.get("call_id") == turn["call_id"]
                ]
                if (
                    turn["requests"] != 2
                    or turn["tool_calls"] != 1
                    or len(outputs) != 1
                    or "LOOKUP_" + turn["nonce"] not in str(outputs[0].get("output"))
                ):
                    self.failure = "Actual MCP call and matching inference tool output were not verified"
                    handler.answer(400, {"error": {"message": self.failure}})
                    return
                turn["output_verified"] = True
                self.emit(
                    "tool_completed",
                    turn=label,
                    call_id=turn["call_id"],
                    nonce_verified=True,
                )
                item = {
                    "type": "message",
                    "role": "assistant",
                    "id": "message-" + turn["nonce"],
                    "content": [
                        {"type": "output_text", "text": "LIFECYCLE_OK_" + label}
                    ],
                }
            response_id = "response-" + turn["nonce"] + "-" + str(turn["requests"])
            events = [
                {"type": "response.created", "response": {"id": response_id}},
                {"type": "response.output_item.done", "item": item},
                {"type": "response.completed", "response": {"id": response_id}},
            ]
            data = "".join(
                f"event: {event['type']}\ndata: {json.dumps(event)}\n\n"
                for event in events
            ).encode()
            handler.send_response(200)
            handler.send_header("Content-Type", "text/event-stream")
            handler.send_header("Content-Length", str(len(data)))
            handler.end_headers()
            handler.wfile.write(data)
