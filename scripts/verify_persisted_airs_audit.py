#!/usr/bin/env python3
"""Correlate only synthetic Terminal requests with persisted AIRS security logs.

Operator-only: requires kubectl access to the existing Elasticsearch credential
and a local port-forward: kubectl -n elk port-forward service/elasticsearch-master
19203:9200. Credentials remain in memory and are sent only over that local tunnel.
No prompts, tokens, raw logs or unrelated user records are emitted.
"""

import argparse
import base64
import json
import subprocess
import urllib.request
import uuid
from datetime import datetime, timezone
from pathlib import Path


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, message, headers, new_url):
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--expectations", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    expected = json.loads(args.expectations.read_text())
    assert 2 <= len(expected) <= 5, "Expected two to five synthetic requests"
    for row in expected:
        uuid.UUID(row["subject"])
        uuid.UUID(row["trace_id"])
    raw = subprocess.check_output(
        [
            "kubectl",
            "get",
            "secret",
            "-n",
            "elk",
            "elasticsearch-master-credentials",
            "-o",
            "json",
        ]
    )
    password = base64.b64decode(json.loads(raw)["data"]["password"]).decode()
    query = {
        "size": 200,
        "query": {
            "bool": {
                "should": [
                    {"match_phrase": {"SessionID": row["trace_id"]}} for row in expected
                ],
                "minimum_should_match": 1,
            }
        },
        "_source": [
            "AIApplicationUserName",
            "SessionID",
            "ScanID",
            "Action",
            "AISecurityProfileName",
            "IsPrompt",
            "IsResponse",
        ],
    }
    request = urllib.request.Request(
        "http://127.0.0.1:19203/airs-logs-*/_search",
        data=json.dumps(query).encode(),
        headers={
            "Authorization": "Basic "
            + base64.b64encode(("elastic:" + password).encode()).decode(),
            "Content-Type": "application/json",
        },
    )
    with urllib.request.build_opener(NoRedirect).open(request, timeout=20) as response:
        data = json.loads(response.read(512 * 1024))
    records = [row["_source"] for row in data["hits"]["hits"]]
    checks = []
    for wanted in expected:
        rows = [r for r in records if r.get("SessionID") == wanted["trace_id"]]
        scans = sorted({r["ScanID"] for r in rows if r.get("ScanID")})
        directions = {
            "input"
            if r.get("IsPrompt") is True
            else "output"
            if r.get("IsResponse") is True
            else "unknown"
            for r in rows
        }
        passed = (
            len(scans) >= 2
            and directions == {"input", "output"}
            and all(
                r.get("AIApplicationUserName") == wanted["subject"]
                and r.get("AISecurityProfileName") == "Prisma AIRS Terminal"
                and r.get("Action") == "allow"
                for r in rows
            )
        )
        checks.append(
            {
                **wanted,
                "passed": passed,
                "persisted_records": len(rows),
                "scan_ids": scans,
                "both_input_and_output_observed": directions == {"input", "output"},
            }
        )
    receipt = {
        "checked_at": datetime.now(timezone.utc).isoformat(),
        "passed": all(row["passed"] for row in checks),
        "source": "Existing AIRS security-log export persisted in Elasticsearch airs-logs-*",
        "attribution_field": "AIApplicationUserName",
        "correlation_field": "SessionID",
        "scope": "Signed-user attribution in persisted input/output security scans; does not certify the management usage/cost dashboard",
        "checks": checks,
    }
    args.output.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt))
    return 0 if receipt["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
