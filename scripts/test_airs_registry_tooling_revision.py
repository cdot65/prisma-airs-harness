"""A post-publication validator repair must not rewrite candidate provenance."""

from pathlib import Path
import json
import subprocess
import sys
import tempfile
import unittest

from airs_release_acceptance import run_acceptance, verify_acceptance_set
from airs_release_receipts import atomic_json
from airs_test_release_spec import TARGETS, load_json
from test_airs_release_acceptance import evidence_set
from test_airs_test_release_stage import candidates, sample_spec


class RegistryToolingRevisionTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.spec = sample_spec()
        self.revision = "f" * 40
        self.packages = self.root / "packages"
        candidates(self.packages, self.spec)
        self.evidence = self.root / "registry"
        evidence_set(self.evidence, self.packages, self.spec, "registry", self.revision)

    def test_explicit_revision_preserves_original_spec_and_candidate_tooling(self):
        result = verify_acceptance_set(
            self.spec, self.evidence, "registry", self.revision
        )
        self.assertEqual(self.spec["tooling_commit"], result["tooling_commit"])
        self.assertEqual(self.revision, result["verification_tooling_commit"])
        self.assertEqual(3, len(result["platforms"]))
        with self.assertRaises(ValueError):
            verify_acceptance_set(self.spec, self.evidence, "registry")
        with self.assertRaises(ValueError):
            verify_acceptance_set(self.spec, self.evidence, "registry", "e" * 40)

    def test_candidate_cannot_accept_a_registry_revision(self):
        with self.assertRaisesRegex(ValueError, "registry"):
            verify_acceptance_set(self.spec, self.evidence, "candidate", self.revision)
        output = self.root / "never-created"
        with self.assertRaisesRegex(ValueError, "registry"):
            run_acceptance(
                self.spec,
                self.packages,
                self.root / "missing-scripts",
                output,
                verification_tooling_commit=self.revision,
            )
        self.assertFalse(output.exists())

    def test_revision_must_be_a_full_commit_and_match_each_manifest(self):
        for invalid in ("HEAD", "f" * 39, "f" * 40 + "\n", ""):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                verify_acceptance_set(self.spec, self.evidence, "registry", invalid)
        path = self.evidence / next(iter(TARGETS)) / "TOOLING.json"
        document = load_json(path)
        document["tooling_commit"] = self.spec["tooling_commit"]
        atomic_json(path, document)
        with self.assertRaisesRegex(ValueError, "tooling"):
            verify_acceptance_set(self.spec, self.evidence, "registry", self.revision)

    def test_cli_requires_explicit_verification_revision(self):
        spec_path = self.root / "spec.json"
        atomic_json(spec_path, self.spec)
        command = [
            sys.executable,
            "-O",
            str(Path(__file__).with_name("airs_test_release.py")),
            "verify-all",
            "--spec",
            str(spec_path),
            "--acceptance",
            str(self.evidence),
            "--installation",
            "registry",
            "--verification-tooling-commit",
            self.revision,
        ]
        result = subprocess.run(command, capture_output=True, text=True)
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(
            self.revision, json.loads(result.stdout)["verification_tooling_commit"]
        )
        rejected = subprocess.run(command[:-2], capture_output=True, text=True)
        self.assertEqual(1, rejected.returncode)
        self.assertIn("tooling revision mismatch", rejected.stderr)


if __name__ == "__main__":
    unittest.main()
