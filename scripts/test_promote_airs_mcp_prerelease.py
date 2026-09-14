"""Promotion rejects evidence from other binaries or incomplete native workflows."""

import copy
import unittest
from promote_airs_mcp_prerelease import TOOLS, validate


class PromotionEvidence(unittest.TestCase):
    def setUp(self):
        self.info = {
            "version": "0.1.0-alpha.13",
            "binary_sha256": "a" * 64,
            "source_commit": "b" * 40,
            "target": "x86_64-unknown-linux-musl",
        }
        cases = [
            "inference_browser_pkce",
            "mcp_browser_pkce",
            "doctor_after_mcp",
            "inference_history_preserved",
            "mcp_logout",
            "mcp_list_after_logout",
            "inference_after_mcp_logout",
            "inference_logout",
        ]
        rows = [{"case": case, "passed": True} for case in cases]
        rows.append(
            {
                "case": "all_read_tools_tool_results",
                "passed": True,
                "tools": sorted(TOOLS),
            }
        )
        rows.extend(
            {
                "case": f"refresh_{c}_{p}_tool_results",
                "passed": True,
                "tools": ["list_workspaces"],
            }
            for c in (1, 2)
            for p in (0, 1)
        )
        self.e2e = {
            "passed": True,
            "binary_sha256": "a" * 64,
            "platform": "Linux",
            "refresh_cycles": 2,
            "results": rows,
        }
        self.upgrade = {
            "passed": True,
            "version": self.info["version"],
            "platform": "Linux",
            "configuration_preserved": True,
            "legacy_target_preserved": True,
            "cases": [
                {
                    "case": "npm-managed-in-place",
                    "passed": True,
                    "binary_sha256": "a" * 64,
                    "uninstall_used": False,
                    "manual_command_removal": False,
                    "force_used": False,
                }
            ],
        }

    def test_accepts_bound_linux_acceptance(self):
        validate(self.info, self.e2e, self.upgrade, None)

    def test_rejects_foreign_binary_or_incomplete_authentication(self):
        for field, value in [
            ("binary_sha256", "c" * 64),
            ("passed", False),
            ("refresh_cycles", 1),
        ]:
            with self.subTest(field=field), self.assertRaises(ValueError):
                altered = {**self.e2e, field: value}
                validate(self.info, altered, self.upgrade, None)

    def test_rejects_failed_tool_result(self):
        altered = copy.deepcopy(self.e2e)
        altered["results"][-1]["passed"] = False
        with self.assertRaises(ValueError):
            validate(self.info, altered, self.upgrade, None)

    def test_rejects_force_for_regular_upgrade(self):
        altered = copy.deepcopy(self.upgrade)
        altered["cases"][0]["force_used"] = True
        with self.assertRaises(ValueError):
            validate(self.info, self.e2e, altered, None)

    def test_mac_requires_exact_verified_signature(self):
        info = {**self.info, "target": "aarch64-apple-darwin"}
        e2e = {**self.e2e, "platform": "Darwin"}
        upgrade = {**self.upgrade, "platform": "Darwin"}
        with self.assertRaises(ValueError):
            validate(info, e2e, upgrade, None)
        signing = {
            "binary_sha256": "a" * 64,
            "source_commit": "b" * 40,
            "team_id": "G5QLZ5A8TA",
            "codesign_verified": True,
            "hardened_runtime": True,
            "notarization_verified": True,
        }
        validate(info, e2e, upgrade, signing)
        with self.assertRaises(ValueError):
            validate(info, e2e, upgrade, {**signing, "notarization_verified": False})
