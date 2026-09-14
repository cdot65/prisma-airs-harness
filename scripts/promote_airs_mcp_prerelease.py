#!/usr/bin/env python3
"""Promote tested alpha native packages using bound MCP and npm acceptance receipts.

This is the internal authenticated-MCP alpha channel, not the independently
reviewed release channel. It preserves that distinction in the published bytes.
No score, independent review or full workspace validation is manufactured.
"""

import argparse
import hashlib
import json
from pathlib import Path
import shutil

TOOLS = {
    "list_workspaces",
    "get_workspace",
    "list_gateway_configs",
    "get_gateway_config",
    "list_gateway_guardrails",
    "get_gateway_guardrail",
    "list_security_profiles",
    "get_security_profile",
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate(info, e2e, upgrade, signing):
    if not info["version"].startswith("0.1.0-alpha."):
        raise ValueError("This promotion is limited to the internal alpha channel")
    if (
        e2e.get("passed") is not True
        or e2e.get("binary_sha256") != info["binary_sha256"]
    ):
        raise ValueError("Live MCP acceptance must bind the exact executable")
    expected_platform = {
        "aarch64-apple-darwin": "Darwin",
        "x86_64-unknown-linux-musl": "Linux",
    }[info["target"]]
    if e2e.get("platform") != expected_platform or e2e.get("refresh_cycles") != 2:
        raise ValueError("Both native refresh cycles must pass on the target platform")
    rows = e2e["results"]
    required = {
        "inference_browser_pkce",
        "mcp_browser_pkce",
        "doctor_after_mcp",
        "inference_history_preserved",
        "mcp_logout",
        "mcp_list_after_logout",
        "inference_after_mcp_logout",
        "inference_logout",
    }
    if any(row.get("passed") is not True for row in rows) or not required <= {
        row["case"] for row in rows
    }:
        raise ValueError(
            "Required native login/history/logout acceptance is incomplete"
        )
    tools = next(row for row in rows if row["case"] == "all_read_tools_tool_results")
    if not TOOLS <= set(tools["tools"]):
        raise ValueError("All eight model-selected reads must succeed")
    for cycle in (1, 2):
        for process in (0, 1):
            row = next(
                row
                for row in rows
                if row["case"] == f"refresh_{cycle}_{process}_tool_results"
            )
            if "list_workspaces" not in row["tools"]:
                raise ValueError("Concurrent refreshed process did not call MCP")
    if (
        upgrade.get("passed") is not True
        or upgrade.get("version") != info["version"]
        or upgrade.get("platform") != expected_platform
    ):
        raise ValueError("In-place npm upgrade acceptance does not match")
    if (
        upgrade.get("configuration_preserved") is not True
        or upgrade.get("legacy_target_preserved") is not True
    ):
        raise ValueError(
            "Upgrade must preserve configuration and the legacy executable"
        )
    for case in upgrade["cases"]:
        if (
            case.get("passed") is not True
            or case.get("binary_sha256") != info["binary_sha256"]
            or case.get("uninstall_used")
            or case.get("manual_command_removal")
        ):
            raise ValueError(
                "Upgrade did not verify the exact native bytes without removal"
            )
    managed = next(
        case for case in upgrade["cases"] if case["case"] == "npm-managed-in-place"
    )
    if managed.get("force_used"):
        raise ValueError("An npm-managed upgrade must not require force")
    if expected_platform == "Darwin":
        expected = {
            "binary_sha256": info["binary_sha256"],
            "source_commit": info["source_commit"],
            "team_id": "G5QLZ5A8TA",
            "codesign_verified": True,
            "hardened_runtime": True,
            "notarization_verified": True,
        }
        if signing is None or any(signing.get(k) != v for k, v in expected.items()):
            raise ValueError("Verified Developer ID and notarization are required")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", type=Path, required=True)
    parser.add_argument("--e2e", type=Path, required=True)
    parser.add_argument("--upgrade", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.release.resolve(strict=True)
    info = json.loads((root / "BUILD-INFO.json").read_text())
    if digest(root / "airs-harness") != info["binary_sha256"]:
        raise ValueError("Native bytes changed")
    signing_path = root / "SIGNING.json"
    signing = json.loads(signing_path.read_text()) if signing_path.exists() else None
    e2e = json.loads(args.e2e.read_text())
    upgrade = json.loads(args.upgrade.read_text())
    validate(info, e2e, upgrade, signing)
    shutil.copytree(root, args.output)
    evidence = args.output / "validation-evidence"
    evidence.mkdir(exist_ok=True)
    records = []
    for role, path in [("native-mcp-e2e", args.e2e), ("npm-upgrade", args.upgrade)]:
        destination = evidence / (role + ".json")
        shutil.copyfile(path, destination)
        records.append(
            {
                "role": role,
                "path": str(destination.relative_to(evidence)),
                "sha256": digest(destination),
            }
        )
    validation = {
        "schema_version": 1,
        "scope": "authenticated-mcp-internal-alpha",
        "passed": True,
        "release_ready": True,
        "full_authentication_release_ready": False,
        "independent_review": "not performed; this receipt is measured acceptance, not an independent review",
        "full_workspace_tests": "not claimed",
        "source_commit": info["source_commit"],
        "product_version": info["version"],
        "target": info["target"],
        "binary_sha256": info["binary_sha256"],
        "evidence": records,
    }
    (args.output / "VALIDATION.json").write_text(
        json.dumps(validation, indent=2) + "\n"
    )
    info.pop("publishable", None)
    info.pop("release_status", None)
    info["release_scope"] = validation["scope"]
    info["validation_receipt_sha256"] = digest(args.output / "VALIDATION.json")
    (args.output / "BUILD-INFO.json").write_text(json.dumps(info, indent=2) + "\n")
    (args.output / "SHA256SUMS").write_text(
        "".join(
            f"{digest(p)}  {p.relative_to(args.output).as_posix()}\n"
            for p in sorted(args.output.rglob("*"))
            if p.is_file() and p.name != "SHA256SUMS"
        )
    )
    print(
        json.dumps(
            {
                "passed": True,
                "scope": validation["scope"],
                "binary_sha256": info["binary_sha256"],
            }
        )
    )


if __name__ == "__main__":
    main()
