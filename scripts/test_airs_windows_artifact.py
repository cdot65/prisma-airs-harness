import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest

import airs_windows_artifact as artifact


class ImmutableWindowsArtifact(unittest.TestCase):
    def test_roundtrip_and_each_independent_identity_fail_closed(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "airs-harness.exe").write_bytes(b"synthetic CLI")
            (root / "store_acceptance.exe").write_bytes(b"synthetic fixture")
            manifest = artifact.create(root, "a" * 40, "1")
            self.assertEqual(
                (root / "runtime-source.txt").read_bytes(), b"a" * 40 + b"\n"
            )
            archive = root / "unvalidated-windows-executables.tar.gz"
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
                            (output / "airs-harness.exe").read_bytes(), b"synthetic CLI"
                        )

    def test_links_duplicates_and_publishable_manifests_are_rejected(self):
        for mutation in [
            "link",
            "duplicate",
            "publishable",
            "target",
            "signing",
            "missing",
        ]:
            with (
                self.subTest(mutation=mutation),
                tempfile.TemporaryDirectory() as temporary,
            ):
                root = Path(temporary)
                (root / "airs-harness.exe").write_bytes(b"fixture")
                (root / "store_acceptance.exe").write_bytes(b"fixture")
                manifest = artifact.create(root, "a" * 40, "1")
                archive = root / "unvalidated-windows-executables.tar.gz"
                with tarfile.open(archive) as tar:
                    entries = [
                        (member, tar.extractfile(member).read()) for member in tar
                    ]
                with tarfile.open(archive, "w:gz") as tar:
                    for member, value in entries:
                        if member.name == "airs-harness.exe" and mutation == "link":
                            member.type, member.linkname = tarfile.SYMTYPE, "/outside"
                        if member.name == "CANDIDATE.json":
                            if mutation == "missing":
                                continue
                            record = json.loads(value)
                            if mutation == "publishable":
                                record["private_candidate"] = False
                            elif mutation == "target":
                                record["target"] = "aarch64-apple-darwin"
                            elif mutation == "signing":
                                record["signing"]["production_signing"] = True
                            value = json.dumps(record).encode()
                            member.size = len(value)
                        tar.addfile(member, io.BytesIO(value))
                        if (
                            member.name == "airs-harness.exe"
                            and mutation == "duplicate"
                        ):
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
