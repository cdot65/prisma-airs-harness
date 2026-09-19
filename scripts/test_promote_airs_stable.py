"""Stable promotion preserves immutable packages and unrelated channels."""

import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from airs_test_release_spec import PACKAGE_ORDER, canonical_digest
from promote_airs_stable import promote, validate_readiness
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
