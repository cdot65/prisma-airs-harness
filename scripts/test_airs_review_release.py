"""Synthetic contract checks; no actual release readiness is asserted by these tests."""

import copy
import hashlib
from pathlib import Path
import tempfile
import unittest

from airs_review_release import SCOPE, validate_release


class ReviewReleaseContract(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.binding = dict(
            binary_sha256="a" * 64,
            target="x86_64-unknown-linux-musl",
            version="0.1.0-alpha.10",
            source_commit="b" * 40,
            evidence_root=self.root,
        )
        self.document = {
            "schema_version": 1,
            "scope": SCOPE,
            "passed": True,
            "release_ready": True,
            "full_authentication_release_ready": False,
            "binary_sha256": "a" * 64,
            "target": self.binding["target"],
            "product_version": self.binding["version"],
            "source_commit": "b" * 40,
            "independent_review": {
                "reviewer": "synthetic-reviewer",
                "scope": SCOPE,
                "verdict": "pass",
                "score": 9,
            },
            "signing": {"kind": "linux-provenance-checksums"},
            "evidence": [],
        }
        for role in [
            "installed-runtime",
            "package-integrity",
            "independent-review",
            "native-provenance",
        ]:
            self.evidence(role)

    def evidence(self, role):
        path = self.root / (role + ".txt")
        path.write_text("Synthetic contract-test evidence: " + role)
        self.document["evidence"].append(
            {
                "role": role,
                "path": path.name,
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            }
        )

    def validate(self, document=None):
        return validate_release(
            self.document if document is None else document, **self.binding
        )

    def test_linux_arm64_uses_the_linux_provenance_contract(self):
        self.binding["target"] = "aarch64-unknown-linux-musl"
        self.document["target"] = self.binding["target"]
        self.validate()

    def mac(self):
        self.binding["target"] = "aarch64-apple-darwin"
        self.document["target"] = self.binding["target"]
        self.document["signing"] = {
            "kind": "developer-id-application",
            "verified": True,
            "hardened_runtime": True,
            "signed_binary_sha256": "a" * 64,
            "team_identifier": "TESTTEAM01",
            "notarization": {
                "status": "Accepted",
                "archive_sha256": "c" * 64,
                "submission_id": "00000000-0000-4000-8000-000000000001",
            },
        }
        self.evidence("developer-id-signature")
        self.evidence("apple-notarization")

    def test_exact_linux_and_signed_mac_attestations(self):
        result = self.validate()
        self.assertEqual(result["scope"], SCOPE)
        self.assertIs(result["full_authentication_release_ready"], False)
        self.mac()
        self.assertEqual(len(self.validate()["evidence_roles_verified"]), 6)

    def test_bindings_failure_flags_and_stale_scope_are_rejected(self):
        changes = {
            "passed": False,
            "release_ready": False,
            "full_authentication_release_ready": True,
            "schema_version": True,
            "scope": "complete-production",
            "binary_sha256": "c" * 64,
            "source_commit": "c" * 40,
            "target": "x86_64-apple-darwin",
            "product_version": "0.1.0-alpha.9",
            "private": False,
            "release_status": "unsigned-unvalidated-candidate",
        }
        for key, value in changes.items():
            with self.subTest(key=key):
                document = copy.deepcopy(self.document)
                document[key] = value
                with self.assertRaises(ValueError):
                    self.validate(document)
        for key in ["independent_review", "evidence", "signing"]:
            with self.subTest(missing=key):
                document = copy.deepcopy(self.document)
                del document[key]
                with self.assertRaises(ValueError):
                    self.validate(document)

    def test_review_score_is_scoped_and_numeric(self):
        for score in [True, "9", 8.999, 11, float("nan"), float("inf")]:
            with self.subTest(score=score):
                document = copy.deepcopy(self.document)
                document["independent_review"]["score"] = score
                with self.assertRaises(ValueError):
                    self.validate(document)
        self.document["independent_review"]["scope"] = "authentication-full-release"
        with self.assertRaises(ValueError):
            self.validate()

    def test_evidence_hash_missing_role_duplicates_and_paths_fail(self):
        record = self.document["evidence"][0]
        for path in ["../escape", "/absolute", "a\\b", "A:stream"]:
            with self.subTest(path=path):
                document = copy.deepcopy(self.document)
                document["evidence"][0]["path"] = path
                with self.assertRaises(ValueError):
                    self.validate(document)
        document = copy.deepcopy(self.document)
        document["evidence"][0]["sha256"] = "0" * 64
        with self.assertRaises(ValueError):
            self.validate(document)
        document = copy.deepcopy(self.document)
        document["evidence"][0]["role"] = "other-evidence"
        with self.assertRaises(ValueError):
            self.validate(document)
        document = copy.deepcopy(self.document)
        document["evidence"].append(document["evidence"][0])
        with self.assertRaises(ValueError):
            self.validate(document)
        (self.root / record["path"]).write_text("Changed after review")
        with self.assertRaises(ValueError):
            self.validate()

    def test_linked_or_oversized_evidence_is_not_read(self):
        file = self.root / self.document["evidence"][0]["path"]
        original = self.root / "original"
        file.rename(original)
        try:
            file.symlink_to(original)
        except OSError:
            self.skipTest("Symlink creation unavailable in this session")
        with self.assertRaises(ValueError):
            self.validate()
        file.unlink()
        with file.open("wb") as stream:
            stream.truncate(16 * 1024 * 1024 + 1)
        with self.assertRaises(ValueError):
            self.validate()

    def test_mac_acceptance_does_not_replace_signature_or_notary_bindings(self):
        self.mac()
        changes = [
            ("verified", False),
            ("hardened_runtime", False),
            ("signed_binary_sha256", "d" * 64),
            ("team_identifier", ""),
        ]
        for key, value in changes:
            with self.subTest(key=key):
                document = copy.deepcopy(self.document)
                document["signing"][key] = value
                with self.assertRaises(ValueError):
                    self.validate(document)
        for key, value in [
            ("status", "In Progress"),
            ("submission_id", "invalid"),
            ("archive_sha256", ""),
        ]:
            with self.subTest(key=key):
                document = copy.deepcopy(self.document)
                document["signing"]["notarization"][key] = value
                with self.assertRaises(ValueError):
                    self.validate(document)


if __name__ == "__main__":
    unittest.main()
