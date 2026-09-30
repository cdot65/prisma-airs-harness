"""Exercise metadata-only promotion and rejection of unaccepted CI candidates."""

import hashlib
import io
import json
import tarfile
import tempfile
import unittest
from pathlib import Path

from airs_test_release_archive import inspect_archive
from airs_signed_macos_artifact import DESIGNATED_REQUIREMENT, TEAM
from stage_airs_macos_preview import BUILD, MANIFEST, NATIVE, REGISTRY, encoded, stage


class MacPreviewStaging(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.packages, self.evidence = self.root / "candidate", self.root / "evidence"
        (self.packages / "tarballs").mkdir(parents=True)
        self.evidence.mkdir()
        self.output = self.root / "staged"
        self.source, self.version = "a" * 40, "0.1.4-alpha.1.mcp.1"
        binary = b"accepted executable payload"
        self.binary_hash = hashlib.sha256(binary).hexdigest()
        self.signing = {
            "binary_sha256": self.binary_hash,
            "source_commit": self.source,
            "target": "aarch64-apple-darwin",
            "team_id": TEAM,
            "identifier": "airs-harness",
            "designated_requirement": DESIGNATED_REQUIREMENT,
            "codesign_verified": True,
            "hardened_runtime": True,
            "notarization_verified": True,
        }
        self.save(self.evidence / "SIGNING.json", self.signing)
        records = []
        for name in [NATIVE, "prisma-airs-harness"]:
            manifest = {
                "name": name,
                "version": self.version,
                "private": True,
                "publishConfig": {"registry": REGISTRY},
            }
            members = {}
            if name == NATIVE:
                manifest.update(os=["darwin"], cpu=["arm64"])
                members.update(
                    {
                        "package/bin/airs-harness": binary,
                        "package/SIGNING.json": encoded(self.signing),
                        BUILD: encoded(
                            {
                                "source_commit": self.source,
                                "version": self.version,
                                "target": "aarch64-apple-darwin",
                                "binary_sha256": self.binary_hash,
                                "publishable": False,
                                "release_status": "signed-unvalidated-candidate",
                                "signing_receipt_sha256": hashlib.sha256(
                                    encoded(self.signing)
                                ).hexdigest(),
                            }
                        ),
                    }
                )
            else:
                manifest["optionalDependencies"] = {NATIVE: self.version}
                members["package/bin/airs.js"] = b"unchanged launcher"
            members[MANIFEST] = encoded(manifest)
            filename = f"{name}-{self.version}.tgz"
            path = self.packages / "tarballs" / filename
            with tarfile.open(path, "w:gz", format=tarfile.USTAR_FORMAT) as writer:
                for member, content in members.items():
                    info = tarfile.TarInfo(member)
                    info.mode, info.size = 0o755, len(content)
                    writer.addfile(info, io.BytesIO(content))
            inventory = inspect_archive(path)
            records.append(
                {
                    "name": name,
                    "version": self.version,
                    "filename": filename,
                    **{k: inventory[k] for k in ["sha256", "integrity"]},
                }
            )
        self.save(
            self.packages / "NPM-PACKAGES.json",
            {
                "source_commit": self.source,
                "packaging_commit": self.source,
                "registry": REGISTRY,
                "publish_order": records,
            },
        )
        self.save(
            self.evidence / "artifact-selection.json",
            {
                "runtime_source": self.source,
                "validation_tooling_source": self.source,
                "binary_sha256": self.binary_hash,
                "acceptance_run": "42",
                "signing": {"ad_hoc": False, "developer_id": True, "notarized": True},
            },
        )
        self.save(
            self.evidence / "VALIDATION.json",
            {
                "source_commit": self.source,
                "validation_tooling_commit": self.source,
                "product_version": self.version,
                "target": "aarch64-apple-darwin",
                "binary_sha256": self.binary_hash,
                "validation_run": "42",
            },
        )
        for name in [
            "native-integrity.json",
            "native-cli-keychain.json",
            "cli-keychain.json",
            "keychain.json",
            "npm-install.json",
            "prisma-cli.json",
        ]:
            self.save(
                self.evidence / name,
                {
                    "passed": True,
                    "binary_sha256": self.binary_hash,
                    "source_commit": self.source,
                    "version": f"airs {self.version}",
                },
            )
        for name in ["native-tests.log", "npm-tests.log"]:
            (self.evidence / name).write_text(
                "Ran 3 tests in 1.234s\n\nOK (skipped=1)\n"
            )

    def save(self, path, value):
        path.write_bytes(encoded(value))

    def promote(self, tag="mac-preview"):
        return stage(self.packages, self.evidence, self.output, self.source, tag)

    def test_stages_only_metadata_and_retains_candidate_provenance(self):
        receipt = self.promote()
        self.assertFalse(receipt["release_ready"])
        self.assertTrue(receipt["notarized"])
        for row in receipt["publish_order"]:
            old = inspect_archive(self.packages / "tarballs" / row["filename"])
            new = inspect_archive(self.output / "tarballs" / row["filename"])
            self.assertTrue(old["json"][MANIFEST]["private"])
            self.assertNotIn("private", new["json"][MANIFEST])
            self.assertEqual(new["json"][MANIFEST]["os"], ["darwin"])
            for path, member in old["members"].items():
                if path not in (MANIFEST, BUILD):
                    self.assertEqual(member, new["members"][path])
            if row["name"] == NATIVE:
                self.assertEqual(
                    new["json"][
                        "package/validation-evidence/original-build-candidate.json"
                    ],
                    old["json"][BUILD],
                )

    def test_rejects_unsigned_or_changed_signing_identity(self):
        for change in [
            {"identifier": "airs-harness-build-id"},
            {"designated_requirement": 'cdhash H"123"'},
            {"notarization_verified": False},
            {"team_id": "OTHER"},
            {"binary_sha256": "f" * 64},
        ]:
            with self.subTest(change=change):
                self.save(self.evidence / "SIGNING.json", dict(self.signing, **change))
                with self.assertRaisesRegex(ValueError, "signing identity"):
                    self.promote()
                self.assertFalse(self.output.exists())

    def test_rejects_protected_tag_without_creating_output(self):
        with self.assertRaisesRegex(ValueError, "mac-preview"):
            self.promote("latest")
        self.assertFalse(self.output.exists())

    def test_rejects_failed_or_foreign_acceptance(self):
        path = self.evidence / "npm-install.json"
        original = json.loads(path.read_text())
        for change in [
            {"passed": False},
            {"binary_sha256": "f" * 64},
            {"source_commit": "b" * 40},
        ]:
            with self.subTest(change=change):
                self.save(path, dict(original, **change))
                with self.assertRaises(ValueError):
                    self.promote()
                self.assertFalse(self.output.exists())

    def test_rejects_failed_test_log(self):
        (self.evidence / "npm-tests.log").write_text(
            "Ran 3 tests in 1s\n\nFAILED (failures=1)\n"
        )
        with self.assertRaisesRegex(ValueError, "npm-tests"):
            self.promote()

    def test_rejects_tampered_candidate(self):
        archive = next((self.packages / "tarballs").glob("*.tgz"))
        with archive.open("ab") as stream:
            stream.write(b"tampered")
        with self.assertRaises((ValueError, OSError)):
            self.promote()

    def test_rejects_public_registry(self):
        path = self.packages / "NPM-PACKAGES.json"
        metadata = json.loads(path.read_text())
        self.save(path, dict(metadata, registry="https://registry.npmjs.org"))
        with self.assertRaisesRegex(ValueError, "npm.cdot.io"):
            self.promote()


if __name__ == "__main__":
    unittest.main()
