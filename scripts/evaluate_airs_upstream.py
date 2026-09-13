#!/usr/bin/env python3
"""Validate the fixed 0.154 acceptance ledger; missing evidence fails closed.

Receipts are review assertions, not cryptographic proof of test execution. This
tool checks their completeness, lineage and integrity; independent review must
still assess whether the linked evidence establishes the claimed behavior.
"""

import argparse
import hashlib
import json
from pathlib import Path
import re


GATES = {
    "G1": ("endpoint_inventory", "inference_boundaries", "optional_features"),
    "G2": ("live_gateway_allow_deny", "scan_correlations", "override_spoof_denials"),
    "G3": (
        "interactive_routes",
        "reasoning_capabilities",
        "tools_and_compaction",
        "doctor_probe",
    ),
    "G4": (
        "workspace_lifecycle",
        "oidc_lifecycle",
        "authority_isolation",
        "concurrent_revocation",
        "redirect_redaction",
        "native_stores",
        "windows_identity",
    ),
    "G5": (
        "live_mcp_scan",
        "mcp_authorization",
        "mcp_refresh_reconnect",
        "mcp_no_replay",
    ),
    "G6": (
        "managed_cli_pins",
        "managed_cli_isolation",
        "eight_skills",
        "agent_cli_workflow",
    ),
    "G7": (
        "old_state_upgrade",
        "resume_fork",
        "worktrees",
        "active_writer",
        "rollback",
    ),
    "G8": (
        "owned_workflows",
        "linux_install",
        "apple_silicon_install",
        "mac_signing",
        "artifact_lineage",
    ),
}
COMPONENTS = {
    "routing": (
        10,
        ("interactive_routes", "inference_boundaries", "reasoning_capabilities"),
    ),
    "scans": (
        10,
        ("live_gateway_allow_deny", "scan_correlations", "override_spoof_denials"),
    ),
    "failure_boundaries": (
        10,
        ("tools_and_compaction", "inference_boundaries", "optional_features"),
    ),
    "identity_lifecycle": (
        10,
        (
            "workspace_lifecycle",
            "oidc_lifecycle",
            "concurrent_revocation",
            "native_stores",
            "windows_identity",
        ),
    ),
    "identity_isolation": (10, ("authority_isolation", "redirect_redaction")),
    "mcp_scanning": (5, ("live_mcp_scan", "mcp_authorization")),
    "mcp_credentials": (5, ("mcp_refresh_reconnect", "authority_isolation")),
    "mcp_safety": (5, ("mcp_no_replay", "mcp_authorization")),
    "managed_invocation": (5, ("managed_cli_pins", "managed_cli_isolation")),
    "skill_contracts": (5, ("eight_skills",)),
    "agent_integration": (5, ("agent_cli_workflow", "doctor_probe")),
    "session_state": (
        5,
        ("old_state_upgrade", "resume_fork", "worktrees", "active_writer", "rollback"),
    ),
    "inline_questions": (5, ("inline_questions",)),
    "maintenance": (
        5,
        ("merge_review", "dependency_schemas", "affected_and_full_tests"),
    ),
    "release": (5, GATES["G8"] + ("rollback",)),
}
CASES = sorted(
    {case for cases in GATES.values() for case in cases}
    | {case for _, cases in COMPONENTS.values() for case in cases}
)


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def evaluate(ledger, root):
    problems = []
    source = ledger.get("source_commit", "")
    artifacts = ledger.get("artifacts", {})
    if not re.fullmatch(r"[a-f0-9]{40}", source):
        problems.append("Missing full candidate source commit")
    if set(artifacts) != {"linux-x64", "darwin-arm64"} or any(
        not re.fullmatch(r"[a-f0-9]{64}", value) for value in artifacts.values()
    ):
        problems.append("Both exact supported native artifact hashes are required")

    def evidence_ok(refs):
        if not refs:
            return False
        for ref in refs:
            path = (root / ref["path"]).resolve()
            if not path.is_relative_to(root.resolve()) or not path.is_file():
                return False
            if digest(path) != ref["sha256"]:
                return False
        return True

    receipts = ledger.get("receipts", {})
    passed = set()
    for case in CASES:
        receipt = receipts.get(case, {})
        valid = (
            receipt.get("status") == "passed"
            and receipt.get("source_commit") == source
            and receipt.get("artifacts") == artifacts
            and all(
                receipt.get(field)
                for field in (
                    "expected",
                    "actual",
                    "invocation",
                    "timestamp",
                    "platform",
                    "configuration_fingerprint",
                    "fixture_provenance",
                    "reviewer_disposition",
                )
            )
            and evidence_ok(receipt.get("evidence"))
        )
        if valid:
            passed.add(case)
        else:
            problems.append(f"Missing, failed, stale or incomplete case: {case}")

    scores = {}
    for component, (maximum, required) in COMPONENTS.items():
        score = ledger.get("scores", {}).get(component, {})
        points = score.get("points", 0)
        if type(points) not in (int, float) or not 0 <= points <= maximum:
            problems.append(f"Invalid score: {component}")
            points = 0
        if not set(required) <= passed or not score.get("reason"):
            points = 0
        scores[component] = points

    for role in ("independent_review", "owner_acceptance"):
        review = ledger.get(role, {})
        if not (
            review.get("status") == "passed"
            and review.get("source_commit") == source
            and review.get("artifacts") == artifacts
            and review.get("reviewer")
            and review.get("reviewer") != ledger.get("implementer")
            and evidence_ok(review.get("evidence"))
        ):
            problems.append(f"Pending {role}")
    if not ledger.get("implementer"):
        problems.append("Missing implementer identity")
    for finding in ledger.get("findings", []):
        if finding.get("status") == "resolved":
            if not evidence_ok(finding.get("resolution_evidence")):
                problems.append(f"Unproved resolution: {finding.get('id')}")
        elif finding.get("airs_boundary") or finding.get("severity") in (
            "critical",
            "high",
        ):
            problems.append(f"Blocking finding: {finding.get('id')}")
        elif not all(
            finding.get(key) for key in ("owner", "disposition", "score_effect")
        ):
            problems.append(f"Untriaged finding: {finding.get('id')}")
    total = sum(scores.values())
    gates = {
        gate: "passed" if set(cases) <= passed else "blocked"
        for gate, cases in GATES.items()
    }
    return {
        "status": "GO" if not problems and total >= 90 else "NO-GO",
        "score": total,
        "maximum": 100,
        "threshold": 90,
        "gates": gates,
        "components": scores,
        "problems": problems,
        "publication_authorized": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ledger", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        result = evaluate(json.loads(args.ledger.read_text()), args.ledger.parent)
    except (KeyError, TypeError, ValueError, OSError) as error:
        result = {"status": "NO-GO", "problems": [f"Invalid ledger: {error}"]}
    output = json.dumps(result, indent=2, allow_nan=False) + "\n"
    if args.output:
        args.output.write_text(output)
    print(output, end="")
    raise SystemExit(0 if result["status"] == "GO" else 1)


if __name__ == "__main__":
    main()
