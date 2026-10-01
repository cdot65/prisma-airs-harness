#!/usr/bin/env python3
"""Run approved public npm publication/promotion on the native release runner.

The owner stages a hash-bound release directory and a temporary private npmrc.
Manual Forgejo dispatch supplies the reviewed plan digest, never a credential.
Every invocation verifies retained native evidence before registry mutation.
"""

import argparse
import json
from pathlib import Path
import subprocess
import sys

from airs_release_contract import verify_acceptance_set
from airs_release_receipts import safe_destination, verify_tooling
from airs_test_release_publish import publish_packages
from airs_test_release_spec import (
    canonical_digest,
    digest_file,
    load_json,
    load_spec,
    require,
)
from airs_test_release_stage import verify_staged
from promote_airs_stable import StableRegistry, promote, validate_readiness

REPO = Path(__file__).resolve().parent.parent
ROOT = Path.home() / ".cache/airs-016-20261001/release/public"


def verify_plan(root, approved_digest, mode):
    plan = load_json(root / "WORKFLOW-PLAN.json")
    require(
        digest_file(root / "WORKFLOW-PLAN.json") == approved_digest,
        "Workflow plan approval mismatch",
    )
    require(
        plan.get("mode") == mode and mode in ("publish", "promote"),
        "Workflow plan mode mismatch",
    )
    require(
        plan.get("registry") == "https://registry.npmjs.org"
        and plan.get("version") == "0.1.6",
        "Unexpected public release",
    )
    expected = {
        "SPEC.json",
        "READINESS.json",
        "staged-npm/NPM-PACKAGES.json",
        "PAYLOAD-EQUIVALENCE.json",
    }
    require(set(plan.get("files", {})) == expected, "Incomplete workflow inputs")
    for name, digest in plan["files"].items():
        require(
            digest_file(safe_destination(root / name)) == digest,
            "Workflow input changed: " + name,
        )
    spec = load_spec(root / "SPEC.json")
    require(
        spec["registry"] == plan["registry"] and spec["version"] == plan["version"],
        "Workflow spec mismatch",
    )
    readiness = load_json(root / "READINESS.json")
    validate_readiness(spec, readiness)
    require(
        canonical_digest(
            verify_acceptance_set(spec, root / "candidate-evidence", "candidate")
        )
        == plan["candidate_acceptance_sha256"],
        "Candidate evidence changed",
    )
    if mode == "promote":
        require(
            canonical_digest(
                verify_acceptance_set(spec, root / "registry-evidence", "registry")
            )
            == plan["registry_acceptance_sha256"],
            "Registry evidence changed",
        )
    verify_staged(spec, root / "staged-npm", root / "candidate-evidence")
    tooling = root / "frozen-tooling"
    verify_tooling(
        tooling, load_json(tooling / "ACCEPTANCE-TOOLING.json"), spec["tooling_commit"]
    )
    commit = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=REPO, text=True
    ).strip()
    require(
        commit == spec["tooling_commit"],
        "Workflow checkout differs from reviewed tooling",
    )
    for name, digest in load_json(tooling / "ACCEPTANCE-TOOLING.json")["files"].items():
        require(digest_file(REPO / name) == digest, "Workflow tooling changed")
    subprocess.run(
        [sys.executable, str(REPO / "validation/2026-10-01/private-0.1.6/audit.py")],
        check=True,
        cwd=REPO,
    )
    require(
        plan.get("production_sso_claimed") is False, "Unattended release scope required"
    )
    return spec, readiness


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=("publish", "promote"), required=True)
    parser.add_argument("--approved-plan-sha256", required=True)
    args = parser.parse_args()
    spec, readiness = verify_plan(ROOT, args.approved_plan_sha256, args.mode)
    config = ROOT / "npmrc"
    output = ROOT / ("workflow-" + args.mode)
    if args.mode == "publish":
        receipt = publish_packages(
            spec, ROOT / "staged-npm", ROOT / "candidate-evidence", output, config
        )
        require(receipt["published"], "Incomplete public publication")
    else:
        verification = {
            **verify_acceptance_set(spec, ROOT / "registry-evidence", "registry"),
            "installation": "registry",
        }
        output.mkdir(parents=True, exist_ok=True)
        receipt = promote(
            spec,
            verify_staged(spec, ROOT / "staged-npm", ROOT / "candidate-evidence"),
            verification,
            readiness,
            output,
            StableRegistry(spec["registry"], config, output),
        )
        require(receipt["complete"], "Incomplete public promotion")
    print(json.dumps({"mode": args.mode, "version": spec["version"], "passed": True}))


if __name__ == "__main__":
    main()
