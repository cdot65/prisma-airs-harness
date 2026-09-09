"""Plan real synthetic bundles from packed files, not npm-omitted stage extras."""

import unittest

import airs_bundle
import plan_airs_review_publication as planner
from restore_airs_review_stage import restore_stage
import test_airs_bundle as bundle_fixture
import test_plan_airs_review_publication as plan_fixture


class ArchivedBundlePlan(unittest.TestCase):
    def setUp(self):
        self.fixture = plan_fixture.ReviewPublicationPlan()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.fixture.bundle_check.stop()
        self.bundle = bundle_fixture.BundleTests()
        self.bundle.setUp()
        self.addCleanup(self.bundle.doCleanups)
        self.bundle.targets = [row[0] for row in planner.NATIVES.values()]
        wanted = airs_bundle.sharp_packages(self.bundle.targets)
        self.bundle.add_package("sharp", "1.0.0", {name: "1.0.0" for name in wanted})
        for name in wanted:
            self.bundle.add_package(name, "1.0.0")
        self.root = self.fixture.root
        self.bundle.launcher = self.root / planner.LAUNCHER
        self.inventory = self.bundle.build()
        self.fixture.receipt["cli_bundle"]["inventory_sha256"] = planner.regular_digest(
            self.bundle.launcher / "BUNDLE-INVENTORY.json"
        )[0]
        self.fixture.write(
            self.bundle.launcher / "PACKAGE-TOOLING.json",
            self.fixture.receipt["package_tooling"],
        )
        self.omitted = "node_modules/" + airs_bundle.CLI + "/CHANGELOG.md"
        self.fixture.pack(planner.LAUNCHER, omitted=self.omitted)
        self.fixture.save_receipt()

    def test_plan_roundtrip_counts_only_archived_verified_inventory_files(self):
        staged_count = airs_bundle.verify_bundle(self.bundle.launcher, self.inventory)[
            "files"
        ]
        before = planner.plan_publication(self.root, "auth-review")
        restored = self.root / "restored"
        restore_stage(self.root / "tarballs", self.root / "NPM-PACKAGES.json", restored)
        after = planner.plan_publication(restored, "auth-review")
        self.assertEqual(before, after)
        self.assertEqual(before["bundle_verification"]["files"], staged_count - 1)
        self.assertTrue((self.bundle.launcher / self.omitted).is_file())
        self.assertFalse((restored / planner.LAUNCHER / self.omitted).exists())

    def test_unarchived_optional_staging_tamper_still_fails_full_verification(self):
        (self.bundle.launcher / self.omitted).write_bytes(b"changed optional content")
        with self.assertRaisesRegex(ValueError, "Installed bundle file differs"):
            planner.plan_publication(self.root, "auth-review")

    def test_required_payload_cannot_be_treated_as_optional(self):
        self.fixture.pack(
            planner.LAUNCHER, omitted="node_modules/" + airs_bundle.CLI + "/run.js"
        )
        self.fixture.save_receipt()
        with self.assertRaisesRegex(
            ValueError, "Required bundled dependency file missing"
        ):
            planner.plan_publication(self.root, "auth-review")


if __name__ == "__main__":
    unittest.main()
