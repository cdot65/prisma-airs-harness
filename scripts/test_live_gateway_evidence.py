import unittest

from validate_live_gateway import completed_scan


class ScanEvidenceTests(unittest.TestCase):
    def test_allow_label_without_completed_scan_is_inconclusive(self):
        scan = {"phase": "before_request_hooks", "action": "allow", "verdict": True}
        self.assertFalse(completed_scan(scan, "before_request_hooks", "allow"))
        scan.update(scan_id="scan-fixture", profile_id="profile-fixture")
        self.assertTrue(completed_scan(scan, "before_request_hooks", "allow"))
        self.assertFalse(completed_scan(scan, "after_request_hooks", "allow"))
        scan["scan_id"] = " "
        self.assertFalse(completed_scan(scan, "before_request_hooks", "allow"))

    def test_policy_denial_requires_scan_and_false_verdict(self):
        scan = {
            "phase": "before_request_hooks",
            "action": "block",
            "verdict": False,
            "scan_id": "scan-fixture",
            "profile_id": "profile-fixture",
        }
        self.assertTrue(completed_scan(scan, "before_request_hooks", "block"))
        scan["verdict"] = "false"
        self.assertFalse(completed_scan(scan, "before_request_hooks", "block"))


if __name__ == "__main__":
    unittest.main()
