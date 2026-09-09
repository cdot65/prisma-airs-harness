"""Real archive and installed-file boundaries with a synthetic read-only registry."""

import json
import shutil
import unittest
from unittest.mock import patch

from plan_airs_review_publication import LAUNCHER, plan_publication, regular_digest
from publish_airs_review import save
import test_plan_airs_review_publication as plan_fixture
from test_publish_airs_review import FakeRegistry
import validate_airs_registry_install as validator


class RegistryInstall(unittest.TestCase):
    def setUp(self):
        self.fixture = plan_fixture.ReviewPublicationPlan()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.root = self.fixture.root
        self.evidence = self.root / "evidence"
        self.evidence.mkdir()
        self.plan = plan_publication(self.root, "auth-review")
        save(self.evidence / "PUBLICATION-PLAN.json", self.plan)
        self.plan_hash = regular_digest(self.evidence / "PUBLICATION-PLAN.json")[0]
        self.publication = {
            "complete": True,
            "published": True,
            "approved_plan_sha256": self.plan_hash,
            "version": self.plan["version"],
            "source_commit": self.plan["source_commit"],
            "registry": self.plan["registry"],
            "dist_tag": "auth-review",
        }
        save(self.evidence / "PUBLICATION.json", self.publication)
        shutil.copyfile(
            self.root / "NPM-PACKAGES.json", self.evidence / "NPM-PACKAGES.json"
        )
        self.registry = FakeRegistry(self.plan["packages"], self.root / "tarballs")
        self.registry.existing = set(self.registry.records)
        self.registry.tag_values = {
            row["name"]: {"auth-review": row["version"]}
            for row in self.plan["packages"]
        }
        self.output = self.root / "acceptance"
        self.output.mkdir()

    def fetch(self):
        return validator.fetch_review(
            self.evidence, self.plan_hash, self.output, self.registry
        )

    def test_exact_registry_archives_restore_the_reviewed_plan_without_mutation(self):
        plan, stage = self.fetch()
        self.assertEqual(plan, self.plan)
        self.assertEqual(plan_publication(stage, "auth-review"), self.plan)
        self.assertEqual(self.registry.mutations, [])
        receipt = json.loads((self.output / "REGISTRY-DOWNLOAD.json").read_text())
        self.assertTrue(receipt["passed"])
        self.assertEqual(len(receipt["packages"]), 3)

    def test_partial_publication_or_changed_plan_cannot_start_download(self):
        self.publication["complete"] = False
        save(self.evidence / "PUBLICATION.json", self.publication)
        with self.assertRaisesRegex(ValueError, "completed matching"):
            self.fetch()
        self.assertFalse((self.output / "registry-archives").exists())
        save(
            self.evidence / "PUBLICATION-PLAN.json", {**self.plan, "version": "changed"}
        )
        with self.assertRaisesRegex(ValueError, "plan hash"):
            self.fetch()

    def test_corrupt_registry_download_or_wrong_association_prevents_install(self):
        self.registry.corrupt_download = next(iter(self.registry.records))
        with self.assertRaisesRegex(ValueError, "approved bytes"):
            self.fetch()
        self.assertFalse((self.output / "verified-stage").exists())
        shutil.rmtree(self.output / "registry-archives")
        self.registry.corrupt_download = None
        self.registry.wrong_association = True
        with self.assertRaisesRegex(ValueError, "associated"):
            self.fetch()
        self.assertEqual(self.registry.mutations, [])

    def test_installed_native_and_launcher_tampering_fail_before_execution(self):
        prefix = self.root / "prefix"
        modules = prefix / "lib/node_modules"
        launcher = modules / LAUNCHER
        native = modules / "@cdot65/prisma-airs-harness-linux-x64"
        shutil.copytree(self.root / LAUNCHER, launcher)
        shutil.copytree(self.root / "@cdot65/prisma-airs-harness-linux-x64", native)
        (prefix / "bin").mkdir()
        (prefix / "bin/airs-harness").symlink_to(launcher / "bin/airs-harness.js")
        with (
            patch.object(validator.platform, "system", return_value="Linux"),
            patch.object(validator.platform, "machine", return_value="x86_64"),
            patch.object(validator, "verify_bundle", return_value={"synthetic": True}),
        ):
            result = validator.verify_installed(
                self.root, prefix, native / "package.json"
            )
            self.assertEqual(result[:2], (launcher, native / "bin/airs-harness"))
            binary = native / "bin/airs-harness"
            original = binary.read_bytes()
            binary.write_bytes(b"tampered native")
            with self.assertRaises(ValueError):
                validator.verify_installed(self.root, prefix, native / "package.json")
            binary.write_bytes(original)
            (launcher / "lib/launcher.js").write_bytes(b"tampered launcher")
            with self.assertRaises(ValueError):
                validator.verify_installed(self.root, prefix, native / "package.json")


if __name__ == "__main__":
    unittest.main()
