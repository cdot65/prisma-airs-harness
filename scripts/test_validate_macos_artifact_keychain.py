import contextlib
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import validate_macos_artifact_keychain as diagnostic


class PreservedKeychainDiagnostic(unittest.TestCase):
    def archive(self, root, *, link=False, duplicate=False):
        archive = root / "artifact.tar.gz"
        with tarfile.open(archive, "w:gz") as tar:
            for name, value in [
                ("airs-harness", b"fixture"),
                ("runtime-source.txt", b"a" * 40 + b"\n"),
            ]:
                member = tarfile.TarInfo(name)
                member.size = len(value)
                if link and name == "airs-harness":
                    member.type = tarfile.SYMTYPE
                    member.linkname = "/outside-fixture"
                tar.addfile(member, io.BytesIO(value))
                if duplicate and name == "airs-harness":
                    tar.addfile(member, io.BytesIO(value))
        return archive

    def test_exact_source_and_digest_required_and_links_duplicates_rejected(self):
        for kind in ["valid", "source", "digest", "link", "duplicate"]:
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                archive = self.archive(
                    root, link=kind == "link", duplicate=kind == "duplicate"
                )
                output = root / "output"
                output.mkdir()
                source = ("b" if kind == "source" else "a") * 40
                digest = (
                    "0" * 64
                    if kind == "digest"
                    else hashlib.sha256(b"fixture").hexdigest()
                )
                if kind == "valid":
                    binary = diagnostic.extract_verified(
                        archive, output, source, digest
                    )
                    self.assertEqual(binary.read_bytes(), b"fixture")
                else:
                    with self.assertRaises(ValueError):
                        diagnostic.extract_verified(archive, output, source, digest)

    def test_fixture_failure_never_retains_secret_output_or_exception(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = self.archive(root)
            receipt = root / "receipt.json"
            canary = "PRIVATE-CREDENTIAL-CANARY"

            def fail():
                print(canary)
                print(canary, file=sys.stderr)
                raise RuntimeError(canary)

            argv = [
                "diagnostic",
                "--archive",
                str(archive),
                "--source-commit",
                "a" * 40,
                "--binary-sha256",
                hashlib.sha256(b"fixture").hexdigest(),
                "--build-run",
                "123",
                "--receipt",
                str(receipt),
            ]
            output = io.StringIO()
            results = [
                subprocess.CompletedProcess([], 0, stdout="arm64\n", stderr=""),
                subprocess.CompletedProcess([], 0),
                subprocess.CompletedProcess(
                    [], 0, stdout="", stderr="Signature=adhoc\n"
                ),
            ]
            with (
                patch.object(sys, "argv", argv),
                patch.object(sys, "platform", "darwin"),
                patch.object(diagnostic.subprocess, "run", side_effect=results),
                patch.dict(
                    sys.modules,
                    {
                        "validate_airs_macos_keychain": SimpleNamespace(main=fail),
                    },
                ),
                contextlib.redirect_stdout(output),
            ):
                self.assertEqual(diagnostic.main(), 1)
            self.assertNotIn(canary, output.getvalue() + receipt.read_text())
            result = json.loads(receipt.read_text())
            self.assertFalse(result["passed"])
            self.assertFalse(result["release_ready"])
            self.assertFalse(result["owner_device_tested"])
            self.assertTrue(result["artifact_verified"])
            self.assertEqual(result["failure_class"], "RuntimeError")


if __name__ == "__main__":
    unittest.main()
