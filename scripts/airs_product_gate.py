#!/usr/bin/env python3
"""Bind native AIRS product acceptance to rollback and separate GNU diagnostics."""

import argparse
from pathlib import Path

from airs_product_workspace import verify_workspace
from airs_release_contract import verify_acceptance_set
from airs_release_receipts import atomic_json, evidence_path
from airs_test_release_spec import (
    TARGETS,
    canonical_digest,
    digest_file,
    load_json,
    require,
    validate_spec,
)


REQUIRED_CASES = {
    "doctor": {
        "test_airs_doctor.SessionDoctor.test_redacted_report_preview_copy_and_save_reuse_one_inspection",
    },
    "installed-regressions": {
        "test_airs_harness_refresh_faults.RefreshRecovery.test_consumed_refresh_response_loss_never_replays_after_restart",
        "test_airs_harness_refresh_faults.RefreshRecovery.test_rejected_refresh_remains_signin_required_after_restart",
    },
}


def product_gate(
    spec,
    acceptance,
    workspace,
    installation="candidate",
    verification_tooling_commit=None,
):
    spec = validate_spec(spec)
    require(
        spec["previous_version"] == "0.1.1",
        "This product gate requires the reviewed stable 0.1.1 baseline",
    )
    native = verify_acceptance_set(
        spec, acceptance, installation, verification_tooling_commit
    )
    checks, rollback = [], []
    for target in TARGETS:
        root = Path(acceptance) / target
        for stage, required in REQUIRED_CASES.items():
            path = evidence_path(root, f"files/{stage}/TEST-RESULTS.json")
            result = load_json(path)
            skipped = {row["test"] for row in result["skipped"]}
            require(
                required <= set(result["test_ids"]) and not required & skipped,
                "Required report and refresh recovery tests must execute on every native target",
            )
            checks.append(
                {
                    "target": target,
                    "stage": stage,
                    "required_executed": sorted(required),
                    "tests_run": result["tests_run"],
                    "skipped": len(skipped),
                    "result_sha256": digest_file(path),
                }
            )
        path = evidence_path(root, "files/upgrade/UPGRADE.json")
        result = load_json(path)
        roundtrip = result.get("roundtrip")
        require(
            isinstance(roundtrip, dict),
            "Native credential/history rollback evidence is required",
        )
        # The shared acceptance authority has already verified every preservation
        # flag and previous/candidate/restored native, source and version binding.
        candidate = next(
            row["binary_sha256"] for row in spec["platforms"] if row["target"] == target
        )
        baseline = next(
            row["binary_sha256"]
            for row in spec["previous_release"]["platforms"]
            if row["target"] == target
        )
        rollback.append(
            {
                "target": target,
                "result_sha256": digest_file(path),
                "previous_version": spec["previous_version"],
                "candidate_version": spec["version"],
                "previous_binary_sha256": baseline,
                "candidate_binary_sha256": candidate,
                "restored_binary_sha256": baseline,
                "real_mcp_turns": roundtrip["real_mcp_turns"],
                "configuration_history_credentials_preserved": True,
                "native_cleanup_completed": True,
            }
        )
    diagnostics = verify_workspace(Path(workspace), spec)
    return {
        "schema_version": 1,
        "scope": "airs-product-acceptance",
        "phase": installation,
        "passed": True,
        "version": spec["version"],
        "source_commit": spec["source_commit"],
        "tooling_commit": spec["tooling_commit"],
        "verification_tooling_commit": verification_tooling_commit,
        "packaging_commit": spec["packaging_commit"],
        "spec_sha256": canonical_digest(spec),
        "native_acceptance": native,
        "behavioral_checks": checks,
        "upgrade_rollback": rollback,
        "workspace_diagnostics": diagnostics,
        "unresolved_product_blockers": [],
        "production_sso_servicenow_claimed": False,
        "stable_promotion_authorized": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, required=True)
    parser.add_argument("--acceptance", type=Path, required=True)
    parser.add_argument("--workspace", type=Path, required=True)
    parser.add_argument(
        "--installation", choices=("candidate", "registry"), default="candidate"
    )
    parser.add_argument("--verification-tooling-commit")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = product_gate(
        load_json(args.spec),
        args.acceptance,
        args.workspace,
        args.installation,
        args.verification_tooling_commit,
    )
    atomic_json(args.output, result)
    print(
        f"AIRS product acceptance passed for {result['version']}; full workspace: {result['workspace_diagnostics']['status']}"
    )


if __name__ == "__main__":
    main()
