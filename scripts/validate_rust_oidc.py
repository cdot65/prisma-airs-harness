#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["requests==2.32.5"]
# ///
"""Drive the Rust browser/device fixture with a temporary Keycloak user.

Requires an owner-only, short-lived operator token file and a built acceptance
example. Authorization URLs, device codes and bearer credentials stay in memory.
Do not run concurrently with other pilot fixtures.
"""

import argparse
from html.parser import HTMLParser
import json
from pathlib import Path
import secrets
import subprocess
from urllib.parse import parse_qs, urljoin, urlsplit

import requests


class Form(HTMLParser):
    def __init__(self, html):
        super().__init__()
        self.action = None
        self.fields = {}
        self.feed(html)

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "form":
            self.action = attrs.get("action")
        if tag == "input" and attrs.get("type") == "hidden" and attrs.get("name"):
            self.fields[attrs["name"]] = attrs.get("value", "")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--admin-token-file", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    assert args.admin_token_file.stat().st_mode & 0o077 == 0
    admin_token = args.admin_token_file.read_text().strip()
    base = "https://auth.dev.cdot.io"
    admin_url = base + "/admin/realms/truffles"

    def admin(path, method="GET", body=None):
        r = requests.request(
            method,
            admin_url + path,
            json=body,
            headers={"Authorization": "Bearer " + admin_token},
            timeout=20,
            allow_redirects=False,
        )
        assert 200 <= r.status_code < 300, f"Operator request failed: {r.status_code}"
        return r.json() if r.content else None

    client = admin("/clients?clientId=airs-terminal-pilot")[0]
    assert not client["enabled"], "Pilot must start disabled"
    username = "airs-rust-oidc-" + secrets.token_hex(8)
    password = secrets.token_urlsafe(32) + "Aa1!"
    user_id = None
    process = None
    rows = []
    try:
        admin(
            "/users",
            "POST",
            {
                "username": username,
                "enabled": True,
                "emailVerified": True,
                "email": username + "@example.invalid",
                "firstName": "Rust",
                "lastName": "Acceptance",
                "credentials": [
                    {"type": "password", "value": password, "temporary": False}
                ],
            },
        )
        user_id = admin("/users?exact=true&username=" + username)[0]["id"]
        admin("/clients/" + client["id"], "PUT", {"enabled": True})
        for device in (False, True):
            process = subprocess.Popen(
                [str(args.binary.resolve()), *(["--device"] if device else [])],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            line = process.stdout.readline(16384)
            if not line:
                _, error = process.communicate(timeout=5)
                raise AssertionError("Rust fixture did not start: " + error[:500])
            instruction = json.loads(line)
            browser = requests.Session()
            url = (
                instruction.get("authorization_url") or instruction["verification_uri"]
            )
            assert urlsplit(url).netloc == urlsplit(base).netloc
            r = browser.get(
                url,
                params={"user_code": instruction["user_code"]} if device else None,
                timeout=20,
                allow_redirects=False,
            )
            for _ in range(10):
                if r.status_code in (302, 303):
                    target = urljoin(r.url, r.headers["Location"])
                    parsed = urlsplit(target)
                    if parsed.hostname == "127.0.0.1" and not device:
                        assert parsed.scheme == "http" and parsed.path == "/callback"
                        r = browser.get(target, timeout=20, allow_redirects=False)
                        assert r.status_code == 200, (
                            "Rust callback rejected: "
                            + json.dumps(
                                {
                                    "status": r.status_code,
                                    "query_fields": sorted(parse_qs(parsed.query)),
                                    "oauth_error": parse_qs(parsed.query).get("error"),
                                }
                            )
                        )
                        break
                    assert parsed.netloc == urlsplit(base).netloc
                    r = browser.get(target, timeout=20, allow_redirects=False)
                    continue
                form = Form(r.text)
                if not form.action:
                    assert device, "Expected identity form"
                    break
                action = urljoin(r.url, form.action)
                assert urlsplit(action).netloc == urlsplit(base).netloc
                fields = {
                    **form.fields,
                    "username": username,
                    "password": password,
                    "accept": "Yes",
                    "user_code": instruction.get("user_code", ""),
                }
                r = browser.post(action, data=fields, timeout=20, allow_redirects=False)
            output, error = process.communicate(timeout=45)
            assert process.returncode == 0, (
                "Rust protocol fixture failed: " + error[:500]
            )
            row = json.loads(output.strip())
            assert row["passed"] and row["subject"] == user_id
            row["flow"] = "device" if device else "browser"
            rows.append(row)
            print(json.dumps(row), flush=True)
            process = None
    finally:
        if process and process.poll() is None:
            process.terminate()
            process.communicate(timeout=5)
        admin("/clients/" + client["id"], "PUT", {"enabled": False})
        if user_id:
            admin("/users/" + user_id, "DELETE")
        removed = not admin("/users?exact=true&username=" + username)
        receipt = {
            "passed": len(rows) == 2 and removed,
            "synthetic_user_removed": removed,
            "pilot_disabled": not admin("/clients/" + client["id"])["enabled"],
            "cases": rows,
        }
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
