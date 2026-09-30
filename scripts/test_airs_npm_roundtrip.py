"""Stable release pins, historical command mapping and rollback contract tests."""

import copy
import subprocess
import sys
import tempfile
from pathlib import Path
import unittest

from airs_npm_versions import released_version
from airs_release_contract import validate_result
from airs_release_execution import invocation
from airs_test_release_spec import LAUNCHER, TARGETS, launcher_migration, validate_spec
from test_airs_test_release_stage import sample_spec
from test_airs_release_acceptance import result_for

SCRIPTS = Path(__file__).resolve().parent


def stable_spec():
    spec = sample_spec()
    spec.update(
        version="0.1.2-alpha.1.mcp.1",
        previous_version="0.1.1",
        previous_release={
            "source_commit": "d" * 40,
            "platforms": [
                {"target": target, "binary_sha256": "e" * 64} for target in TARGETS
            ],
        },
    )
    return spec


def roundtrip_result(spec, target):
    result = result_for("upgrade", spec, target)
    result.update(
        schema_version=2,
        roundtrip={
            "passed": True,
            "previous_version": spec["previous_version"],
            "restored_version": spec["previous_version"],
            "candidate_version": spec["version"],
            "previous_binary_sha256": "e" * 64,
            "restored_binary_sha256": "e" * 64,
            "candidate_binary_sha256": next(
                row["binary_sha256"]
                for row in spec["platforms"]
                if row["target"] == target
            ),
            "candidate_source_commit": spec["source_commit"],
            "previous_source_commit": "d" * 40,
            **{
                key: True
                for key in (
                    "command_links_preserved",
                    "configuration_preserved",
                    "real_conversation_preserved",
                    "inference_credential_reused",
                    "mcp_credential_reused",
                    "native_cleanup_completed",
                )
            },
            "real_mcp_turns": 3,
            "command_links_retargeted": launcher_migration(spec),
            "configuration_rewrites": [],
            "uninstall_used": launcher_migration(spec),
            "force_used": False,
            "production_acceptance": False,
        },
    )
    return result


class VersionSelection(unittest.TestCase):
    def test_historical_command_and_setup_boundaries(self):
        for version, command, setup in (
            ("0.1.0-alpha.12", "airs-harness", ["setup"]),
            ("0.1.0-alpha.20", "airs-harness", ["setup"]),
            ("0.1.2-alpha.1.mcp.1", "airs", ["env", "create", "work"]),
            ("0.1.0-alpha.21", "airs-harness", ["env", "create", "work"]),
            ("0.1.0-alpha.22.onboarding.3", "airs", ["env", "create", "work"]),
            ("0.1.0-alpha.22.mcp.6", "airs", ["env", "create", "work"]),
            ("0.1.1", "airs", ["env", "create", "work"]),
        ):
            with self.subTest(version=version):
                parsed = released_version(version)
                self.assertEqual((parsed.command, parsed.setup), (command, setup))

    def test_launcher_package_name_boundary(self):
        for version, package in (
            ("0.1.0-alpha.12", "airs-harness"),
            ("0.1.0-alpha.22.mcp.6", "airs-harness"),
            ("0.1.3", "airs-harness"),
            ("0.1.3-alpha.7.mcp.1", "airs-harness"),
            ("0.1.4-alpha.1.mcp.1", "airs-harness"),
            ("0.1.4-alpha.2.mcp.1", LAUNCHER),
            ("0.1.4", LAUNCHER),
            ("0.2.0", LAUNCHER),
        ):
            with self.subTest(version=version):
                self.assertEqual(released_version(version).package, package)

    def test_tags_ranges_and_unpinned_stable_fail_before_output_mutation(self):
        with tempfile.TemporaryDirectory() as directory:
            for value in (
                "latest",
                "mcp",
                "^0.1.1",
                "0.1",
                "0.1.1+local",
                "01.1.1",
                "0.1.1",
                "0.1.0-alpha.22.mcp.01",
            ):
                with self.subTest(value=value):
                    output = Path(directory) / "must-not-exist"
                    result = subprocess.run(
                        [
                            sys.executable,
                            str(SCRIPTS / "validate_airs_npm_upgrade.py"),
                            "--previous",
                            value,
                            "--packages",
                            str(Path(directory) / "absent"),
                            "--output",
                            str(output),
                        ],
                        capture_output=True,
                        text=True,
                    )
                    self.assertEqual(result.returncode, 2)
                    self.assertFalse(output.exists())


class StableBaseline(unittest.TestCase):
    def test_baseline_is_canonical_and_bound_to_each_native_invocation(self):
        spec = stable_spec()
        spec["previous_release"]["platforms"].reverse()
        normalized = validate_spec(spec)
        self.assertEqual(
            [row["target"] for row in normalized["previous_release"]["platforms"]],
            list(TARGETS),
        )
        self.assertNotEqual(
            normalized["previous_release"]["platforms"],
            spec["previous_release"]["platforms"],
        )
        for target in TARGETS:
            args, _ = invocation(
                "upgrade",
                normalized,
                target,
                SCRIPTS,
                Path("packages"),
                Path("work"),
                Path("prefix"),
                (Path("airs"), Path("native"), Path("launcher")),
            )
            self.assertEqual(args[args.index("--previous-native-sha256") + 1], "e" * 64)
            self.assertEqual(args[args.index("--previous-source-commit") + 1], "d" * 40)
            self.assertEqual(args[args.index("--previous-package") + 1], "airs-harness")

    def test_missing_malformed_duplicate_or_extra_baselines_are_rejected(self):
        original = stable_spec()
        variants = []
        value = copy.deepcopy(original)
        del value["previous_release"]
        variants.append(value)
        value = copy.deepcopy(original)
        value["previous_release"]["source_commit"] = "main"
        variants.append(value)
        value = copy.deepcopy(original)
        value["previous_release"]["platforms"][1] = value["previous_release"][
            "platforms"
        ][0]
        variants.append(value)
        value = copy.deepcopy(original)
        value["previous_release"]["platforms"][0]["binary_sha256"] = "latest"
        variants.append(value)
        value = sample_spec()
        value["previous_release"] = original["previous_release"]
        variants.append(value)
        for spec in variants:
            with self.subTest(spec=spec), self.assertRaises(ValueError):
                validate_spec(spec)

    def test_stable_gate_requires_actual_identity_bound_preservation(self):
        spec = validate_spec(stable_spec())
        target = next(iter(TARGETS))
        receipt = roundtrip_result(spec, target)
        validate_result("upgrade", receipt, spec, target)
        with self.assertRaises(ValueError):
            validate_result(
                "upgrade", result_for("upgrade", spec, target), spec, target
            )
        for key, current in receipt["roundtrip"].items():
            value = copy.deepcopy(receipt)
            value["roundtrip"][key] = (
                (not current) if isinstance(current, bool) else "tampered"
            )
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_result("upgrade", value, spec, target)
        for key in ("force_used", "uninstall_used", "production_acceptance"):
            value = copy.deepcopy(receipt)
            value["roundtrip"][key] = 0
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_result("upgrade", value, spec, target)

    def test_renamed_launcher_requires_by_name_replacement_without_force(self):
        spec = validate_spec(stable_spec())
        self.assertTrue(launcher_migration(spec))
        target = next(iter(TARGETS))
        receipt = roundtrip_result(spec, target)
        self.assertTrue(receipt["roundtrip"]["uninstall_used"])
        validate_result("upgrade", receipt, spec, target)
        for key in ("uninstall_used", "command_links_retargeted"):
            value = copy.deepcopy(receipt)
            value["roundtrip"][key] = False
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_result("upgrade", value, spec, target)
        value = copy.deepcopy(receipt)
        value["roundtrip"]["force_used"] = True
        with self.assertRaises(ValueError):
            validate_result("upgrade", value, spec, target)
        for key, wrong in (
            ("previous_package", LAUNCHER),
            ("package", "airs-harness"),
            ("launcher_migration", False),
        ):
            value = copy.deepcopy(receipt)
            value[key] = wrong
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_result("upgrade", value, spec, target)

    def test_configuration_rewrites_are_bounded_to_the_launcher_directories(self):
        spec = validate_spec(stable_spec())
        target = next(iter(TARGETS))
        receipt = roundtrip_result(spec, target)
        rewrite = {
            "phase": "candidate-resumed-turn",
            "file": "config.toml",
            "key": "mcp_servers.airs.command",
            "from_package": "airs-harness",
            "to_package": LAUNCHER,
        }
        receipt["roundtrip"]["configuration_rewrites"] = [
            rewrite,
            {
                **rewrite,
                "phase": "previous-resumed-turn",
                "from_package": LAUNCHER,
                "to_package": "airs-harness",
            },
        ]
        validate_result("upgrade", receipt, spec, target)
        for bad in (
            {**rewrite, "file": "environments.json"},
            {**rewrite, "to_package": "airs-harness"},
            {**rewrite, "from_package": "something-else"},
            {**rewrite, "key": None},
        ):
            value = copy.deepcopy(receipt)
            value["roundtrip"]["configuration_rewrites"] = [bad]
            with self.subTest(bad=bad), self.assertRaises(ValueError):
                validate_result("upgrade", value, spec, target)
        same = stable_spec()
        same.update(version="0.1.5-alpha.1.mcp.1", previous_version="0.1.4")
        same = validate_spec(same)
        value = roundtrip_result(same, target)
        value["roundtrip"]["configuration_rewrites"] = [
            {**rewrite, "from_package": LAUNCHER, "to_package": LAUNCHER}
        ]
        with self.assertRaises(ValueError):
            validate_result("upgrade", value, same, target)

    def test_same_name_upgrade_must_not_uninstall(self):
        spec = stable_spec()
        spec.update(version="0.1.5-alpha.1.mcp.1", previous_version="0.1.4")
        spec = validate_spec(spec)
        self.assertFalse(launcher_migration(spec))
        target = next(iter(TARGETS))
        receipt = roundtrip_result(spec, target)
        self.assertFalse(receipt["roundtrip"]["uninstall_used"])
        validate_result("upgrade", receipt, spec, target)
        for key in ("uninstall_used", "command_links_retargeted"):
            value = copy.deepcopy(receipt)
            value["roundtrip"][key] = True
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate_result("upgrade", value, spec, target)

    def test_alpha_migration_evidence_does_not_require_modern_fixture(self):
        spec = validate_spec(sample_spec())
        validate_result(
            "upgrade",
            result_for("upgrade", spec, next(iter(TARGETS))),
            spec,
            next(iter(TARGETS)),
        )


if __name__ == "__main__":
    unittest.main()
