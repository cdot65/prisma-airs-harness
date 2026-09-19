"""Reject false OAuth and tool-result proof in lifecycle fixture plumbing."""

import base64
import hashlib
import io
import json
import threading
from types import SimpleNamespace
import unittest
from urllib.parse import urlencode

from airs_lifecycle_fixture import LifecycleFixture
from airs_lifecycle_tokens import RotatingTokens


class Handler:
    def __init__(self):
        self.headers = {"x-portkey-api-key": "access"}
        self.wfile = io.BytesIO()
        self.status = None

    def answer(self, status, body):
        self.status = status

    def send_response(self, status):
        self.status = status

    def send_header(self, *_args):
        pass

    def end_headers(self):
        pass


class LifecycleFixtureProof(unittest.TestCase):
    def setUp(self):
        self.events = []
        self.fixture = LifecycleFixture.__new__(LifecycleFixture)
        fixture = self.fixture
        fixture.emit = lambda kind, **fields: self.events.append(
            dict(kind=kind, **fields)
        )
        fixture.lock = threading.RLock()
        fixture.turns, fixture.failure, fixture.mcp_authorization = {}, None, None
        fixture.gateway = SimpleNamespace(endpoint="https://127.0.0.1:443/mcp")
        fixture.mcp = RotatingTokens("mcp", 90, fixture.emit)
        fixture.inference = RotatingTokens("inference", 90, fixture.emit)
        fixture.inference.issue(
            lambda *_: {"access_token": "access", "refresh_token": "refresh"}
        )

    def authorize(self):
        verifier = "correct-fixture-verifier"
        challenge = (
            base64.urlsafe_b64encode(hashlib.sha256(verifier.encode()).digest())
            .rstrip(b"=")
            .decode()
        )
        query = {
            "state": "state",
            "code_challenge_method": "S256",
            "code_challenge": challenge,
            "client_id": "manager-fixture",
            "redirect_uri": "http://127.0.0.1/callback",
        }
        self.fixture.approve_mcp(
            "https://127.0.0.1/authorize?" + urlencode(query),
            "http://127.0.0.1/callback?state=state&code=one-use",
        )
        return {
            "grant_type": ["authorization_code"],
            "code": ["one-use"],
            "code_verifier": [verifier],
            "client_id": ["manager-fixture"],
            "redirect_uri": [query["redirect_uri"]],
            "resource": [self.fixture.gateway.endpoint],
        }

    def test_mcp_code_requires_bound_verifier_resource_and_client(self):
        for field in ("code_verifier", "resource", "client_id", "redirect_uri", "code"):
            with self.subTest(field=field):
                values = self.authorize()
                values[field] = ["changed"]
                self.assertEqual(self.fixture.exchange_mcp(values)[0], 400)
        self.assertEqual(self.fixture.mcp.generation, 0)

    def test_mcp_code_cannot_be_replayed(self):
        values = self.authorize()
        self.assertEqual(self.fixture.exchange_mcp(values)[0], 200)
        self.assertEqual(self.fixture.exchange_mcp(values)[0], 400)
        self.assertEqual(self.fixture.mcp.generation, 1)

    def test_planned_tool_output_without_actual_mcp_call_is_not_proof(self):
        prompt = self.fixture.prepare_turn("test")
        body = {
            "input": [{"role": "user", "content": [{"text": prompt}]}],
            "tools": [{"name": "mcp__fixture__incident_lookup"}],
        }
        handler = Handler()
        self.fixture.responses(handler, body)
        row = self.fixture.turns["test"]
        body["input"].append(
            {
                "type": "function_call_output",
                "call_id": row["call_id"],
                "output": "LOOKUP_" + row["nonce"],
            }
        )
        self.fixture.responses(handler, body)
        self.assertEqual(handler.status, 400)
        self.assertFalse(row["output_verified"])
        self.assertFalse(
            any(event["kind"] == "tool_completed" for event in self.events)
        )

    def test_duplicate_tool_call_cannot_supply_success_proof(self):
        self.fixture.prepare_turn("test")
        row = self.fixture.turns["test"]
        body = {"params": {"arguments": {"turn": "test", "nonce": row["nonce"]}}}
        self.assertNotIn("isError", self.fixture.tool_call(body))
        self.assertTrue(self.fixture.tool_call(body)["isError"])
        self.assertEqual(row["tool_calls"], 1)
