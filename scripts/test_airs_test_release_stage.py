"""Adversarial identity, streaming archive, and metadata-only stage checks."""

import gzip
import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import airs_test_release_archive as archive
import airs_test_release_stage as stage
from airs_test_release_spec import (
    TARGETS,
    canonical_digest,
    digest_file,
    json_bytes,
    validate_spec,
)


def encoded(value):
    return json.dumps(value, sort_keys=True).encode()


def write_tar(path, members):
    with tarfile.open(path, "w:gz", format=tarfile.USTAR_FORMAT) as writer:
        for name, content, mode in members:
            info = tarfile.TarInfo(name)
            info.size, info.mode = len(content), mode
            writer.addfile(info, io.BytesIO(content))


def sample_spec():
    return {
        "schema_version": 1,
        "scope": "owner-authorized-test",
        "version": "0.1.0-alpha.22.mcp.3",
        "tag": "mcp",
        "registry": "https://npm.cdot.io",
        "source_commit": "a" * 40,
        "tooling_commit": "b" * 40,
        "packaging_commit": "c" * 40,
        "previous_version": "0.1.0-alpha.22.mcp.2",
        "developer_id_team": "G5QLZ5A8TA",
        "platforms": [
            {
                "target": target,
                "binary_sha256": hashlib.sha256(target.encode()).hexdigest(),
            }
            for target in TARGETS
        ],
    }


def candidates(root, spec):
    (root / "tarballs").mkdir(parents=True)
    records = []
    for target, name in [*TARGETS.items(), (None, "airs-harness")]:
        manifest = {"name": name, "version": spec["version"], "private": True}
        members = []
        if target:
            system, architecture = stage.PLATFORM_MANIFEST[target]
            manifest.update(os=[system], cpu=[architecture])
            native = target.encode()
            info = {
                "product": "Prisma AIRS Harness",
                "version": spec["version"],
                "source_commit": spec["source_commit"],
                "target": target,
                "binary_sha256": hashlib.sha256(native).hexdigest(),
                "publishable": False,
                "release_status": "candidate",
            }
            if target == "aarch64-apple-darwin":
                signing = encoded(
                    {
                        **info,
                        "team_id": spec["developer_id_team"],
                        "codesign_verified": True,
                        "hardened_runtime": True,
                        "notarization_verified": True,
                    }
                )
                members.append(("package/SIGNING.json", signing, 0o644))
                info["signing_receipt_sha256"] = hashlib.sha256(signing).hexdigest()
            members.extend(
                [
                    ("package/bin/airs-harness", native, 0o755),
                    (stage.BUILD, encoded(info), 0o644),
                ]
            )
        else:
            manifest["optionalDependencies"] = {
                package: spec["version"] for package in TARGETS.values()
            }
            members.append(("package/bin/launcher.js", b"trusted launcher", 0o755))
        members.append((stage.MANIFEST, encoded(manifest), 0o644))
        filename = f"{name}-{spec['version']}.tgz"
        write_tar(root / "tarballs" / filename, members)
        inspected = archive.inspect_archive(root / "tarballs" / filename)
        records.append(
            {
                "name": name,
                "version": spec["version"],
                "filename": filename,
                "sha256": inspected["sha256"],
                "integrity": inspected["integrity"],
            }
        )
    metadata = {
        "source_commit": spec["source_commit"],
        "packaging_commit": spec["packaging_commit"],
        "registry": spec["registry"],
        "publish_order": records,
        "cli_bundle": {"inventory_sha256": "f" * 64, "cli_version": "7.0.0"},
        "package_tooling": {
            "packaging_commit": spec["packaging_commit"],
            "files": {"packager.py": "d" * 64},
        },
        "required_dependencies": {"@cdot65/prisma-airs-cli": "7.0.0"},
    }
    (root / "NPM-PACKAGES.json").write_bytes(encoded(metadata))
    candidate = stage.inspect_packages(spec, root)
    return {
        "spec_sha256": canonical_digest(spec),
        **{
            key: spec[key]
            for key in (
                "version",
                "source_commit",
                "tooling_commit",
                "packaging_commit",
            )
        },
        **{
            key: candidate[key]
            for key in ("candidate_metadata_sha256", "candidate_packages_sha256")
        },
        "platforms": [
            {**row, "acceptance_sha256": "d" * 64} for row in spec["platforms"]
        ],
        "evidence_sha256": "e" * 64,
    }


class SpecTests(unittest.TestCase):
    def test_valid_spec_is_copied_and_ordered(self):
        spec = sample_spec()
        spec["platforms"].reverse()
        self.assertEqual(
            list(TARGETS), [p["target"] for p in validate_spec(spec)["platforms"]]
        )
        self.assertEqual("aarch64-apple-darwin", spec["platforms"][0]["target"])

    def test_unsafe_or_ambiguous_identity_rejected(self):
        for key, value in [
            ("schema_version", True),
            ("scope", "production"),
            ("tag", "latest"),
            ("version", "1.0.0"),
            ("version", "0.01.0-alpha.22.mcp.3"),
            ("source_commit", "HEAD"),
            ("tooling_commit", "a" * 39),
            ("registry", "https://user:secret@npm.cdot.io"),
            ("registry", "http://npm.cdot.io"),
            ("registry", "https://npm.cdot.io/?token=secret"),
            ("registry", "https://npm.cdot.io/../foo"),
            ("registry", "https://npm.cdot.io\n"),
            ("developer_id_team", "bad"),
        ]:
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                validate_spec({**sample_spec(), key: value})
        spec = sample_spec()
        spec["platforms"][1] = spec["platforms"][0]
        with self.assertRaises(ValueError):
            validate_spec(spec)
        for raw in (b'{"x":1,"x":2}', b'{"x":NaN}'):
            with self.assertRaises(ValueError):
                json_bytes(raw)


class ArchiveTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "archive.tgz"

    def test_payload_larger_than_dependency_limit_is_streamed(self):
        class Zeros:
            def read(self, count):
                if count > 65536:
                    raise ValueError("Unbounded fixture read")
                return bytes(count)

        with tarfile.open(self.path, "w:gz", format=tarfile.USTAR_FORMAT) as writer:
            info = tarfile.TarInfo("package/bin/large")
            info.size = 65 * 1024 * 1024
            writer.addfile(info, Zeros())
        self.assertEqual(
            65 * 1024 * 1024,
            archive.inspect_archive(self.path)["members"][info.name]["size"],
        )

    def test_unsafe_paths_duplicates_and_parent_conflicts_rejected(self):
        cases = [
            ["package/../escape"],
            ["/package/file"],
            ["package/C:bad"],
            ["package/CON"],
            ["package/a", "package/a"],
            ["package/A", "package/a"],
            ["package/café", "package/cafe\u0301"],
            ["package/a", "package/a/b"],
            ["package/a/b", "package/a"],
        ]
        for names in cases:
            with self.subTest(names=names):
                write_tar(self.path, [(name, b"x", 0o644) for name in names])
                with self.assertRaises(ValueError):
                    archive.inspect_archive(self.path)

    def test_links_extensions_special_modes_and_size_limits_rejected(self):
        for kind in (
            tarfile.SYMTYPE,
            tarfile.LNKTYPE,
            tarfile.CHRTYPE,
            tarfile.XHDTYPE,
            tarfile.GNUTYPE_SPARSE,
        ):
            with self.subTest(kind=kind):
                info = tarfile.TarInfo("package/bad")
                info.type = kind
                with gzip.open(self.path, "wb") as stream:
                    stream.write(info.tobuf(format=tarfile.USTAR_FORMAT) + bytes(1024))
                with self.assertRaises(ValueError):
                    archive.inspect_archive(self.path)
        write_tar(self.path, [("package/setuid", b"x", 0o4755)])
        with self.assertRaises(ValueError):
            archive.inspect_archive(self.path)
        write_tar(self.path, [("package/a", bytes(2048), 0o644)])
        with patch.object(archive, "MAX_DECODED", 1000), self.assertRaises(ValueError):
            archive.inspect_archive(self.path)
        with patch.object(archive, "MAX_MEMBERS", 0), self.assertRaises(ValueError):
            archive.inspect_archive(self.path)

    def test_truncation_trailing_payload_and_checksum_rejected(self):
        write_tar(self.path, [("package/a", b"x", 0o644)])
        original = gzip.decompress(self.path.read_bytes())
        for content in (
            original[:600],
            original[:-1024] + b"bad",
            b"bad" + original[3:],
        ):
            self.path.write_bytes(gzip.compress(content))
            with self.assertRaises((ValueError, tarfile.TarError)):
                archive.inspect_archive(self.path)

    def test_rewrite_changes_only_allowlisted_metadata(self):
        write_tar(
            self.path,
            [
                (stage.MANIFEST, b'{"private":true}', 0o644),
                ("package/bin/airs-harness", b"binary", 0o755),
            ],
        )
        target = self.path.with_name("rewritten.tgz")
        rewritten = archive.rewrite_archive(
            self.path, target, {stage.MANIFEST: b"{}", stage.ORIGINAL: b"{}"}
        )
        original = archive.inspect_archive(self.path)
        self.assertEqual(
            original["members"]["package/bin/airs-harness"],
            rewritten["members"]["package/bin/airs-harness"],
        )
        with self.assertRaises(ValueError):
            archive.rewrite_archive(
                self.path,
                target.with_name("bad.tgz"),
                {"package/bin/airs-harness": b"{}"},
            )


class StageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.spec, self.source = sample_spec(), self.root / "source"
        self.acceptance = candidates(self.source, self.spec)
        self.evidence, self.output = self.root / "evidence", self.root / "output"
        self.evidence.mkdir()
        mocked = patch.object(
            stage, "verify_acceptance_set", return_value=self.acceptance
        )
        self.verifier = mocked.start()
        self.addCleanup(mocked.stop)

    def test_stable_candidate_stage_preserves_runtime_and_scope(self):
        spec = {
            **self.spec,
            "scope": "owner-authorized-stable",
            "tag": "stable-candidate",
            "version": "0.1.1",
        }
        validate_spec(spec)
        source = self.root / "stable-source"
        self.verifier.return_value = candidates(source, spec)
        plan = stage.stage_packages(spec, source, self.evidence, self.output)
        self.assertEqual(plan, stage.verify_staged(spec, self.output, self.evidence))
        self.assertEqual(plan["release_scope"], spec["scope"])
        self.assertTrue(
            all(row["runtime_payload_unchanged"] for row in plan["publish_order"])
        )
        native = archive.inspect_archive(
            self.output / "tarballs" / plan["publish_order"][0]["filename"]
        )
        self.assertEqual(native["json"][stage.VALIDATION]["scope"], spec["scope"])
        self.assertEqual(native["json"][stage.BUILD]["release_scope"], spec["scope"])
        for change in (
            {"tag": "latest"},
            {"scope": "owner-authorized-test"},
            {"version": "0.1.1-rc.1"},
        ):
            with self.subTest(change=change), self.assertRaises(ValueError):
                validate_spec({**spec, **change})

    def test_complete_stage_roundtrip_retains_provenance_and_honest_gates(self):
        plan = stage.stage_packages(self.spec, self.source, self.evidence, self.output)
        self.assertEqual(
            plan, stage.verify_staged(self.spec, self.output, self.evidence)
        )
        self.assertGreaterEqual(self.verifier.call_count, 3)
        self.assertFalse(plan["production_release_ready"])
        candidate_metadata = json.loads(
            (self.source / "NPM-PACKAGES.json").read_bytes()
        )
        for key in stage.RETAINED_METADATA:
            self.assertEqual(candidate_metadata[key], plan[key])
        self.assertEqual(
            digest_file(self.source / "NPM-PACKAGES.json"),
            digest_file(self.output / "candidates/NPM-PACKAGES.json"),
        )
        native = archive.inspect_archive(
            self.output / "tarballs" / plan["publish_order"][0]["filename"]
        )
        self.assertFalse(native["json"][stage.VALIDATION]["passed"])
        self.assertNotIn("private", native["json"][stage.MANIFEST])
        with self.assertRaises(ValueError):
            stage.stage_packages(self.spec, self.source, self.evidence, self.output)

    def test_failed_or_wrong_acceptance_leaves_no_final_stage(self):
        self.acceptance["candidate_packages_sha256"] = "f" * 64
        with self.assertRaises(ValueError):
            stage.stage_packages(self.spec, self.source, self.evidence, self.output)
        self.assertFalse(self.output.exists())
        self.verifier.side_effect = ValueError("missing ARM acceptance")
        with self.assertRaises(ValueError):
            stage.stage_packages(self.spec, self.source, self.evidence, self.output)
        self.assertFalse(self.output.exists())

    def test_source_mismatch_and_unsafe_dependency_set_rejected(self):
        with self.assertRaises(ValueError):
            stage.inspect_packages(
                {**self.spec, "source_commit": "d" * 40}, self.source
            )
        row = stage.inspect_packages(self.spec, self.source)["publish_order"][-1]
        inventory = archive.inspect_archive(self.source / "tarballs" / row["filename"])
        inventory["json"][stage.MANIFEST]["optionalDependencies"][
            "airs-harness-win32-x64"
        ] = self.spec["version"]
        with self.assertRaises(ValueError):
            stage._package_identity(self.spec, row, inventory)

    def test_rehashing_forged_staged_launcher_cannot_bypass_candidate_binding(self):
        plan = stage.stage_packages(self.spec, self.source, self.evidence, self.output)
        row = plan["publish_order"][-1]
        target = self.output / "tarballs" / row["filename"]
        original = archive.inspect_archive(target)
        write_tar(
            target,
            [
                (stage.MANIFEST, original["metadata"][stage.MANIFEST], 0o644),
                ("package/bin/launcher.js", b"replaced launcher", 0o755),
            ],
        )
        forged = archive.inspect_archive(target)
        row.update({key: forged[key] for key in ("sha256", "integrity")})
        (self.output / "NPM-PACKAGES.json").write_bytes(encoded(plan))
        with self.assertRaisesRegex(ValueError, "runtime payload"):
            stage.verify_staged(self.spec, self.output, self.evidence)

    def test_symlinked_archive_is_rejected(self):
        record = stage.inspect_packages(self.spec, self.source)["publish_order"][0]
        original = self.source / "tarballs" / record["filename"]
        moved = self.root / "moved.tgz"
        original.rename(moved)
        original.symlink_to(moved)
        with self.assertRaises(ValueError):
            stage.inspect_packages(self.spec, self.source)

    def test_retained_launcher_and_inventory_cannot_be_rebound(self):
        stage.stage_packages(self.spec, self.source, self.evidence, self.output)
        retained = self.output / "candidates"
        metadata = json.loads((retained / "NPM-PACKAGES.json").read_bytes())
        row = metadata["publish_order"][-1]
        target = retained / "tarballs" / row["filename"]
        original = archive.inspect_archive(target)
        write_tar(
            target,
            [
                (stage.MANIFEST, original["metadata"][stage.MANIFEST], 0o644),
                ("package/bin/launcher.js", b"forged candidate", 0o755),
            ],
        )
        forged = archive.inspect_archive(target)
        row.update({key: forged[key] for key in ("sha256", "integrity")})
        (retained / "NPM-PACKAGES.json").write_bytes(encoded(metadata))
        with self.assertRaisesRegex(ValueError, "Acceptance candidate"):
            stage.verify_staged(self.spec, self.output, self.evidence)

    def test_stage_plan_cannot_claim_production_readiness(self):
        plan = stage.stage_packages(self.spec, self.source, self.evidence, self.output)
        plan["production_release_ready"] = True
        (self.output / "NPM-PACKAGES.json").write_bytes(encoded(plan))
        with self.assertRaisesRegex(ValueError, "manifest identity"):
            stage.verify_staged(self.spec, self.output, self.evidence)

    def test_native_platform_and_product_must_match(self):
        candidate = stage.inspect_packages(self.spec, self.source)
        row = candidate["publish_order"][0]
        inventory = candidate["inventories"][row["name"]]
        for key, value in (
            ("os", ["darwin"]),
            ("cpu", ["arm64"]),
            ("cpu", ["x64", "arm64"]),
        ):
            manifest = inventory["json"][stage.MANIFEST]
            original = manifest[key]
            manifest[key] = value
            with (
                self.subTest(key=key, value=value),
                self.assertRaisesRegex(ValueError, "manifest platform"),
            ):
                stage._package_identity(self.spec, row, inventory)
            manifest[key] = original
        inventory["json"][stage.BUILD]["product"] = "Another product"
        with self.assertRaisesRegex(ValueError, "build product"):
            stage._package_identity(self.spec, row, inventory)

    def test_candidate_requires_bundle_tooling_and_dependency_metadata(self):
        path = self.source / "NPM-PACKAGES.json"
        metadata = json.loads(path.read_bytes())
        for key in stage.RETAINED_METADATA:
            missing = dict(metadata)
            missing.pop(key)
            path.write_bytes(encoded(missing))
            with (
                self.subTest(key=key),
                self.assertRaisesRegex(ValueError, "Missing candidate"),
            ):
                stage.inspect_packages(self.spec, self.source)

    def test_staged_bundle_metadata_cannot_change(self):
        plan = stage.stage_packages(self.spec, self.source, self.evidence, self.output)
        for key in stage.RETAINED_METADATA:
            changed = dict(plan)
            changed[key] = {"tampered": True}
            (self.output / "NPM-PACKAGES.json").write_bytes(encoded(changed))
            with (
                self.subTest(key=key),
                self.assertRaisesRegex(ValueError, "manifest identity"),
            ):
                stage.verify_staged(self.spec, self.output, self.evidence)

    def test_stage_rejects_symlinked_output_ancestor(self):
        actual = self.root / "actual"
        actual.mkdir()
        linked = self.root / "linked"
        linked.symlink_to(actual, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "Linked evidence destination"):
            stage.stage_packages(
                self.spec, self.source, self.evidence, linked / "stage"
            )
        self.assertEqual([], list(actual.iterdir()))


if __name__ == "__main__":
    unittest.main()
