"""Adversarial committed-evidence checks using disposable local Git repositories."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import airs_release_git_audit as audit_module
from airs_release_git_audit import audit
from airs_test_release_spec import canonical_digest


class GitEvidenceAudit(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repository = Path(self.temporary.name) / "repository"
        self.repository.mkdir()
        self.git("init", "--object-format=sha1", "--quiet")
        self.root = self.repository / "validation/release"
        self.root.mkdir(parents=True)
        self.acceptance = self.root / "candidate/target"
        self.log = "candidate/target/logs/check.log"
        self.receipt_path = "candidate/target/receipts/check.json"
        self.write(self.log, b"passed fixture\n")
        self.stage = {
            "schema_version": 1,
            "stage": "check",
            "inputs": {"source": "fixture"},
            "dependencies": {},
            "command": ["python3", "fixture.py"],
            "exit_code": 0,
            "outputs": {
                "logs/check.log": {"sha256": self.sha(b"passed fixture\n"), "size": 15}
            },
        }
        self.stage["input_sha256"] = canonical_digest(self.stage["inputs"])
        self.stage["command_sha256"] = canonical_digest(self.stage["command"])
        self.sync_stage()
        self.manifest()

    def git(self, *args, cwd=None):
        return subprocess.check_output(
            [
                "git",
                "-C",
                str(cwd or self.repository),
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                *args,
            ],
            stderr=subprocess.DEVNULL,
        )

    @staticmethod
    def sha(content):
        return hashlib.sha256(content).hexdigest()

    def write(self, relative, payload):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(payload)

    def sync_stage(self):
        payload = json.dumps(self.stage).encode()
        self.write(self.receipt_path, payload)
        self.write(
            "candidate/target/ACCEPTANCE.json",
            json.dumps({"stage_receipts": {"check": self.sha(payload)}}).encode(),
        )

    def manifest(self):
        rows = []
        for path in sorted(self.root.rglob("*")):
            if path.is_file() and path.name != "SHA256SUMS":
                rows.append(
                    f"{self.sha(path.read_bytes())}  {path.relative_to(self.root).as_posix()}\n"
                )
        self.write("SHA256SUMS", "".join(rows).encode())

    def commit(self):
        self.git("add", "-A")
        self.git("commit", "--quiet", "--allow-empty", "-m", "fixture")
        return self.git("rev-parse", "HEAD").decode().strip()

    def check(self, commit):
        return audit(self.repository, commit, "validation/release")

    def test_valid_commit_is_reproducible_without_checkout_or_worktree_bytes(self):
        commit = self.commit()
        expected = self.check(commit)
        self.assertEqual(
            (expected["files_checked"], expected["stage_receipts_checked"]), (3, 1)
        )
        self.assertFalse(expected["product_acceptance_claimed"])
        (self.root / self.log).write_bytes(b"uncommitted replacement")
        self.assertEqual(self.check(commit), expected)
        (self.root / self.log).unlink()
        self.assertEqual(self.check(commit), expected)
        clone = Path(self.temporary.name) / "bare.git"
        self.git("clone", "--bare", "--quiet", str(self.repository), str(clone))
        self.assertEqual(audit(clone, commit, "validation/release"), expected)

    def test_ignored_and_staged_log_cannot_rescue_committed_reference(self):
        (self.repository / ".gitignore").write_text("*.log\n")
        commit = self.commit()
        with self.assertRaisesRegex(ValueError, "manifest inventory"):
            self.check(commit)
        self.git("add", "-f", str(self.root / self.log))
        with self.assertRaisesRegex(ValueError, "manifest inventory"):
            self.check(commit)

    def test_wrong_committed_bytes_cannot_be_rescued_locally(self):
        (self.root / self.log).write_bytes(b"incorrect committed log")
        commit = self.commit()
        (self.root / self.log).write_bytes(b"passed fixture\n")
        with self.assertRaisesRegex(ValueError, "manifest digest mismatch"):
            self.check(commit)

    def test_extra_committed_file_not_in_manifest_is_rejected(self):
        self.write("omitted.txt", b"not inventoried")
        with self.assertRaisesRegex(ValueError, "manifest inventory"):
            self.check(self.commit())

    def test_manifest_paths_duplicates_and_digests_are_rejected(self):
        original = (self.root / "SHA256SUMS").read_bytes()
        for line in [
            b"0" * 64 + b"  ../outside\n",
            b"0" * 64 + b"  /absolute\n",
            b"0" * 64 + b"  path\\escape\n",
            b"0" * 64 + b"  a//b\n",
            b"0" * 64 + b"  SHA256SUMS\n",
            original.splitlines(keepends=True)[0],
            b"X" * 64 + b"  bad.txt\n",
        ]:
            with self.subTest(line=line):
                self.write("SHA256SUMS", original + line)
                with self.assertRaises(ValueError):
                    self.check(self.commit())

    def test_unsafe_roots_and_noncommit_revisions_are_rejected(self):
        commit = self.commit()
        for root in [
            "../validation",
            "/validation",
            "validation\\release",
            ":(glob)validation",
            "validation//release",
            "validation/./release",
            "validation/\x00release",
        ]:
            with self.subTest(root=root), self.assertRaises(ValueError):
                audit(self.repository, commit, root)
        blob = (
            self.git("rev-parse", f"{commit}:validation/release/SHA256SUMS")
            .decode()
            .strip()
        )
        for revision in ["HEAD", commit[:12], "--help", "0" * 40, blob, f"{commit}~1"]:
            with self.subTest(revision=revision), self.assertRaises(ValueError):
                self.check(revision)

    def test_symlink_root_and_output_are_never_followed(self):
        commit = self.commit()
        (self.repository / "linked-root").symlink_to(
            "validation/release", target_is_directory=True
        )
        linked = self.commit()
        with self.assertRaisesRegex(ValueError, "non-regular"):
            audit(self.repository, linked, "linked-root")
        (self.root / self.log).unlink()
        (self.root / self.log).symlink_to("../ACCEPTANCE.json")
        with self.assertRaisesRegex(ValueError, "non-regular"):
            self.check(self.commit())
        self.assertTrue(self.check(commit)["passed"])

    def test_gitlink_is_rejected_and_executable_regular_file_is_allowed(self):
        (self.root / self.log).chmod(0o755)
        commit = self.commit()
        self.assertTrue(self.check(commit)["passed"])
        self.git(
            "update-index",
            "--add",
            "--cacheinfo",
            f"160000,{commit},validation/release/submodule",
        )
        self.git("commit", "--quiet", "-m", "gitlink")
        with self.assertRaisesRegex(ValueError, "non-regular"):
            self.check(self.git("rev-parse", "HEAD").decode().strip())

    def test_stage_output_input_command_and_dependency_corruption_reject(self):
        original = json.dumps(self.stage)
        mutations = [
            lambda row: row["outputs"]["logs/check.log"].update(size=True),
            lambda row: row["outputs"]["logs/check.log"].update(size=-1),
            lambda row: row["outputs"]["logs/check.log"].update(size=17),
            lambda row: row["outputs"]["logs/check.log"].update(sha256="0" * 64),
            lambda row: row.update(
                outputs={"../outside": {"sha256": "0" * 64, "size": 0}}
            ),
            lambda row: row.update(input_sha256="0" * 64),
            lambda row: row.update(command_sha256="0" * 64),
            lambda row: row.update(command=[]),
            lambda row: row.update(dependencies={"missing": "0" * 64}),
            lambda row: row.update(dependencies={"../check": "0" * 64}),
            lambda row: row.update(dependencies={"check": "0" * 64}),
            lambda row: row.update(exit_code=True),
        ]
        for index, mutation in enumerate(mutations):
            with self.subTest(index=index):
                self.stage = json.loads(original)
                mutation(self.stage)
                self.sync_stage()
                self.manifest()
                with self.assertRaises(ValueError):
                    self.check(self.commit())

    def test_acceptance_references_and_duplicate_json_fields_reject(self):
        for references in [{}, {"check": "0" * 64}, {"missing": "0" * 64}]:
            self.write(
                "candidate/target/ACCEPTANCE.json",
                json.dumps({"stage_receipts": references}).encode(),
            )
            self.manifest()
            with self.subTest(references=references), self.assertRaises(ValueError):
                self.check(self.commit())
        self.sync_stage()
        payload = (self.root / self.receipt_path).read_bytes()
        self.write(self.receipt_path, payload[:-1] + b',"exit_code":0}')
        self.manifest()
        with self.assertRaisesRegex(ValueError, "Duplicate JSON"):
            self.check(self.commit())

    def test_limits_fail_before_unbounded_blob_read(self):
        commit = self.commit()
        for constant, limit in [
            ("MAX_FILES", 1),
            ("MAX_BLOB", 1),
            ("MAX_TOTAL", 1),
            ("MAX_JSON", 1),
            ("MAX_TREE", 1),
        ]:
            with (
                self.subTest(constant=constant),
                patch.object(audit_module, constant, limit),
                self.assertRaises(ValueError),
            ):
                self.check(commit)

    def test_replacement_refs_and_git_environment_cannot_redirect_audit(self):
        commit = self.commit()
        expected = self.check(commit)
        (self.root / self.log).write_bytes(b"wrong replacement content")
        replacement = self.commit()
        self.git("replace", commit, replacement)
        with patch.dict(
            "os.environ",
            {
                "GIT_DIR": "/missing-repository",
                "GIT_WORK_TREE": "/missing",
                "GIT_CONFIG_COUNT": "invalid",
            },
        ):
            self.assertEqual(self.check(commit), expected)

    def test_failed_cli_does_not_overwrite_receipt(self):
        commit = self.commit()
        output = Path(self.temporary.name) / "receipt.json"
        output.write_text("preserve me")
        result = subprocess.run(
            [
                sys.executable,
                str(Path(audit_module.__file__)),
                "--repository",
                str(self.repository),
                "--commit",
                commit,
                "--root",
                "missing",
                "--output",
                str(output),
            ],
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 1)
        self.assertEqual(output.read_text(), "preserve me")


if __name__ == "__main__":
    unittest.main()
