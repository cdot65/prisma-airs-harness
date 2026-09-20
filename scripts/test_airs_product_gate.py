"""Product success cannot hide skipped features, broken rollback or GNU failures."""

import copy
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from airs_product_gate import REQUIRED_CASES, product_gate
from airs_product_workspace import verify_workspace
from airs_release_receipts import atomic_json
from airs_test_release_spec import TARGETS, digest_file, load_json
from test_airs_release_acceptance import evidence_set, result_for
from test_airs_test_release_stage import candidates, sample_spec


class ProductGate(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.spec = sample_spec()
        self.spec.update(
            previous_version="0.1.1",
            previous_release={
                "source_commit": "d" * 40,
                "platforms": [
                    {"target": target, "binary_sha256": "e" * 64} for target in TARGETS
                ],
            },
        )
        self.packages = self.root / "packages"
        candidates(self.packages, self.spec)
        self.acceptance = self.root / "acceptance"
        self.workspace = self.root / "WORKSPACE.json"
        self.write_workspace()
        self.build_evidence()

    def result(self, name, spec, target):
        value = result_for(name, spec, target)
        if name in REQUIRED_CASES:
            value.update(
                test_ids=sorted(REQUIRED_CASES[name]) + ["other.executed.test"],
                tests_run=len(REQUIRED_CASES[name]) + 1,
            )
        if name == "upgrade":
            candidate = next(
                row["binary_sha256"]
                for row in spec["platforms"]
                if row["target"] == target
            )
            value.update(
                schema_version=2,
                roundtrip={
                    "passed": True,
                    "previous_version": "0.1.1",
                    "restored_version": "0.1.1",
                    "candidate_version": spec["version"],
                    "previous_binary_sha256": "e" * 64,
                    "restored_binary_sha256": "e" * 64,
                    "candidate_binary_sha256": candidate,
                    "candidate_source_commit": spec["source_commit"],
                    "previous_source_commit": spec["previous_release"]["source_commit"],
                    "command_links_preserved": True,
                    "configuration_preserved": True,
                    "real_conversation_preserved": True,
                    "inference_credential_reused": True,
                    "mcp_credential_reused": True,
                    "native_cleanup_completed": True,
                    "real_mcp_turns": 3,
                    "uninstall_used": False,
                    "force_used": False,
                    "production_acceptance": False,
                },
            )
        return value

    def build_evidence(
        self, mutate=None, installation="candidate", verification_tooling_commit=None
    ):
        import shutil

        shutil.rmtree(self.acceptance, ignore_errors=True)

        def result(name, spec, target):
            value = self.result(name, spec, target)
            if mutate:
                mutate(name, value)
            return value

        with patch("test_airs_release_acceptance.result_for", side_effect=result):
            evidence_set(
                self.acceptance,
                self.packages,
                self.spec,
                installation,
                verification_tooling_commit,
            )

    def write_workspace(self, failed=False):
        failed_name = "codex-disabled::integration requires_remote_service"
        log = "Summary [1.0s] 3 tests run: " + (
            "2 passed, 1 failed, 0 skipped\n  TRY 2 FAIL [ 0.2s] (3/3) "
            + failed_name
            + "\n"
            if failed
            else "3 passed, 0 skipped\n"
        )
        files = {}
        for key, text in {
            "log": log,
            "source": self.spec["source_commit"],
            "tooling": self.spec["tooling_commit"],
            "scope": "full-workspace",
        }.items():
            p = self.root / (key + ".txt")
            p.write_text(text + "\n")
            files[key] = {"path": p.name, "sha256": digest_file(p)}
        review = self.root / "review.md"
        review.write_text(
            "Independent review: this synthetic case requires a disabled upstream-only service.\n"
        )
        value = {
            "schema_version": 1,
            "scope": "full-workspace",
            "source_commit": self.spec["source_commit"],
            "tooling_commit": self.spec["tooling_commit"],
            "status": "failure" if failed else "success",
            "files": files,
            "counts": {
                "tests_run": 3,
                "passed": 2 if failed else 3,
                "failed": 1 if failed else 0,
                "skipped": 0,
            },
            "failures": [failed_name] if failed else [],
            "failure_reviews": [
                {
                    "test": failed_name,
                    "product_blocking": False,
                    "reason": review.read_text(),
                    "evidence": {"path": review.name, "sha256": digest_file(review)},
                }
            ]
            if failed
            else [],
        }
        atomic_json(self.workspace, value)

    def run_gate(self, installation="candidate"):
        return product_gate(self.spec, self.acceptance, self.workspace, installation)

    def test_real_receipt_chain_preserves_product_and_workspace_distinction(self):
        self.write_workspace(failed=True)
        result = self.run_gate()
        self.assertTrue(result["passed"])
        self.assertEqual(result["workspace_diagnostics"]["status"], "failure")
        self.assertEqual(len(result["upgrade_rollback"]), 3)
        self.assertFalse(result["stable_promotion_authorized"])
        self.assertFalse(result["production_sso_servicenow_claimed"])
        log = self.acceptance / next(iter(TARGETS)) / "logs/doctor.log"
        log.write_text("changed after stage verification")
        with self.assertRaises(ValueError):
            self.run_gate()

    def test_required_feature_case_cannot_be_missing_or_skipped(self):
        for stage in REQUIRED_CASES:
            for skipped in (True, False):
                with self.subTest(stage=stage, skipped=skipped):

                    def mutate(name, value):
                        if name == stage:
                            case = sorted(REQUIRED_CASES[stage])[0]
                            if skipped:
                                value["skipped"] = [
                                    {"test": case, "reason": "unavailable"}
                                ]
                            else:
                                value["test_ids"].remove(case)
                                value["tests_run"] -= 1

                    self.build_evidence(mutate)
                    with self.assertRaises(ValueError):
                        self.run_gate()

    def test_wrong_or_config_only_rollback_cannot_pass(self):
        for key, replacement in [
            ("real_conversation_preserved", False),
            ("inference_credential_reused", False),
            ("mcp_credential_reused", False),
            ("restored_binary_sha256", "f" * 64),
            ("candidate_binary_sha256", "f" * 64),
            ("candidate_source_commit", "f" * 40),
            ("native_cleanup_completed", False),
            ("force_used", True),
        ]:
            with self.subTest(key=key):

                def mutate(name, value):
                    if name == "upgrade":
                        value["roundtrip"][key] = replacement

                self.build_evidence(mutate)
                with self.assertRaises(ValueError):
                    self.run_gate()

    def test_phase_and_native_target_are_not_interchangeable(self):
        with self.assertRaises(ValueError):
            self.run_gate("registry")
        self.build_evidence(installation="registry")
        self.assertEqual(self.run_gate("registry")["phase"], "registry")
        (self.acceptance / next(iter(TARGETS)) / "ACCEPTANCE.json").unlink()
        with self.assertRaises(ValueError):
            self.run_gate("registry")

    def test_registry_validator_revision_is_explicit_and_phase_bound(self):
        revision = "f" * 40
        self.build_evidence(
            installation="registry", verification_tooling_commit=revision
        )
        with self.assertRaises(ValueError):
            self.run_gate("registry")
        result = product_gate(
            self.spec, self.acceptance, self.workspace, "registry", revision
        )
        self.assertEqual(result["verification_tooling_commit"], revision)
        with self.assertRaises(ValueError):
            product_gate(
                self.spec, self.acceptance, self.workspace, "candidate", revision
            )

    def test_incomplete_focused_or_stale_workspace_is_not_full_acceptance(self):
        original = load_json(self.workspace)
        for key, value in [
            ("scope", "focused"),
            ("status", "pending"),
            ("source_commit", "f" * 40),
            ("counts", {"tests_run": 3, "passed": 3, "failed": 1, "skipped": 0}),
        ]:
            with self.subTest(key=key):
                document = copy.deepcopy(original)
                document[key] = value
                atomic_json(self.workspace, document)
                with self.assertRaises(ValueError):
                    verify_workspace(self.workspace, self.spec)
        atomic_json(self.workspace, original)
        (self.root / "scope.txt").write_text("test(airs_harness)\n")
        with self.assertRaises(ValueError):
            verify_workspace(self.workspace, self.spec)

    def test_workspace_failures_need_current_nonblocking_review_evidence(self):
        self.write_workspace(failed=True)
        original = load_json(self.workspace)
        for key, value in [
            ("failure_reviews", []),
            ("failures", []),
            ("status", "success"),
        ]:
            document = copy.deepcopy(original)
            document[key] = value
            atomic_json(self.workspace, document)
            with self.assertRaises(ValueError):
                verify_workspace(self.workspace, self.spec)
        document = copy.deepcopy(original)
        document["failure_reviews"][0]["product_blocking"] = True
        atomic_json(self.workspace, document)
        with self.assertRaises(ValueError):
            verify_workspace(self.workspace, self.spec)
        atomic_json(self.workspace, original)
        (self.root / "review.md").unlink()
        with self.assertRaises(ValueError):
            verify_workspace(self.workspace, self.spec)


if __name__ == "__main__":
    unittest.main()
