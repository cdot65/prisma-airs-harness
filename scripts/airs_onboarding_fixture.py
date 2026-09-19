"""Local HTTPS OIDC and gateway fixture. Never connects to production services."""

import base64
import hashlib
import json
import secrets
import ssl
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlencode, urlsplit


def encoded(value):
    return base64.urlsafe_b64encode(value).rstrip(b"=").decode()


class IdentityFixture:
    def __init__(self, directory, *, refresh_exchange=None, gateway_response=None):
        self.directory = Path(directory)
        self.codes, self.devices, self.access_tokens = {}, {}, set()
        self.refresh_tokens = set()
        self.requests, self.exchanges = [], []
        self.gateway_status = 200
        self.deny_browser = False
        self.invalid_nonce = False
        self.subject = "onboarding-fixture-user"
        self.client = "airs-fixture-client"
        self.audience = "airs-fixture-gateway"
        self.key = (
            Path(__file__).resolve().parents[1]
            / "codex-rs/airs-identity/src/fixtures/test-only-private.pem"
        )
        self.keys = json.loads(self.key.with_name("test-only-jwks.json").read_text())
        self.certificate = self.directory / "fixture-ca.pem"
        tls_key = self.directory / "fixture-tls.key"
        subprocess.run(
            [
                "openssl",
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-days",
                "1",
                "-subj",
                "/CN=localhost",
                "-addext",
                "subjectAltName=DNS:localhost,IP:127.0.0.1",
                "-addext",
                "basicConstraints=critical,CA:TRUE",
                "-addext",
                "keyUsage=critical,keyCertSign,cRLSign",
                "-keyout",
                str(tls_key),
                "-out",
                str(self.certificate),
            ],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        ca_key = tls_key
        tls_key = self.directory / "fixture-server.key"
        request = self.directory / "fixture-server.csr"
        leaf = self.directory / "fixture-server.pem"
        extensions = self.directory / "fixture-server.ext"
        extensions.write_text(
            "basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost,IP:127.0.0.1\nsubjectKeyIdentifier=hash\nauthorityKeyIdentifier=keyid,issuer\n"
        )
        subprocess.run(
            [
                "openssl",
                "req",
                "-new",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-subj",
                "/CN=localhost",
                "-keyout",
                str(tls_key),
                "-out",
                str(request),
            ],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        subprocess.run(
            [
                "openssl",
                "x509",
                "-req",
                "-in",
                str(request),
                "-CA",
                str(self.certificate),
                "-CAkey",
                str(ca_key),
                "-CAcreateserial",
                "-days",
                "1",
                "-extfile",
                str(extensions),
                "-out",
                str(leaf),
            ],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        ca_key.chmod(0o600)
        tls_key.chmod(0o600)
        fixture = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def answer(self, status, body):
                data = json.dumps(body).encode()
                self.send_response(status)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)

            def do_GET(self):
                path = urlsplit(self.path).path
                fixture.requests.append(("GET", path))
                query = parse_qs(urlsplit(self.path).query)
                if path == "/.well-known/openid-configuration":
                    self.answer(
                        200,
                        {
                            "issuer": fixture.issuer,
                            "authorization_endpoint": fixture.issuer + "/authorize",
                            "token_endpoint": fixture.issuer + "/token",
                            "revocation_endpoint": fixture.issuer + "/revoke",
                            "jwks_uri": fixture.issuer + "/keys",
                            "device_authorization_endpoint": fixture.issuer + "/device",
                            "code_challenge_methods_supported": ["S256"],
                            "id_token_signing_alg_values_supported": ["RS256"],
                            "subject_types_supported": ["public"],
                            "response_types_supported": ["code"],
                        },
                    )
                elif path == "/keys":
                    self.answer(200, fixture.keys)
                elif path == "/authorize":
                    assert query["client_id"] == [fixture.client]
                    assert query["code_challenge_method"] == ["S256"]
                    redirect = query["redirect_uri"][0]
                    parsed = urlsplit(redirect)
                    assert (
                        parsed.scheme == "http"
                        and parsed.hostname == "127.0.0.1"
                        and parsed.path == "/callback"
                    )
                    code = secrets.token_urlsafe(24)
                    fixture.codes[code] = query
                    params = {"state": query["state"][0], "iss": fixture.issuer}
                    params.update(
                        {"error": "access_denied"}
                        if fixture.deny_browser
                        else {"code": code}
                    )
                    self.send_response(302)
                    self.send_header("Location", redirect + "?" + urlencode(params))
                    self.send_header("Content-Length", "0")
                    self.end_headers()
                elif path == "/verify":
                    for device in fixture.devices.values():
                        if device["user_code"] == query.get("user_code", [""])[0]:
                            device["authorized"] = True
                    self.answer(200, {"authorized": True})
                else:
                    self.answer(404, {"error": "not_found"})

            def do_POST(self):
                path = urlsplit(self.path).path
                fixture.requests.append(("POST", path))
                raw = self.rfile.read(int(self.headers.get("Content-Length", "0")))
                values = parse_qs(raw.decode())
                if path == "/device":
                    assert values["client_id"] == [fixture.client]
                    code = secrets.token_urlsafe(24)
                    device = {
                        "challenge": values["code_challenge"][0],
                        "user_code": "AIRS-" + secrets.token_hex(2).upper(),
                        "authorized": False,
                    }
                    fixture.devices[code] = device
                    self.answer(
                        200,
                        {
                            "device_code": code,
                            "user_code": device["user_code"],
                            "verification_uri": fixture.issuer + "/verify",
                            "expires_in": 300,
                            "interval": 1,
                        },
                    )
                elif path == "/token":
                    assert values["client_id"] == [fixture.client]
                    grant = values["grant_type"][0]
                    if grant == "authorization_code":
                        auth = fixture.codes.pop(values["code"][0])
                        assert values["redirect_uri"] == auth["redirect_uri"]
                        challenge, nonce = auth["code_challenge"][0], auth["nonce"][0]
                    elif grant == "urn:ietf:params:oauth:grant-type:device_code":
                        device = fixture.devices[values["device_code"][0]]
                        if not device["authorized"]:
                            self.answer(400, {"error": "authorization_pending"})
                            return
                        challenge, nonce = device["challenge"], None
                    elif grant == "refresh_token" and refresh_exchange is not None:
                        self.answer(*refresh_exchange(values))
                        return
                    else:
                        self.answer(400, {"error": "unsupported_grant_type"})
                        return
                    assert (
                        encoded(
                            hashlib.sha256(values["code_verifier"][0].encode()).digest()
                        )
                        == challenge
                    )
                    fixture.exchanges.append({"grant": grant, "pkce_verified": True})
                    self.answer(200, fixture.tokens(nonce))
                elif path == "/revoke":
                    assert values["client_id"] == [fixture.client]
                    assert values["token_type_hint"] == ["refresh_token"]
                    assert values["token"][0] in fixture.refresh_tokens
                    fixture.refresh_tokens.remove(values["token"][0])
                    self.answer(200, {})
                elif path == "/v1/responses":
                    if gateway_response is not None:
                        gateway_response(self, json.loads(raw))
                        return
                    token = self.headers.get(
                        "x-portkey-api-key",
                        self.headers.get("Authorization", "").removeprefix("Bearer "),
                    )
                    assert token in fixture.access_tokens, (
                        "Gateway did not receive the original verified access JWT"
                    )
                    body = json.loads(raw)
                    assert (
                        body.get("store") is False
                        and body.get("max_output_tokens") == 16
                    )
                    if fixture.gateway_status != 200:
                        self.answer(
                            fixture.gateway_status,
                            {"error": {"message": "Synthetic gateway denial"}},
                        )
                    else:
                        self.answer(
                            200,
                            {
                                "id": "resp-fixture",
                                "object": "response",
                                "status": "completed",
                                "output": [
                                    {
                                        "type": "message",
                                        "role": "assistant",
                                        "status": "completed",
                                        "content": [
                                            {"type": "output_text", "text": "OK"}
                                        ],
                                    }
                                ],
                            },
                        )
                else:
                    self.answer(404, {"error": "not_found"})

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.issuer = f"https://127.0.0.1:{self.server.server_port}"
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(leaf, tls_key)
        self.server.socket = context.wrap_socket(self.server.socket, server_side=True)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def jwt(self, claims):
        payload = (
            encoded(
                json.dumps(
                    {"alg": "RS256", "kid": self.keys["keys"][0]["kid"], "typ": "JWT"}
                ).encode()
            )
            + "."
            + encoded(json.dumps(claims).encode())
        )
        signature = subprocess.run(
            ["openssl", "dgst", "-sha256", "-sign", str(self.key)],
            input=payload.encode(),
            capture_output=True,
            check=True,
        ).stdout
        return payload + "." + encoded(signature)

    def tokens(self, nonce):
        now = int(time.time())
        claims = {
            "iss": self.issuer,
            "sub": self.subject,
            "iat": now,
            "exp": now + 600,
            "azp": self.client,
        }
        access = self.jwt({**claims, "aud": self.audience})
        identity = {**claims, "aud": self.client, "preferred_username": "fixture-user"}
        if nonce is not None:
            identity["nonce"] = "invalid-fixture-nonce" if self.invalid_nonce else nonce
        self.access_tokens.add(access)
        refresh = "fixture-refresh-" + secrets.token_urlsafe(24)
        self.refresh_tokens.add(refresh)
        return {
            "token_type": "Bearer",
            "expires_in": 600,
            "access_token": access,
            "id_token": self.jwt(identity),
            "refresh_token": refresh,
        }

    def close(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=5)
