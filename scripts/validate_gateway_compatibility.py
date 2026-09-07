#!/usr/bin/env python3
"""Live model-specific compatibility checks; credentials and prompts are not logged."""

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

from validate_live_gateway import request


def scan_results(response):
    hooks = response.get("hook_results", {})
    return {
        phase: [
            {
                "guardrail": hook.get("id"),
                "verdict": hook.get("verdict"),
                "scan_ids": [
                    (check.get("data") or {}).get("scan_id")
                    for check in hook.get("checks", [])
                ],
            }
            for hook in hooks.get(phase, [])
        ]
        for phase in ("before_request_hooks", "after_request_hooks")
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--gateway-url", required=True)
    parser.add_argument("--credential-file", type=Path, required=True)
    parser.add_argument("--reasoning-model", default="@openai/gpt-5-mini")
    parser.add_argument("--guardrail", default="pg-prisma-ff3021")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if not args.gateway_url.startswith("https://"):
        parser.error("gateway URL must use HTTPS")
    key = args.credential_file.read_text().strip()
    base = {
        "input": "Reply with exactly: reasoning.effort remains prompt text",
        "max_output_tokens": 512,
        "stream": False,
        "reasoning": {"effort": "low"},
    }
    cases = [
        ("default_drops_reasoning", base, 200, "drop"),
        (
            "explicit_gpt41_drops_reasoning",
            {**base, "model": "@openai/gpt-4.1"},
            200,
            "drop",
        ),
        (
            "capable_model_preserves_low",
            {**base, "model": args.reasoning_model},
            200,
            "preserve",
        ),
        (
            "capable_model_rejects_invalid_effort",
            {
                **base,
                "model": args.reasoning_model,
                "reasoning": {"effort": "airs-invalid-effort"},
            },
            400,
            "invalid",
        ),
    ]
    # A root-only removal must not damage identically named tool-schema fields.
    tool = {
        "type": "function",
        "name": "compatibility_probe",
        "strict": True,
        "description": "Return the supplied values unchanged.",
        "parameters": {
            "type": "object",
            "properties": {
                "reasoning": {
                    "type": "object",
                    "properties": {"effort": {"type": "string"}},
                    "required": ["effort"],
                    "additionalProperties": False,
                }
            },
            "required": ["reasoning"],
            "additionalProperties": False,
        },
    }
    for route in (None, "@openai/gpt-4.1"):
        body = {**base, "tools": [tool], "tool_choice": "none"}
        if route:
            body["model"] = route
        cases.append(
            (
                "nested_schema_" + ("explicit" if route else "default"),
                body,
                200,
                "schema",
            )
        )
    results = []
    for name, body, expected_status, kind in cases:
        row = {
            "case": name,
            "client_model_present": "model" in body,
            "route": body.get("model"),
            "expected_status": expected_status,
        }
        try:
            status, _, data = request(
                args.gateway_url.rstrip("/") + "/responses", body, key
            )
            response = data[-1] if data else {}
            scans = scan_results(response)
            passed = status == expected_status
            if status == 200:
                passed &= all(
                    any(
                        h["guardrail"] == args.guardrail
                        and h["verdict"] is True
                        and h["scan_ids"]
                        and all(h["scan_ids"])
                        for h in values
                    )
                    for values in scans.values()
                )
                passed &= response.get("status") == "completed"
                if kind in ("drop", "preserve", "schema"):
                    output = "".join(
                        c.get("text", "")
                        for item in response.get("output", [])
                        for c in item.get("content", [])
                    )
                    passed &= "reasoning.effort remains prompt text" in output
                if kind == "preserve":
                    passed &= response.get("reasoning", {}).get("effort") == "low"
            if kind == "invalid":
                passed &= response.get("error", {}).get("param") == "reasoning.effort"
            row.update(
                status=status,
                passed=bool(passed),
                response_id=response.get("id"),
                resolved_model=response.get("model"),
                reasoning=response.get("reasoning"),
                error_code=(response.get("error") or {}).get("code"),
                scans=scans,
            )
        except Exception as exc:
            # Exception strings can carry provider data; retain only the class.
            row.update(passed=False, exception=type(exc).__name__)
        results.append(row)
        print(json.dumps(row), flush=True)
    receipt = {
        "checked_at": datetime.now(timezone.utc).isoformat(),
        "cases": results,
        "passed": all(row["passed"] for row in results),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(receipt, indent=2) + "\n")
    return 0 if receipt["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
