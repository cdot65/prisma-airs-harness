"""Synthetic publication planning controls; never publish or execute a binary."""

import copy
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

from airs_review_release import SCOPE
import plan_airs_review_publication as planner


class ReviewPublicationPlan(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.version = "0.1.0-alpha.10"
        self.source = "a" * 40
        self.receipt = {
            "published": False,
            "registry": planner.REGISTRY,
            "source_commit": self.source,
            "publish_order": [],
        }
        (self.root / "tarballs").mkdir()
        for name, (target, platform, arch) in planner.NATIVES.items():
            directory = self.root / name
            (directory / "bin").mkdir(parents=True)
            (directory / "bin/airs-harness").write_text(
                "Synthetic nonexecutable native: " + target
            )
            native_hash = planner.regular_digest(directory / "bin/airs-harness")[0]
            document = {
                "schema_version": 1,
                "scope": SCOPE,
                "passed": True,
                "release_ready": True,
                "full_authentication_release_ready": False,
                "binary_sha256": native_hash,
                "target": target,
                "product_version": self.version,
                "source_commit": self.source,
                "independent_review": {
                    "scope": SCOPE,
                    "reviewer": "synthetic-test-only",
                    "verdict": "pass",
                    "score": 9,
                },
                "evidence": [],
                "signing": {"kind": "linux-provenance-checksums"},
            }
            roles = [
                "installed-runtime",
                "package-integrity",
                "independent-review",
                "native-provenance",
            ]
            if platform == "darwin":
                roles += ["developer-id-signature", "apple-notarization"]
                document["signing"] = {
                    "kind": "developer-id-application",
                    "verified": True,
                    "hardened_runtime": True,
                    "signed_binary_sha256": native_hash,
                    "team_identifier": "TESTTEAM01",
                    "notarization": {
                        "status": "Accepted",
                        "archive_sha256": "b" * 64,
                        "submission_id": "00000000-0000-4000-8000-000000000001",
                    },
                }
            evidence = directory / "validation-evidence"
            evidence.mkdir()
            for role in roles:
                file = evidence / (role + ".txt")
                file.write_text("Synthetic unit-test evidence only")
                document["evidence"].append(
                    {
                        "role": role,
                        "path": file.name,
                        "sha256": planner.regular_digest(file)[0],
                    }
                )
            self.write(directory / "VALIDATION.json", document)
            self.write(
                directory / "BUILD-INFO.json",
                {
                    "product": "Prisma AIRS Harness",
                    "version": self.version,
                    "source_commit": self.source,
                    "target": target,
                    "binary_sha256": native_hash,
                    "release_scope": SCOPE,
                    "validation_receipt_sha256": planner.regular_digest(
                        directory / "VALIDATION.json"
                    )[0],
                },
            )
            if platform == "darwin":
                signing = document["signing"]
                self.write(
                    directory / "SIGNING.json",
                    {
                        "binary_sha256": native_hash,
                        "source_commit": self.source,
                        "target": target,
                        "team_id": signing["team_identifier"],
                        "codesign_verified": True,
                        "hardened_runtime": True,
                        "notarization_verified": True,
                        "archive_sha256": signing["notarization"]["archive_sha256"],
                        "owner_reported_submission_id": signing["notarization"][
                            "submission_id"
                        ],
                    },
                )
                info = json.loads((directory / "BUILD-INFO.json").read_text())
                info["signing_receipt_sha256"] = planner.regular_digest(
                    directory / "SIGNING.json"
                )[0]
                self.write(directory / "BUILD-INFO.json", info)
            manifest = self.manifest(name)
            manifest.update(os=[platform], cpu=[arch])
            self.write(directory / "package.json", manifest)
        launcher = self.root / planner.LAUNCHER
        launcher.mkdir(parents=True)
        inventory = {"packages": []}
        self.write(launcher / "BUNDLE-INVENTORY.json", inventory)
        bundle = {
            "inventory_sha256": planner.regular_digest(
                launcher / "BUNDLE-INVENTORY.json"
            )[0],
            "required_pins": {
                "@cdot65/prisma-airs-cli": "5.2.0",
                "@cdot65/prisma-airs-sdk": "0.28.0",
            },
            "targets": [value[0] for value in planner.NATIVES.values()],
        }
        manifest = self.manifest(planner.LAUNCHER)
        manifest.update(
            optionalDependencies={name: self.version for name in planner.NATIVES},
            dependencies={"@cdot65/prisma-airs-cli": "5.2.0"},
            bundleDependencies=["@cdot65/prisma-airs-cli"],
            type="module",
            bin={"airs-harness": "bin/airs-harness.js"},
        )
        self.write(launcher / "package.json", manifest)
        sources = {}
        for name in [
            "bin/airs-harness.js",
            "lib/child.js",
            "lib/launcher.js",
            "lib/prisma-cli.js",
            "managed-cli/airs",
            "managed-cli/airs.cmd",
            "managed-cli/empty.env",
        ]:
            path = launcher / name
            path.parent.mkdir(exist_ok=True)
            path.write_text("synthetic nonexecutable fixture")
            sources["npm/airs-harness/" + name] = planner.regular_digest(path)[0]
        tooling = {
            "native_source_commit": self.source,
            "cli_bundle": bundle,
            "files": sources,
        }
        self.write(launcher / "PACKAGE-TOOLING.json", tooling)
        self.receipt.update(cli_bundle=bundle, package_tooling=tooling)
        for name in [*planner.NATIVES, planner.LAUNCHER]:
            self.pack(name)
        self.save_receipt()
        self.bundle_check = patch.object(
            planner, "verify_bundle", return_value={"synthetic_unit_control": True}
        )
        self.mock_bundle = self.bundle_check.start()
        self.addCleanup(self.bundle_check.stop)

    def manifest(self, name):
        return {
            "name": name,
            "version": self.version,
            "publishConfig": {"registry": planner.REGISTRY},
            "repository": {
                "type": "git",
                "url": "git+https://github.com/cdot65/airs-harness.git",
            },
        }

    def write(self, path, value):
        path.write_text(json.dumps(value, indent=2) + "\n")

    def save_receipt(self):
        self.write(self.root / "NPM-PACKAGES.json", self.receipt)

    def pack(self, name, omitted=None):
        filename = name.split("/")[1] + ".tgz"
        directory = self.root / name
        archive = self.root / "tarballs" / filename
        with tarfile.open(archive, "w:gz") as tar:
            for file in sorted(directory.rglob("*")):
                if file.is_file() and file.relative_to(directory).as_posix() != omitted:
                    tar.add(
                        file,
                        arcname="package/" + file.relative_to(directory).as_posix(),
                        recursive=False,
                    )
        sha256, integrity = planner.regular_digest(archive)
        record = {
            "name": name,
            "version": self.version,
            "filename": filename,
            "sha256": sha256,
            "integrity": integrity,
        }
        self.receipt["publish_order"] = [
            row for row in self.receipt["publish_order"] if row["name"] != name
        ] + [record]

    def plan(self):
        return planner.plan_publication(self.root, "auth-review")

    def test_plan_is_exact_review_only_and_native_first_without_execution(self):
        with patch(
            "subprocess.run", side_effect=AssertionError("No executable may run")
        ):
            plan = self.plan()
        self.assertFalse(plan["published"])
        self.assertTrue(plan["registry_preflight_required"])
        self.assertFalse(plan["full_authentication_release_ready"])
        self.assertEqual(plan["packages"][-1]["name"], planner.LAUNCHER)
        self.assertEqual(len(plan["native_validation"]), 2)
        self.mock_bundle.assert_called_once()

    def test_candidate_markers_and_nonreview_tags_fail(self):
        original = copy.deepcopy(self.receipt)
        for key, value in [
            ("private", True),
            ("publishable", False),
            ("release_status", "candidate"),
        ]:
            with self.subTest(key=key):
                self.receipt = copy.deepcopy(original)
                self.receipt[key] = value
                self.save_receipt()
                with self.assertRaises(ValueError):
                    self.plan()
        self.receipt = original
        self.save_receipt()
        for tag in ["latest", "stable", "production", ""]:
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                planner.plan_publication(self.root, tag)

    def test_archive_sri_and_actual_staged_payload_both_bind(self):
        self.receipt["publish_order"][0]["integrity"] = "sha512-invalid"
        self.save_receipt()
        with self.assertRaises(ValueError):
            self.plan()
        name = next(iter(planner.NATIVES))
        self.pack(name)
        self.save_receipt()
        (self.root / name / "bin/airs-harness").write_text("modified after packing")
        with self.assertRaises(ValueError):
            self.plan()

    def test_omitted_validation_evidence_fails_despite_matching_archive_receipt(self):
        name = next(iter(planner.NATIVES))
        self.pack(name, "validation-evidence/installed-runtime.txt")
        self.save_receipt()
        with self.assertRaises(ValueError):
            self.plan()

    def test_remote_native_urls_and_launcher_code_changes_fail(self):
        path = self.root / planner.LAUNCHER / "package.json"
        manifest = json.loads(path.read_text())
        manifest["optionalDependencies"][next(iter(planner.NATIVES))] = (
            "https://example.invalid/native.tgz"
        )
        self.write(path, manifest)
        self.pack(planner.LAUNCHER)
        self.save_receipt()
        with self.assertRaises(ValueError):
            self.plan()
        manifest["optionalDependencies"] = {
            name: self.version for name in planner.NATIVES
        }
        self.write(path, manifest)
        (path.parent / "bin/airs-harness.js").write_text("unreviewed launcher")
        self.pack(planner.LAUNCHER)
        self.save_receipt()
        with self.assertRaises(ValueError):
            self.plan()

    def test_mac_native_signing_receipt_must_match_even_when_repacked(self):
        name = planner.LAUNCHER + "-darwin-arm64"
        path = self.root / name / "SIGNING.json"
        signing = json.loads(path.read_text())
        signing["notarization_verified"] = False
        self.write(path, signing)
        provenance = json.loads((path.parent / "BUILD-INFO.json").read_text())
        provenance["signing_receipt_sha256"] = planner.regular_digest(path)[0]
        self.write(path.parent / "BUILD-INFO.json", provenance)
        self.pack(name)
        self.save_receipt()
        with self.assertRaises(ValueError):
            self.plan()

    def test_missing_required_bundle_file_fails_even_if_staging_verifier_passes(self):
        path = self.root / planner.LAUNCHER / "BUNDLE-INVENTORY.json"
        self.write(
            path,
            {
                "packages": [
                    {
                        "path": "node_modules/required",
                        "files": {"package.json": "a" * 64},
                    }
                ]
            },
        )
        self.receipt["cli_bundle"]["inventory_sha256"] = planner.regular_digest(path)[0]
        self.write(
            path.parent / "PACKAGE-TOOLING.json", self.receipt["package_tooling"]
        )
        self.pack(planner.LAUNCHER)
        self.save_receipt()
        with self.assertRaises(ValueError):
            self.plan()


if __name__ == "__main__":
    unittest.main()
