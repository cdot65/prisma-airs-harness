"""Public release authorization and workflow inputs fail closed before mutation."""

import copy
import json
from pathlib import Path
import tempfile
import unittest

from airs_forgejo_public_npm import verify_plan
from airs_test_release_spec import digest_file
from promote_airs_stable import (
    WORKSPACE_SOURCE_016,
    WORKSPACE_BASELINE_016,
    validate_readiness,
)


class PublicWorkflowTests(unittest.TestCase):
    def readiness(self):
        spec = {
            "scope": "owner-authorized-stable",
            "version": "0.1.6",
            "source_commit": WORKSPACE_SOURCE_016,
        }
        readiness = {
            "version": "0.1.6",
            "source_commit": WORKSPACE_SOURCE_016,
            "workspace": {
                "scope": "full-workspace",
                "source_commit": WORKSPACE_SOURCE_016,
                "passed": 19097,
                "failed": 3,
                "failures": list(WORKSPACE_BASELINE_016),
                "baseline_review": {
                    "source_commit": WORKSPACE_SOURCE_016,
                    "upstream_revision": "rust-v0.154.0",
                    "unresolved_release_blockers": [],
                    "selected_tests_and_cli_gate_unchanged": True,
                    "assertions_match_baseline": True,
                    "cases": {
                        name: {
                            "disposition": disposition,
                            "evidence_verified": True,
                            "evidence_sha256": "a" * 64,
                            "installed_command_rejected": True,
                        }
                        for name, disposition in WORKSPACE_BASELINE_016.items()
                    },
                },
            },
            "release_authorization": {
                "version": "0.1.6",
                "source_commit": WORKSPACE_SOURCE_016,
                "explicit_stable_publication": True,
                "attended_acceptance_claimed": False,
                "runtime_unchanged_from_requested_alpha": True,
                "previous_alpha": "0.1.6",
                "reviewed_changes": None,
                "evidence_sha256": "b" * 64,
            },
        }
        return spec, readiness

    def test_exact_native_runtime_authorization_and_inherited_failures(self):
        spec, readiness = self.readiness()
        validate_readiness(spec, readiness)
        for change in ["source", "unknown-failure", "attended", "unauthorized"]:
            changed = copy.deepcopy(readiness)
            if change == "source":
                changed["source_commit"] = "c" * 40
            if change == "unknown-failure":
                changed["workspace"]["failures"][0] = "new regression"
            if change == "attended":
                changed["release_authorization"]["attended_acceptance_claimed"] = True
            if change == "unauthorized":
                changed["release_authorization"]["explicit_stable_publication"] = False
            with self.subTest(change=change), self.assertRaises(ValueError):
                validate_readiness(spec, changed)

    def test_changed_plan_or_mode_rejects_before_reading_packages(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / "WORKFLOW-PLAN.json"
            path.write_text(json.dumps({"mode": "publish"}))
            with self.assertRaisesRegex(ValueError, "approval mismatch"):
                verify_plan(root, "0" * 64, "publish")
            with self.assertRaisesRegex(ValueError, "mode mismatch"):
                verify_plan(root, digest_file(path), "promote")


if __name__ == "__main__":
    unittest.main()
