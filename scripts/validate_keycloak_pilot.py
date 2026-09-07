#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["requests==2.32.5", "PyJWT[crypto]==2.10.1"]
# ///
"""Operator acceptance of Keycloak/AIRS using two temporary users; secrets stay in memory."""

import argparse
import base64
import hashlib
import json
import secrets
import time
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import parse_qs, urljoin, urlsplit

import jwt
import requests


class Form(HTMLParser):
    def __init__(self, text, form_id="kc-form-login"):
        super().__init__()
        self.form_id = form_id
        self.action = None
        self.fields = {}
        self.feed(text)

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if tag == "form" and (self.form_id is None or attrs.get("id") == self.form_id):
            self.action = attrs.get("action")
        if tag == "input" and attrs.get("type") == "hidden" and attrs.get("name"):
            self.fields[attrs["name"]] = attrs.get("value", "")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--admin-token-file", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--gateway-url", default="https://airs.cdot.io/v1")
    args = parser.parse_args()
    metadata = args.admin_token_file.stat()
    assert metadata.st_mode & 0o077 == 0, "Admin token file must be owner-only"
    admin_token = args.admin_token_file.read_text().strip()
    issuer = "https://auth.dev.cdot.io/realms/truffles"
    client_name = "airs-terminal-pilot"
    admin_url = "https://auth.dev.cdot.io/admin/realms/truffles"
    redirect = "http://127.0.0.1:45197/callback"
    discovery = requests.get(
        issuer + "/.well-known/openid-configuration", timeout=15
    ).json()
    assert discovery["issuer"] == issuer
    jwks = jwt.PyJWKSet.from_dict(
        requests.get(discovery["jwks_uri"], timeout=15).json()
    )
    users = []
    enabled_client = None
    rows = []

    def admin(path, method="GET", body=None):
        r = requests.request(
            method,
            admin_url + path,
            json=body,
            headers={"Authorization": "Bearer " + admin_token},
            timeout=20,
            allow_redirects=False,
        )
        assert 200 <= r.status_code < 300, (
            f"Admin {method} failed: HTTP {r.status_code}"
        )
        return r.json() if r.content else None

    def record(case, passed, **details):
        row = {"case": case, "passed": passed, **details}
        rows.append(row)
        print(json.dumps(row), flush=True)

    def token_request(data):
        return requests.post(
            discovery["token_endpoint"], data=data, timeout=20, allow_redirects=False
        )

    def claims(token, audience):
        key = jwks[jwt.get_unverified_header(token)["kid"]]
        return jwt.decode(
            token,
            key.key,
            algorithms=["RS256"],
            issuer=issuer,
            audience=audience,
            options={"require": ["iss", "aud", "sub", "exp", "iat"]},
        )

    def code_login(user, bad_verifier=False):
        verifier = secrets.token_urlsafe(48)
        challenge = (
            base64.urlsafe_b64encode(hashlib.sha256(verifier.encode()).digest())
            .decode()
            .rstrip("=")
        )
        state, nonce = secrets.token_urlsafe(32), secrets.token_urlsafe(32)
        params = {
            "client_id": client_name,
            "redirect_uri": redirect,
            "response_type": "code",
            "scope": "openid",
            "state": state,
            "nonce": nonce,
            "code_challenge": challenge,
            "code_challenge_method": "S256",
        }
        session = requests.Session()
        page = session.get(
            discovery["authorization_endpoint"],
            params=params,
            timeout=20,
            allow_redirects=False,
        )
        assert page.status_code == 200, f"Authorization failed: HTTP {page.status_code}"
        form = Form(page.text)
        assert (
            form.action and urlsplit(form.action).netloc == urlsplit(issuer).netloc
        ), "Unexpected login form"
        response = session.post(
            form.action,
            data={
                **form.fields,
                "username": user["username"],
                "password": user["password"],
            },
            timeout=20,
            allow_redirects=False,
        )
        assert response.status_code == 302, (
            f"Browser login failed: HTTP {response.status_code}"
        )
        callback = urlsplit(response.headers["Location"])
        expected = urlsplit(redirect)
        assert (callback.scheme, callback.netloc, callback.path) == (
            expected.scheme,
            expected.netloc,
            expected.path,
        )
        query = parse_qs(callback.query, strict_parsing=True)
        assert query.get("state") == [state] and query.get("iss") == [issuer], (
            "Invalid callback binding"
        )
        token_response = session.post(
            discovery["token_endpoint"],
            data={
                "grant_type": "authorization_code",
                "client_id": client_name,
                "code": query["code"][0],
                "redirect_uri": redirect,
                "code_verifier": secrets.token_urlsafe(48)
                if bad_verifier
                else verifier,
            },
            timeout=20,
            allow_redirects=False,
        )
        if bad_verifier:
            return token_response
        assert token_response.status_code == 200, (
            f"Token exchange failed: HTTP {token_response.status_code}"
        )
        tokens = token_response.json()
        identity = claims(tokens["id_token"], client_name)
        access = claims(tokens["access_token"], "airs-terminal-inference")
        assert (
            identity["nonce"] == nonce
            and identity["sub"] == access["sub"] == user["id"]
        )
        assert (
            access["azp"] == client_name
            and access["portkey_workspace"] == "ws-prisma-ff3d74"
        )
        assert access["exp"] - access["iat"] <= 120
        user["browser"] = session
        return tokens, access

    def infer(case, tokens, expected, extra=None, body=None):
        r = requests.post(
            args.gateway_url.rstrip("/") + "/responses",
            json=body
            or {
                "input": "Reply exactly HEALTHY.",
                "max_output_tokens": 50,
                "stream": False,
            },
            headers={"x-portkey-api-key": tokens["access_token"], **(extra or {})},
            timeout=90,
            allow_redirects=False,
        )
        d = r.json()
        hooks = d.get("hook_results", {})
        phases = {
            phase: [
                {
                    "id": h.get("id"),
                    "verdict": h.get("verdict"),
                    "scan_ids": [
                        (c.get("data") or {}).get("scan_id")
                        for c in h.get("checks", [])
                        if (c.get("data") or {}).get("scan_id")
                    ],
                }
                for h in hooks.get(phase, [])
            ]
            for phase in ["before_request_hooks", "after_request_hooks"]
        }
        checks_pass = (
            r.status_code in expected
            if isinstance(expected, tuple)
            else r.status_code == expected
        )
        if expected == 200:
            checks_pass &= any(
                h["id"] == "pg-termin-da0e73" and h["verdict"]
                for h in phases["before_request_hooks"]
            )
            checks_pass &= all(
                any(h["scan_ids"] and h["verdict"] for h in phases[p]) for p in phases
            )
        record(
            case,
            checks_pass,
            status=r.status_code,
            expected_status=expected,
            response_id=d.get("id"),
            hooks=phases,
        )
        return d

    try:
        client = admin("/clients?clientId=" + client_name)[0]
        assert client["publicClient"] and not client["directAccessGrantsEnabled"]
        assert client["attributes"]["pkce.code.challenge.method"] == "S256"
        role = admin("/clients/" + client["id"] + "/roles/terminal-user")
        assert not client["enabled"], "The acceptance pilot must start disabled"
        assert not admin("/clients/" + client["id"] + "/roles/terminal-user/users"), (
            "Pilot already has users"
        )
        admin("/clients/" + client["id"], "PUT", {"enabled": True})
        enabled_client = client["id"]
        for index in range(2):
            username = "airs-terminal-validation-" + secrets.token_hex(6)
            password = secrets.token_urlsafe(32) + "Aa1!"
            admin(
                "/users",
                "POST",
                {
                    "username": username,
                    "enabled": True,
                    "emailVerified": True,
                    "email": username + "@example.invalid",
                    "firstName": "Terminal",
                    "lastName": "Validation",
                    "credentials": [
                        {"type": "password", "value": password, "temporary": False}
                    ],
                },
            )
            user = admin("/users?exact=true&username=" + username)[0]
            users.append({"id": user["id"], "username": username, "password": password})
        group = admin("/group-by-path/stacks/airs-terminal/users")
        mapped = admin(
            "/groups/" + group["id"] + "/role-mappings/clients/" + client["id"]
        )
        assert [r["id"] for r in mapped] == [role["id"]], (
            "Unexpected stack role mapping"
        )
        grant_path = "/users/" + users[0]["id"] + "/groups/" + group["id"]
        admin(grant_path, "PUT")
        approved, approved_claims = code_login(users[0])
        denied, denied_claims = code_login(users[1])
        record(
            "two_user_pkce_identity",
            approved_claims["sub"] != denied_claims["sub"]
            and "terminal-user" in approved_claims.get("airs_roles", [])
            and "terminal-user" not in denied_claims.get("airs_roles", []),
            subjects=[approved_claims["sub"], denied_claims["sub"]],
            access_token_lifetime_seconds=approved_claims["exp"]
            - approved_claims["iat"],
        )
        infer("authorized_user_default", approved, 200)
        infer("unassigned_user_denied", denied, 446)
        infer(
            "authorized_user_inline_config_denied",
            approved,
            403,
            {
                "x-portkey-config": json.dumps(
                    {
                        "provider": "@openai-terminal-auth",
                        "override_params": {"model": "gpt-4.1"},
                    }
                )
            },
        )
        infer(
            "unassigned_header_role_spoof_denied",
            denied,
            403,
            {
                "x-portkey-metadata": json.dumps(
                    {"airs_roles": ["terminal-user"], "_user": users[0]["id"]}
                ),
                "x-portkey-config": json.dumps(
                    {
                        "provider": "@openai-terminal-auth",
                        "override_params": {"model": "gpt-4.1"},
                        "input_guardrails": [],
                        "output_guardrails": [],
                    }
                ),
            },
        )
        infer(
            "unassigned_nitro_mode_denied",
            denied,
            (400, 403, 446),
            {"x-portkey-nitro-mode": "true"},
        )
        device_verifier = secrets.token_urlsafe(48)
        device_challenge = (
            base64.urlsafe_b64encode(hashlib.sha256(device_verifier.encode()).digest())
            .decode()
            .rstrip("=")
        )
        device = requests.post(
            discovery["device_authorization_endpoint"],
            data={
                "client_id": client_name,
                "scope": "openid",
                "code_challenge": device_challenge,
                "code_challenge_method": "S256",
            },
            timeout=20,
            allow_redirects=False,
        )
        assert device.status_code == 200, "Device authorization unavailable"
        authorization = device.json()
        pending = token_request(
            {
                "client_id": client_name,
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                "device_code": authorization["device_code"],
                "code_verifier": device_verifier,
            }
        )
        record(
            "device_requires_user_approval",
            pending.status_code == 400
            and pending.json().get("error") == "authorization_pending",
            status=pending.status_code,
        )
        verification = users[0]["browser"].get(
            authorization["verification_uri_complete"],
            timeout=20,
            allow_redirects=False,
        )
        for _ in range(5):
            if verification.status_code not in (302, 303):
                break
            location = verification.headers["Location"]
            assert urlsplit(location).netloc == urlsplit(issuer).netloc
            verification = users[0]["browser"].get(
                location, timeout=20, allow_redirects=False
            )
        form = Form(verification.text, form_id=None)
        assert form.action, "Device consent form missing"
        consent_url = urljoin(verification.url, form.action)
        assert urlsplit(consent_url).netloc == urlsplit(issuer).netloc
        consent = users[0]["browser"].post(
            consent_url,
            data={**form.fields, "accept": "Yes"},
            timeout=20,
            allow_redirects=False,
        )
        assert consent.status_code in (200, 302, 303), "Device consent failed"
        time.sleep(authorization.get("interval", 5))
        device_tokens = token_request(
            {
                "client_id": client_name,
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                "device_code": authorization["device_code"],
                "code_verifier": device_verifier,
            }
        )
        assert device_tokens.status_code == 200, (
            f"Device exchange failed: HTTP {device_tokens.status_code}"
        )
        device_identity = claims(device_tokens.json()["id_token"], client_name)
        device_access = claims(
            device_tokens.json()["access_token"], "airs-terminal-inference"
        )
        record(
            "device_user_identity",
            device_identity["sub"] == device_access["sub"] == users[0]["id"],
        )
        infer("device_user_inference_scanned", device_tokens.json(), 200)
        bad = code_login(users[0], bad_verifier=True)
        record(
            "wrong_pkce_verifier_denied",
            bad.status_code == 400 and bad.json().get("error") == "invalid_grant",
            status=bad.status_code,
        )
        refresh = token_request(
            {
                "client_id": client_name,
                "grant_type": "refresh_token",
                "refresh_token": approved["refresh_token"],
            }
        )
        assert refresh.status_code == 200, "Refresh failed"
        refreshed = refresh.json()
        record(
            "refresh_preserves_subject",
            claims(refreshed["access_token"], "airs-terminal-inference")["sub"]
            == users[0]["id"],
        )
        replay = token_request(
            {
                "client_id": client_name,
                "grant_type": "refresh_token",
                "refresh_token": approved["refresh_token"],
            }
        )
        record(
            "refresh_reuse_denied", replay.status_code == 400, status=replay.status_code
        )
        # Replay detection detaches the compromised client session. Use a fresh
        # browser login to test role removal independently of that revocation.
        refreshed, _ = code_login(users[0])
        admin(grant_path, "DELETE")
        revoked = token_request(
            {
                "client_id": client_name,
                "grant_type": "refresh_token",
                "refresh_token": refreshed["refresh_token"],
            }
        )
        assert revoked.status_code == 200, "Role-change refresh failed"
        infer("role_removed_refresh_denied", revoked.json(), 446)
        logout = requests.post(
            discovery["end_session_endpoint"],
            data={
                "client_id": client_name,
                "refresh_token": revoked.json()["refresh_token"],
            },
            timeout=20,
            allow_redirects=False,
        )
        assert logout.status_code == 204, "Logout failed"
        after_logout = token_request(
            {
                "client_id": client_name,
                "grant_type": "refresh_token",
                "refresh_token": revoked.json()["refresh_token"],
            }
        )
        record(
            "logout_prevents_refresh",
            after_logout.status_code == 400,
            status=after_logout.status_code,
        )
    finally:
        if enabled_client:
            admin("/clients/" + enabled_client, "PUT", {"enabled": False})
        removed = []
        for user in users:
            admin("/users/" + user["id"], "DELETE")
            removed.append(not admin("/users?exact=true&username=" + user["username"]))
        report = {
            "checked_at": datetime.now(timezone.utc).isoformat(),
            "purpose": "Public-client IdP and native gateway checks; not installed CLI acceptance",
            "synthetic_users_removed": len(removed) == 2 and all(removed),
            "pilot_client_disabled": enabled_client is not None
            and not admin("/clients/" + enabled_client)["enabled"],
            "passed": len(rows) == 14
            and len(removed) == 2
            and all(removed)
            and all(row["passed"] for row in rows),
            "cases": rows,
        }
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
