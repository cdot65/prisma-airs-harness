"""Stable promotion preserves immutable packages and unrelated channels."""

import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from airs_test_release_spec import PACKAGE_ORDER, canonical_digest
from promote_airs_stable import (
    WORKSPACE_BASELINE_011,
    WORKSPACE_BASELINE_012,
    WORKSPACE_SOURCE_012,
    promote,
    validate_readiness,
    validate_workspace,
)
from test_airs_test_release_publish import MemoryRegistry, fixture


class PromotionRegistry(MemoryRegistry):
    def __init__(self, plan):
        super().__init__(plan)
        self.promoted = []
        for row in plan["publish_order"]:
            self.documents[row["name"]]["versions"][row["version"]] = {
                "name": row["name"],
                "version": row["version"],
                "dist": {"integrity": row["integrity"]},
            }
            self.documents[row["name"]]["dist-tags"]["stable-candidate"] = row[
                "version"
            ]

    def promote(self, name, version):
        if name == self.fail_at:
            raise RuntimeError("Synthetic interruption")
        self.documents[name]["dist-tags"]["latest"] = version
        self.promoted.append(name)


class StablePromotionTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        with patch("test_airs_test_release_publish.VERSION", "0.1.1"):
            self.spec, self.plan, _ = fixture(self.root)
        self.spec.update(
            scope="owner-authorized-stable",
            tag="stable-candidate",
            previous_version="0.1.0-alpha.22.mcp.6",
        )
        self.verification = {
            "spec_sha256": canonical_digest(self.spec),
            "installation": "registry",
        }
        self.readiness = {
            "version": "0.1.1",
            "source_commit": self.spec["source_commit"],
            "workspace": {
                "scope": "full-workspace",
                "passed": 100,
                "failed": 0,
                "source_commit": self.spec["source_commit"],
            },
            "owner_acceptance": {
                "version": self.spec["previous_version"],
                "inference_signin": True,
                "mcp_signin": True,
                "servicenow_read": True,
                "restart_reuse": True,
            },
            "runtime_behavior_unchanged_since_owner_acceptance": True,
        }
        self.registry = PromotionRegistry(self.plan)

    def run_promotion(self):
        return promote(
            self.spec,
            self.plan,
            self.verification,
            self.readiness,
            self.root / "promotion",
            self.registry,
        )

    def test_interruption_resumes_native_first_and_preserves_other_channels(self):
        original = copy.deepcopy(self.registry.documents)
        self.registry.fail_at = PACKAGE_ORDER[1]
        with self.assertRaisesRegex(RuntimeError, "interruption"):
            self.run_promotion()
        self.assertEqual(self.registry.promoted, PACKAGE_ORDER[:1])
        self.registry.fail_at = None
        receipt = self.run_promotion()
        self.assertTrue(receipt["complete"])
        self.assertEqual(self.registry.promoted, PACKAGE_ORDER)
        for name in PACKAGE_ORDER:
            self.assertEqual(
                self.registry.documents[name],
                {
                    **original[name],
                    "dist-tags": {**original[name]["dist-tags"], "latest": "0.1.1"},
                },
            )
        self.assertEqual(self.run_promotion(), receipt)

    def test_incomplete_or_mismatched_readiness_is_rejected_before_mutation(self):
        changes = [
            {"source_commit": "b" * 40},
            {"workspace": {**self.readiness["workspace"], "failed": 1}},
            {"workspace": {**self.readiness["workspace"], "source_commit": "b" * 40}},
            {
                "owner_acceptance": {
                    **self.readiness["owner_acceptance"],
                    "restart_reuse": False,
                }
            },
            {"runtime_behavior_unchanged_since_owner_acceptance": False},
        ]
        for change in changes:
            with self.subTest(change=change), self.assertRaises(ValueError):
                validate_readiness(self.spec, {**self.readiness, **change})
        self.assertEqual(self.registry.promoted, [])

    def test_reviewed_baseline_keeps_failures_visible_in_promotion_receipt(self):
        cases = {
            name: {
                "disposition": disposition,
                "evidence_verified": True,
                "evidence_sha256": "a" * 64,
                "installed_command_rejected": True,
                "focused_check_passed": True,
                "runtime_source_unchanged": True,
            }
            for name, disposition in WORKSPACE_BASELINE_011.items()
        }
        self.readiness["workspace"].update(
            failed=len(cases),
            failures=list(cases),
            baseline_review={
                "source_commit": self.spec["source_commit"],
                "upstream_revision": "rust-v0.154.0",
                "upstream_implementations_unchanged": True,
                "unresolved_release_blockers": [],
                "cases": cases,
            },
        )
        receipt = self.run_promotion()
        self.assertEqual(receipt["workspace"], self.readiness["workspace"])
        self.assertEqual(receipt["workspace"]["failed"], len(cases))

    def test_unknown_or_unverified_baseline_cannot_hide_a_failure(self):
        name = next(
            name
            for name, kind in WORKSPACE_BASELINE_011.items()
            if kind == "corrected-test-fixture"
        )
        self.readiness["workspace"].update(
            failed=1,
            failures=[name],
            baseline_review={
                "source_commit": self.spec["source_commit"],
                "upstream_revision": "rust-v0.154.0",
                "upstream_implementations_unchanged": True,
                "unresolved_release_blockers": [],
                "cases": {
                    name: {
                        "disposition": "corrected-test-fixture",
                        "evidence_verified": True,
                        "evidence_sha256": "a" * 64,
                        "focused_check_passed": True,
                        "runtime_source_unchanged": True,
                    }
                },
            },
        )
        validate_readiness(self.spec, self.readiness)
        for field in (
            "evidence_verified",
            "focused_check_passed",
            "runtime_source_unchanged",
        ):
            changed = copy.deepcopy(self.readiness)
            changed["workspace"]["baseline_review"]["cases"][name][field] = False
            with self.subTest(field=field), self.assertRaises(ValueError):
                validate_readiness(self.spec, changed)
        changed = copy.deepcopy(self.readiness)
        changed["workspace"]["failures"] = ["new supported authentication regression"]
        with self.assertRaises(ValueError):
            validate_readiness(self.spec, changed)
        changed = copy.deepcopy(self.readiness)
        changed["workspace"]["scope"] = "focused-diagnostic"
        with self.assertRaises(ValueError):
            validate_readiness(self.spec, changed)
        self.assertEqual(self.registry.promoted, [])

    def test_012_review_requires_exact_source_and_rejects_old_fixture_exceptions(self):
        spec = {"version": "0.1.2", "source_commit": WORKSPACE_SOURCE_012}
        cases = {
            name: {
                "disposition": disposition,
                "evidence_verified": True,
                "evidence_sha256": "a" * 64,
                "installed_command_rejected": True,
            }
            for name, disposition in WORKSPACE_BASELINE_012.items()
        }
        workspace = {
            "scope": "full-workspace",
            "source_commit": WORKSPACE_SOURCE_012,
            "passed": 100,
            "failed": len(cases),
            "failures": list(cases),
            "baseline_review": {
                "source_commit": WORKSPACE_SOURCE_012,
                "upstream_revision": "rust-v0.154.0",
                "upstream_implementations_unchanged": True,
                "unresolved_release_blockers": [],
                "cases": cases,
            },
        }
        validate_workspace(spec, workspace)
        for version in ["0.1.3", "0.1.2-alpha.6.mcp.1"]:
            with self.subTest(version=version), self.assertRaises(ValueError):
                validate_workspace({**spec, "version": version}, workspace)
        changed = copy.deepcopy(workspace)
        changed["source_commit"] = "b" * 40
        changed["baseline_review"]["source_commit"] = "b" * 40
        with self.assertRaises(ValueError):
            validate_workspace({**spec, "source_commit": "b" * 40}, changed)
        for name, disposition in WORKSPACE_BASELINE_011.items():
            if disposition == "disabled-upstream-service":
                continue
            changed = copy.deepcopy(workspace)
            changed["failures"].append(name)
            changed["failed"] += 1
            changed["baseline_review"]["cases"][name] = {
                "disposition": disposition,
                "evidence_verified": True,
                "evidence_sha256": "a" * 64,
                "focused_check_passed": True,
                "runtime_source_unchanged": True,
            }
            with self.subTest(old_exception=name), self.assertRaises(ValueError):
                validate_workspace(spec, changed)
        for name in cases:
            for field in ["evidence_verified", "installed_command_rejected"]:
                changed = copy.deepcopy(workspace)
                changed["baseline_review"]["cases"][name][field] = False
                with (
                    self.subTest(case=name, field=field),
                    self.assertRaises(ValueError),
                ):
                    validate_workspace(spec, changed)
        self.assertEqual(workspace["failed"], 3)

    def test_wrong_registry_mode_or_package_bytes_prevent_promotion(self):
        self.verification["installation"] = "candidate"
        with self.assertRaisesRegex(ValueError, "acceptance identity"):
            self.run_promotion()
        self.verification["installation"] = "registry"
        row = self.registry.documents[PACKAGE_ORDER[0]]["versions"]["0.1.1"]
        row["dist"]["integrity"] = "changed"
        with self.assertRaisesRegex(ValueError, "immutable package"):
            self.run_promotion()
        self.assertEqual(self.registry.promoted, [])

    def test_concurrent_tag_change_after_interruption_is_not_overwritten(self):
        self.registry.fail_at = PACKAGE_ORDER[1]
        with self.assertRaises(RuntimeError):
            self.run_promotion()
        self.registry.fail_at = None
        self.registry.documents[PACKAGE_ORDER[2]]["dist-tags"]["mcp"] = "unexpected"
        with self.assertRaisesRegex(ValueError, "concurrently"):
            self.run_promotion()
        self.assertEqual(self.registry.promoted, PACKAGE_ORDER[:1])


if __name__ == "__main__":
    unittest.main()
