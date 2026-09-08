import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import airs_macos_artifact as artifact


class ImmutableMacArtifact(unittest.TestCase):
    def test_legacy_artifact_does_not_invent_its_build_profile_or_builder(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "airs-harness").write_bytes(b"legacy CLI")
            (root / "store_acceptance").write_bytes(b"legacy fixture")
            manifest = artifact.create(root, "a" * 40, "1")
            archive = root / "legacy.tar.gz"
            with tarfile.open(archive, "w:gz") as tar:
                for name in ["airs-harness", "store_acceptance", "runtime-source.txt"]:
                    tar.add(root / name, arcname=name)
            with patch.dict("os.environ", GITHUB_SHA="b" * 40):
                result = artifact.restore(
                    archive,
                    root / "out",
                    "a" * 40,
                    artifact.digest(archive),
                    manifest["binary_sha256"],
                    manifest["fixture_sha256"],
                )
            self.assertEqual(result["cli_opt_level"], "unrecorded")
            self.assertIsNone(result["build_tooling_source"])
            self.assertIsNone(result["compilation_attempt"])
            self.assertFalse(result["release_ready"])

    def test_roundtrip_and_each_independent_identity_fail_closed(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "airs-harness").write_bytes(b"synthetic CLI")
            (root / "store_acceptance").write_bytes(b"synthetic fixture")
            manifest = artifact.create(root, "a" * 40, "1")
            archive = root / "unvalidated-macos-executables.tar.gz"
            for mismatch in [
                None,
                "runtime_source",
                "archive_sha256",
                "binary_sha256",
                "fixture_sha256",
            ]:
                with self.subTest(mismatch=mismatch):
                    expected = dict(manifest)
                    if mismatch:
                        expected[mismatch] = "b" * len(expected[mismatch])
                    output = root / str(mismatch)
                    args = [
                        archive,
                        output,
                        expected["runtime_source"],
                        expected["archive_sha256"],
                        expected["binary_sha256"],
                        expected["fixture_sha256"],
                    ]
                    if mismatch:
                        with self.assertRaises(ValueError):
                            artifact.restore(*args)
                    else:
                        restored = artifact.restore(*args)
                        self.assertEqual(restored, manifest)
                        self.assertEqual(
                            (output / "airs-harness").read_bytes(), b"synthetic CLI"
                        )

    def test_links_duplicates_and_publishable_manifests_are_rejected(self):
        for mutation in ["link", "duplicate", "publishable"]:
            with (
                self.subTest(mutation=mutation),
                tempfile.TemporaryDirectory() as temporary,
            ):
                root = Path(temporary)
                (root / "airs-harness").write_bytes(b"fixture")
                (root / "store_acceptance").write_bytes(b"fixture")
                manifest = artifact.create(root, "a" * 40, "1")
                archive = root / "unvalidated-macos-executables.tar.gz"
                with tarfile.open(archive) as tar:
                    entries = [
                        (member, tar.extractfile(member).read()) for member in tar
                    ]
                with tarfile.open(archive, "w:gz") as tar:
                    for member, value in entries:
                        if member.name == "airs-harness" and mutation == "link":
                            member.type, member.linkname = tarfile.SYMTYPE, "/outside"
                        if (
                            member.name == "CANDIDATE.json"
                            and mutation == "publishable"
                        ):
                            record = json.loads(value)
                            record["private_candidate"] = False
                            value = json.dumps(record).encode()
                            member.size = len(value)
                        tar.addfile(member, io.BytesIO(value))
                        if member.name == "airs-harness" and mutation == "duplicate":
                            tar.addfile(member, io.BytesIO(value))
                with self.assertRaises(ValueError):
                    artifact.restore(
                        archive,
                        root / "out",
                        "a" * 40,
                        artifact.digest(archive),
                        manifest["binary_sha256"],
                        manifest["fixture_sha256"],
                    )


if __name__ == "__main__":
    unittest.main()
