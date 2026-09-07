#!/usr/bin/env python3
"""Unauthenticated TLS/connectivity checks for the owner's Terminal deployment.

This sends no credentials, user data or inference requests. Passing does not
certify authentication, authorization or model/tool execution.
"""

import argparse
import json
import socket
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

ENDPOINTS = [
    (
        "keycloak-discovery",
        "https://auth.dev.cdot.io/realms/truffles/.well-known/openid-configuration",
        "GET",
        {200},
    ),
    ("gateway-rejects-anonymous", "https://airs.cdot.io/v1/responses", "POST", {401}),
    (
        "mcp-rejects-anonymous",
        "https://mcp-airs.cdot.io/ws-prisma-ff3d74/airs-terminal-runtime-scanner/mcp",
        "POST",
        {401},
    ),
    (
        "management-oauth-reachable",
        "https://auth.apps.paloaltonetworks.com/oauth2/access_token",
        "POST",
        {400, 401},
    ),
]


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, new_url):
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    rows = []
    opener = urllib.request.build_opener(NoRedirect)
    for name, url, method, expected in ENDPOINTS:
        request = urllib.request.Request(
            url, method=method, data=b"" if method == "POST" else None
        )
        status, error = None, None
        try:
            with opener.open(request, timeout=12) as response:
                status = response.status
        except urllib.error.HTTPError as exc:
            status = exc.code
        except (urllib.error.URLError, TimeoutError) as exc:
            error = str(getattr(exc, "reason", exc))[:200]
        rows.append(
            {
                "test": name,
                "status": status,
                "error": error,
                "passed": status in expected,
            }
        )
    receipt = {
        "checked_at": datetime.now(timezone.utc).isoformat(),
        "passed": all(r["passed"] for r in rows),
        "checks": rows,
        "management_addresses": sorted(
            {
                row[4][0]
                for row in socket.getaddrinfo(
                    "auth.apps.paloaltonetworks.com", 443, type=socket.SOCK_STREAM
                )
            }
        ),
    }
    args.output.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt))
    return 0 if receipt["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
