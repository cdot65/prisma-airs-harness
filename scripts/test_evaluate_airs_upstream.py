import copy
import hashlib
from pathlib import Path
import tempfile
import unittest

from evaluate_airs_upstream import CASES, COMPONENTS, evaluate


class EvaluationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        proof = self.root / "fixture.txt"
        proof.write_text("Synthetic evaluator fixture, not release evidence.\n")
        evidence = [
            {
                "path": proof.name,
                "sha256": hashlib.sha256(proof.read_bytes()).hexdigest(),
            }
        ]
        common = {
            "status": "passed",
            "source_commit": "a" * 40,
            "artifacts": {"linux-x64": "b" * 64, "darwin-arm64": "c" * 64},
            "evidence": evidence,
        }
        receipt = {
            **common,
            "expected": "fixture",
            "actual": "fixture",
            "invocation": "synthetic",
            "timestamp": "2026-09-13T00:00:00Z",
            "platform": "synthetic",
            "configuration_fingerprint": "fixture",
            "fixture_provenance": "unit test",
            "reviewer_disposition": "fixture",
        }
        self.ledger = {
            "source_commit": common["source_commit"],
            "artifacts": common["artifacts"],
            "implementer": "fixture implementer",
            "receipts": {case: copy.deepcopy(receipt) for case in CASES},
            "scores": {
                name: {"points": maximum, "reason": "fixture"}
                for name, (maximum, _) in COMPONENTS.items()
            },
            "independent_review": {**common, "reviewer": "fixture reviewer"},
            "owner_acceptance": {**common, "reviewer": "fixture owner"},
            "findings": [],
        }

    def test_complete_ledger_and_threshold(self):
        self.assertEqual(evaluate(self.ledger, self.root)["status"], "GO")
        self.ledger["scores"]["routing"]["points"] = 0
        self.assertEqual(evaluate(self.ledger, self.root)["status"], "GO")
        self.ledger["scores"]["release"]["points"] = 4
        self.assertEqual(evaluate(self.ledger, self.root)["status"], "NO-GO")

    def test_high_score_cannot_hide_missing_gate(self):
        del self.ledger["receipts"]["endpoint_inventory"]
        result = evaluate(self.ledger, self.root)
        self.assertEqual(result["score"], 100)
        self.assertEqual(result["status"], "NO-GO")
        self.assertEqual(result["gates"]["G1"], "blocked")

    def test_wrong_source_or_artifact_receives_no_credit(self):
        for field, value in (
            ("source_commit", "d" * 40),
            ("artifacts", {"linux-x64": "e" * 64}),
        ):
            with self.subTest(field=field):
                ledger = copy.deepcopy(self.ledger)
                ledger["receipts"]["scan_correlations"][field] = value
                result = evaluate(ledger, self.root)
                self.assertEqual(result["components"]["scans"], 0)
                self.assertEqual(result["status"], "NO-GO")

    def test_skip_and_tampered_evidence_cannot_pass(self):
        self.ledger["receipts"]["live_mcp_scan"]["status"] = "skipped"
        self.assertEqual(evaluate(self.ledger, self.root)["gates"]["G5"], "blocked")
        (self.root / "fixture.txt").write_text("changed")
        self.assertEqual(evaluate(self.ledger, self.root)["score"], 0)

    def test_self_review_and_low_severity_boundary_issue_block(self):
        self.ledger["independent_review"]["reviewer"] = self.ledger["implementer"]
        self.assertEqual(evaluate(self.ledger, self.root)["status"], "NO-GO")
        self.ledger["independent_review"]["reviewer"] = "fixture reviewer"
        self.ledger["findings"] = [
            {"id": "F1", "status": "open", "severity": "low", "airs_boundary": True}
        ]
        self.assertEqual(evaluate(self.ledger, self.root)["status"], "NO-GO")

    def test_path_escape_and_nonfinite_scores_are_rejected(self):
        self.ledger["receipts"]["endpoint_inventory"]["evidence"][0]["path"] = (
            "../outside"
        )
        self.ledger["scores"]["routing"]["points"] = float("nan")
        self.assertEqual(evaluate(self.ledger, self.root)["status"], "NO-GO")


if __name__ == "__main__":
    unittest.main()
