#!/usr/bin/env python3
"""Probe repeated, concurrent and idle AIRS MCP sessions without hidden retries.

Runs discovery and one benign read-only scan. No gateway configuration changes.
"""

import argparse
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import time
import urllib.request
import uuid

from validate_live_gateway import NoRedirect, request


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--url", required=True)
    parser.add_argument("--credential-file", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--samples", type=int, default=8)
    parser.add_argument("--idle-seconds", type=int, default=120)
    args = parser.parse_args()
    assert args.url.startswith("https://")
    assert 1 <= args.samples <= 30 and 0 <= args.idle_seconds <= 600
    key = args.credential_file.read_text().strip()
    rows = []

    def initialize(label):
        status, headers, payloads = request(
            args.url,
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-03-26",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "airs-harness-session-" + uuid.uuid4().hex[:8],
                        "version": "0.1.0-alpha.4",
                    },
                },
            },
            key,
        )
        row = {"case": label, "initialize_status": status, "passed": False}
        data = payloads[0] if payloads else {}
        if status != 200 or "result" not in data:
            row["error"] = data.get("error")
            return row, None
        session = {
            "Mcp-Session-Id": value
            for name, value in headers.items()
            if name.lower() == "mcp-session-id"
        }
        session["MCP-Protocol-Version"] = data["result"]["protocolVersion"]
        notified, _, _ = request(
            args.url,
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
            key,
            session,
        )
        listed, _, tools = request(
            args.url, {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}, key, session
        )
        names = (
            [tool["name"] for tool in (tools[0].get("result") or {}).get("tools", [])]
            if tools
            else []
        )
        row.update(
            notification_status=notified,
            list_status=listed,
            tools=names,
            passed=notified in (200, 202, 204)
            and listed == 200
            and names == ["pan_inline_scan"],
        )
        return row, session

    def close(session):
        if not session:
            return
        # Session termination is optional in the protocol. Never log the key.
        req = urllib.request.Request(
            args.url, headers={"x-portkey-api-key": key, **session}, method="DELETE"
        )
        try:
            with urllib.request.build_opener(NoRedirect()).open(
                req, timeout=15
            ) as response:
                response.read()
        except Exception:
            pass

    def sample(label):
        row, session = initialize(label)
        close(session)
        print(label, "PASS" if row["passed"] else "FAIL", flush=True)
        return row

    for index in range(args.samples):
        rows.append(sample(f"sequential-{index + 1}"))
    with ThreadPoolExecutor(max_workers=4) as pool:
        rows.extend(
            pool.map(
                sample, [f"concurrent-{index + 1}" for index in range(args.samples)]
            )
        )
    idle, session = initialize("idle-resume")
    if session:
        try:
            print(f"Holding session idle for {args.idle_seconds}s", flush=True)
            time.sleep(args.idle_seconds)
            status, _, payloads = request(
                args.url,
                {
                    "jsonrpc": "2.0",
                    "id": 3,
                    "method": "tools/call",
                    "params": {
                        "name": "pan_inline_scan",
                        "arguments": {
                            "scan_request": {
                                "prompt": "Hello from AIRS Harness",
                                "profile": "Prisma AIRS Terminal",
                                "app_name": "prisma-airs-harness",
                            }
                        },
                    },
                },
                key,
                session,
            )
            result = (payloads[0].get("result") or {}) if payloads else {}
            structured = result.get("structuredContent") or {}
            if not structured:
                for content in result.get("content", []):
                    if content.get("type") == "text":
                        try:
                            structured = json.loads(content["text"])
                            break
                        except (ValueError, KeyError):
                            continue
            scan = structured.get("results") or {}
            idle.update(
                scan_status=status,
                scan_id=scan.get("scan_id"),
                passed=idle["passed"]
                and status == 200
                and not result.get("isError")
                and scan.get("action") == "allow"
                and bool(scan.get("scan_id")),
            )
        finally:
            close(session)
    rows.append(idle)
    args.output.write_text(
        json.dumps(
            {"passed": all(row["passed"] for row in rows), "checks": rows}, indent=2
        )
        + "\n"
    )
    print("PASS" if all(row["passed"] for row in rows) else "FAIL", args.output)
    raise SystemExit(0 if all(row["passed"] for row in rows) else 1)


if __name__ == "__main__":
    main()
