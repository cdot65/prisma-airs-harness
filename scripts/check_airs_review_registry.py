#!/usr/bin/env python3
"""Read GitHub package metadata without publishing or disclosing credentials."""

import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import re
from urllib.error import HTTPError
from urllib.parse import quote
from urllib.request import HTTPRedirectHandler, Request, build_opener

PACKAGES = (
    "@cdot65/prisma-airs-harness",
    "@cdot65/prisma-airs-harness-linux-x64",
    "@cdot65/prisma-airs-harness-darwin-arm64",
)


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *_args, **_kwargs):
        return None


def inspect(version, token):
    if not re.fullmatch(r"0\.1\.0-alpha\.[1-9][0-9]*", version):
        raise ValueError("Expected an explicit alpha prerelease version")
    if not token:
        raise ValueError("GitHub Packages read credential is required")
    opener = build_opener(NoRedirect())
    records = []
    for name in PACKAGES:
        record = {"name": name}
        request = Request(
            "https://npm.pkg.github.com/" + quote(name, safe="@"),
            headers={"Authorization": "Bearer " + token, "Accept": "application/json"},
        )
        try:
            with opener.open(request, timeout=20) as response:
                raw = response.read(2 * 1024 * 1024 + 1)
                if len(raw) > 2 * 1024 * 1024:
                    raise ValueError("Registry metadata exceeds limit")
                document = json.loads(raw)
                if document.get("name") != name or not isinstance(
                    document.get("versions"), dict
                ):
                    raise ValueError("Registry package identity or versions differ")
                record.update(
                    {
                        "http_status": response.status,
                        "published_version_count": len(document["versions"]),
                        "published_alpha9_present": "0.1.0-alpha.9"
                        in document["versions"],
                        "candidate_version_present": version in document["versions"],
                    }
                )
        except HTTPError as error:
            record["http_status"] = error.code
            error.close()
        records.append(record)
    return {
        "recorded_at_utc": datetime.now(timezone.utc).isoformat(),
        "candidate_version": version,
        "packages": records,
        "passed": all(
            r.get("http_status") == 200
            and r.get("published_alpha9_present")
            and r.get("candidate_version_present") is False
            for r in records
        ),
        "published": False,
        "version_reserved": False,
        "teammate_access_tested": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    result = inspect(args.version, os.environ.get("NODE_AUTH_TOKEN"))
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))
    raise SystemExit(0 if result["passed"] else 1)


if __name__ == "__main__":
    main()
