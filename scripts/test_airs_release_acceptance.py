"""Adversarial checks for portable acceptance and resumable stage evidence."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from types import SimpleNamespace

import airs_release_acceptance as acceptance
from airs_release_receipts import (
    atomic_json,
    evidence_path,
    snapshot_tooling,
    verify_stage,
    verify_tooling,
    write_stage,
)
from airs_test_release_spec import (
    TARGETS,
    canonical_digest,
    digest_file,
    load_json,
    validate_spec,
)
from test_airs_test_release_stage import candidates, sample_spec

SCRIPTS = Path(__file__).resolve().parent


def result_for(name, spec, target):
    digest = next(
        row["binary_sha256"] for row in spec["platforms"] if row["target"] == target
    )
    result = {"passed": True, "checks": ["synthetic behavior exercised"]}
    result.update(
        {
            key: digest
            for key in ("binary_sha256", "native_binary_sha256", "native_sha256")
        }
    )
    if name == "install":
        result.update(
            source_commit=spec["source_commit"],
            version="airs " + spec["version"],
            native_package=TARGETS[target],
        )
    elif name == "onboarding":
        result.update(
            local_https_oidc=True,
            native_os_store=True,
            production_sso=False,
            production_servicenow=False,
        )
    elif name in acceptance.PATTERNS:
        result.update(
            schema_version=1,
            pattern=acceptance.PATTERNS[name],
            test_ids=["fixture.case.test_behavior"],
            tests_run=1,
            failures=[],
            errors=[],
            skipped=[],
        )
    elif name == "upgrade":
        result.update(
            previous=spec["previous_version"],
            previous_registry=spec["registry"],
            version=spec["version"],
            configuration_preserved=True,
            legacy_target_preserved=True,
        )
    elif name == "managed-cli":
        result.update(live_api_operations=False)
    elif name == "mac-signature":
        result.update(
            developer_id_team=spec["developer_id_team"],
            notarization_verified=True,
            installed_bytes=True,
        )
    return result


def evidence_set(
    root, packages, spec, installation="candidate", verification_tooling_commit=None
):
    for target in TARGETS:
        output = root / target
        output.mkdir(parents=True)
        tooling = {
            "schema_version": 1,
            "tooling_commit": verification_tooling_commit or spec["tooling_commit"],
            "files": {"scripts/example.py": "a" * 64},
        }
        identity = {
            "installation": installation,
            "spec_sha256": canonical_digest(spec),
            **{
                key: spec[key]
                for key in (
                    "source_commit",
                    "tooling_commit",
                    "packaging_commit",
                    "version",
                )
            },
            "target": target,
            "observed_target": target,
            "binary_sha256": next(
                row["binary_sha256"]
                for row in spec["platforms"]
                if row["target"] == target
            ),
            **acceptance.candidate_identity(spec, packages),
            "tooling_sha256": canonical_digest(tooling),
        }
        if verification_tooling_commit is not None:
            identity["verification_tooling_commit"] = verification_tooling_commit
        shutil.copyfile(
            packages / "NPM-PACKAGES.json", output / "CANDIDATE-NPM-PACKAGES.json"
        )
        atomic_json(output / "TOOLING.json", tooling)
        hashes = {}
        for name in acceptance.stages(target):
            paths = acceptance.paths_for(name, target)
            (output / paths[0]).parent.mkdir(exist_ok=True)
            (output / paths[0]).write_text("controlled fixture output\n")
            atomic_json(output / paths[1], result_for(name, spec, target))
            for relative in paths[2:]:
                atomic_json(output / relative, {"passed": True, "failures": []})
            write_stage(
                output,
                name,
                {**identity, "contract": acceptance.contract(name, target)},
                dict(hashes),
                ["python", "fixture.py", name],
                paths,
            )
            hashes[name] = digest_file(output / "receipts" / f"{name}.json")
        atomic_json(
            output / "ACCEPTANCE.json",
            {
                "schema_version": 1,
                "passed": True,
                "identity": identity,
                "stages": acceptance.stages(target),
                "stage_receipts": hashes,
                "production_sso": False,
                "production_servicenow": False,
            },
        )


class AcceptanceEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.spec = validate_spec(sample_spec())
        self.packages = self.root / "packages"
        candidates(self.packages, self.spec)
        self.evidence = self.root / "evidence"
        evidence_set(self.evidence, self.packages, self.spec)
        self.target = next(iter(TARGETS))
        self.output = self.evidence / self.target

    def test_portable_receipts_bind_all_native_targets_and_exact_archives(self):
        shutil.rmtree(self.packages)
        result = acceptance.verify_acceptance_set(self.spec, self.evidence)
        self.assertEqual(list(TARGETS), [row["target"] for row in result["platforms"]])
        self.assertEqual(self.spec["source_commit"], result["source_commit"])
        self.assertEqual(64, len(result["candidate_packages_sha256"]))

    def test_candidate_and_registry_acceptances_cannot_substitute(self):
        with self.assertRaisesRegex(ValueError, "installation mode"):
            acceptance.verify_acceptance_set(
                self.spec, self.evidence, installation="registry"
            )
        other = self.root / "registry"
        evidence_set(other, self.packages, self.spec, installation="registry")
        acceptance.verify_acceptance_set(self.spec, other, installation="registry")
        with self.assertRaises(ValueError):
            acceptance.verify_acceptance_set(self.spec, other)

    def test_missing_native_target_and_missing_stage_never_pass(self):
        receipt = self.output / "receipts/doctor.json"
        receipt.unlink()
        with self.assertRaises(ValueError):
            acceptance.verify_acceptance_set(self.spec, self.evidence)
        shutil.rmtree(self.output)
        with self.assertRaisesRegex(ValueError, "three native"):
            acceptance.verify_acceptance_set(self.spec, self.evidence)

    def test_resume_stage_checks_actual_outputs_inputs_dependencies_and_command(self):
        path = self.output / "receipts/doctor.json"
        original = load_json(path)
        for field, value in [
            ("inputs", {}),
            ("input_sha256", "0" * 64),
            ("dependencies", {}),
            ("command", ["echo", "passed"]),
            ("exit_code", 1),
            ("schema_version", 2),
        ]:
            with self.subTest(field=field):
                atomic_json(path, {**original, field: value})
                with self.assertRaises(ValueError):
                    verify_stage(
                        self.output,
                        "doctor",
                        original["inputs"],
                        original["dependencies"],
                    )
        atomic_json(path, original)
        (self.output / "logs/doctor.log").write_text("changed")
        with self.assertRaisesRegex(ValueError, "output evidence"):
            verify_stage(
                self.output, "doctor", original["inputs"], original["dependencies"]
            )

    def test_top_level_identity_and_stage_set_changes_fail_closed(self):
        path = self.output / "ACCEPTANCE.json"
        original = load_json(path)
        cases = [
            dict(original, stages=original["stages"][:-1]),
            dict(original, schema_version=2),
            dict(original, production_sso=True),
        ]
        for field, value in [
            ("source_commit", "f" * 40),
            ("tooling_commit", "f" * 40),
            ("observed_target", "aarch64-apple-darwin"),
            ("binary_sha256", "f" * 64),
            ("candidate_packages_sha256", "f" * 64),
        ]:
            cases.append(
                dict(original, identity={**original["identity"], field: value})
            )
        for value in cases:
            with self.subTest(value=value):
                atomic_json(path, value)
                with self.assertRaises(ValueError):
                    acceptance.verify_acceptance_set(self.spec, self.evidence)
        atomic_json(path, original)

    def test_rehashed_zero_or_all_skipped_suite_still_rejected(self):
        for count, ids, skipped in [
            (0, [], []),
            (1, ["fixture.test"], [{"test": "fixture.test", "reason": "unavailable"}]),
        ]:
            with self.subTest(count=count):
                value = result_for("doctor", self.spec, self.target)
                value.update(tests_run=count, test_ids=ids, skipped=skipped)
                with self.assertRaisesRegex(ValueError, "Empty or entirely skipped"):
                    acceptance.validate_result("doctor", value, self.spec, self.target)

    def test_symlinked_evidence_and_parent_destinations_are_rejected(self):
        path = self.output / "logs/doctor.log"
        path.unlink()
        path.symlink_to(self.output / "TOOLING.json")
        with self.assertRaisesRegex(ValueError, "Linked"):
            acceptance.verify_acceptance_set(self.spec, self.evidence)
        linked = self.root / "linked"
        linked.symlink_to(self.output, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "Linked"):
            atomic_json(linked / "injected.json", {})
        self.assertFalse((self.output / "injected.json").exists())
        for relative in ("../outside", "/absolute", "logs/../../outside"):
            with self.assertRaises((ValueError, RuntimeError)):
                evidence_path(self.output, relative)

    def test_output_hash_verification_survives_optimized_python(self):
        spec_path = self.root / "spec.json"
        atomic_json(spec_path, self.spec)
        (self.output / "logs/doctor.log").write_text("tampered")
        code = "from airs_release_acceptance import verify_acceptance_set; from airs_test_release_spec import load_spec; import sys; verify_acceptance_set(load_spec(sys.argv[1]),sys.argv[2])"
        result = subprocess.run(
            [sys.executable, "-O", "-c", code, str(spec_path), str(self.evidence)],
            cwd=SCRIPTS,
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(0, result.returncode)
        self.assertIn("Stage output evidence changed", result.stderr)

    def test_upgrade_consumes_archive_only_inputs_and_rejects_changed_integrity(self):
        from validate_airs_npm_upgrade import archive_manifest

        record = load_json(self.packages / "NPM-PACKAGES.json")["publish_order"][-1]
        self.assertFalse((self.packages / "airs-harness").exists())
        manifest, archive = archive_manifest(self.packages, record)
        self.assertEqual("airs-harness", manifest["name"])
        self.assertEqual(self.spec["version"], manifest["version"])
        with self.assertRaisesRegex(ValueError, "integrity mismatch"):
            archive_manifest(self.packages, {**record, "sha256": "f" * 64})
        with self.assertRaisesRegex(ValueError, "identity mismatch"):
            archive_manifest(
                self.packages, {**record, "version": "0.1.0-alpha.22.mcp.9"}
            )

    def test_changed_candidate_archive_is_not_resumable(self):
        archive = next((self.packages / "tarballs").glob("*.tgz"))
        archive.write_bytes(b"changed archive")
        with self.assertRaisesRegex(ValueError, "archive integrity"):
            acceptance.candidate_identity(self.spec, self.packages)


class AcceptanceRunnerTests(unittest.TestCase):
    setUp = AcceptanceEvidenceTests.setUp

    # Exercise real subprocess output and an intentionally interrupted stage.
    def test_interrupted_run_resumes_only_rehashed_completed_stages(self):
        self.exercise_interrupted_run("candidate", None)

    def test_registry_revision_is_bound_through_run_and_resume(self):
        self.exercise_interrupted_run("registry", "f" * 40)

    def exercise_interrupted_run(self, installation, revision):
        scripts = self.root / "tooling/scripts"
        scripts.mkdir(parents=True)
        filenames = (
            "airs_release_acceptance.py",
            "airs_release_contract.py",
            "airs_release_execution.py",
            "airs_release_receipts.py",
            "airs_release_unittest.py",
            "validate_airs_npm.py",
            "validate_airs_command_output.py",
            "validate_airs_npm_upgrade.py",
            "validate_airs_test_registry_install.py",
        )
        tooling = {
            "schema_version": 1,
            "tooling_commit": revision or self.spec["tooling_commit"],
            "files": {f"scripts/{name}": "a" * 64 for name in filenames},
        }
        atomic_json(scripts.parent / "ACCEPTANCE-TOOLING.json", tooling)
        output = self.root / "execution" / self.target
        calls = []
        interrupted = [True]

        def invoke(
            name, spec, target, scripts, packages, work, prefix, installed, installation
        ):
            calls.append(name)
            if name == "doctor" and interrupted[0]:
                raise RuntimeError("controlled interruption before doctor")
            destination = prefix if name == "install" else work
            values = {acceptance.RESULTS[name]: result_for(name, spec, target)}
            if name == "install":
                values["INSTALL-NETWORK.json"] = {"passed": True}
            code = "import pathlib,sys,json; root=pathlib.Path(sys.argv[1]); root.mkdir(parents=True,exist_ok=True); [(root/name).write_text(json.dumps(value)) for name,value in json.loads(sys.argv[2]).items()]; print('controlled stage complete')"
            return [
                sys.executable,
                "-c",
                code,
                str(destination),
                json.dumps(values),
            ], destination

        installed = (self.root / "airs", self.root / "native", self.root / "launcher")
        with (
            patch.object(acceptance, "observed_target", return_value=self.target),
            patch.object(
                acceptance, "verify_tooling", return_value=canonical_digest(tooling)
            ),
            patch.object(
                acceptance, "installed_identity", return_value=installed
            ) as actual_identity,
            patch.object(acceptance, "invocation", side_effect=invoke),
        ):
            with self.assertRaisesRegex(RuntimeError, "controlled interruption"):
                acceptance.run_acceptance(
                    self.spec,
                    self.packages,
                    scripts,
                    output,
                    installation=installation,
                    verification_tooling_commit=revision,
                )
            self.assertFalse((output / "ACCEPTANCE.json").exists())
            completed = calls[:-1]
            interrupted[0] = False
            calls.clear()
            acceptance.run_acceptance(
                self.spec,
                self.packages,
                scripts,
                output,
                resume=True,
                installation=installation,
                verification_tooling_commit=revision,
            )
            self.assertEqual(acceptance.stages(self.target)[len(completed) :], calls)
            self.assertGreaterEqual(actual_identity.call_count, 3)
            acceptance.verify_one(
                self.spec, output, self.target, installation, revision
            )
            calls.clear()
            (output / "logs/install.log").write_text("tampered after successful run")
            with self.assertRaisesRegex(ValueError, "output evidence"):
                acceptance.run_acceptance(
                    self.spec,
                    self.packages,
                    scripts,
                    output,
                    resume=True,
                    installation=installation,
                    verification_tooling_commit=revision,
                )
            self.assertEqual([], calls)


class StructuredExecutionTests(unittest.TestCase):
    def test_real_subprocess_results_distinguish_success_failure_zero_and_skipped(self):
        cases = {
            "pass": "self.assertEqual(2 + 2, 4)",
            "fail": 'self.fail("controlled failure")',
            "skip": 'self.skipTest("controlled unavailable fixture")',
            "empty": None,
        }
        for name, body in cases.items():
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                if body:
                    (root / "test_fixture.py").write_text(
                        "import unittest\nclass Fixture(unittest.TestCase):\n    def test_behavior(self):\n        "
                        + body
                        + "\n"
                    )
                receipt = root / "result.json"
                result = subprocess.run(
                    [
                        sys.executable,
                        "-O",
                        str(SCRIPTS / "airs_release_unittest.py"),
                        "--scripts",
                        str(root),
                        "--pattern",
                        "test_fixture.py",
                        "--receipt",
                        str(receipt),
                    ],
                    capture_output=True,
                    text=True,
                )
                summary = load_json(receipt)
                self.assertEqual(name == "pass", result.returncode == 0, result.stderr)
                self.assertEqual(name == "pass", summary["passed"])
                self.assertEqual(0 if name == "empty" else 1, summary["tests_run"])
                self.assertEqual(1 if name == "skip" else 0, len(summary["skipped"]))

    def test_child_environment_drops_credentials_and_disables_optimization(self):
        with patch.dict(
            os.environ,
            {
                "PYTHONOPTIMIZE": "2",
                "NODE_OPTIONS": "--eval bad",
                "NPM_TOKEN": "synthetic",
                "OPENAI_API_KEY": "synthetic",
                "AIRS_API_KEY": "synthetic",
            },
        ):
            environment = acceptance.clean_environment()
        for key in ("NODE_OPTIONS", "NPM_TOKEN", "OPENAI_API_KEY", "AIRS_API_KEY"):
            self.assertNotIn(key, environment)
        result = subprocess.check_output(
            [sys.executable, "-c", "import sys; print(sys.flags.optimize)"],
            env=environment,
            text=True,
        )
        self.assertEqual("0", result.strip())

    def test_installed_identity_rehashes_native_launcher_and_managed_bundle(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            spec = validate_spec(sample_spec())
            target = next(iter(TARGETS))
            prefix = root / "prefix"
            launcher = prefix / "lib/node_modules/airs-harness"
            native_package = prefix / "lib/node_modules" / TARGETS[target]
            (native_package / "bin").mkdir(parents=True)
            native = native_package / "bin/airs-harness"
            native.write_bytes(target.encode())
            for directory in ("bin", "lib", "managed-cli"):
                (launcher / directory).mkdir(parents=True)
            entry = launcher / "bin/airs.js"
            entry.write_text("fixture launcher")
            module = launcher / "lib/launcher.js"
            module.write_text("fixture implementation")
            atomic_json(
                launcher / "package.json",
                {
                    "name": "airs-harness",
                    "version": spec["version"],
                    "optionalDependencies": {
                        name: spec["version"] for name in TARGETS.values()
                    },
                },
            )
            atomic_json(
                native_package / "package.json",
                {"name": TARGETS[target], "version": spec["version"]},
            )
            atomic_json(
                native_package / "BUILD-INFO.json",
                {
                    "source_commit": spec["source_commit"],
                    "version": spec["version"],
                    "target": target,
                    "binary_sha256": digest_file(native),
                },
            )
            (prefix / "bin").mkdir()
            (prefix / "bin/airs").symlink_to(entry)
            tooling = {
                "packaging_commit": spec["packaging_commit"],
                "files": {
                    "npm/airs-harness/"
                    + path.relative_to(launcher).as_posix(): digest_file(path)
                    for path in (entry, module)
                },
            }
            atomic_json(launcher / "PACKAGE-TOOLING.json", tooling)
            atomic_json(launcher / "BUNDLE-INVENTORY.json", {"fixture": True})
            packages = root / "packages"
            atomic_json(
                packages / "NPM-PACKAGES.json",
                {
                    "package_tooling": tooling,
                    "cli_bundle": {
                        "inventory_sha256": digest_file(
                            launcher / "BUNDLE-INVENTORY.json"
                        )
                    },
                },
            )

            def output(command, **kwargs):
                return (
                    str(native_package / "package.json")
                    if command[0] == "node"
                    else "airs " + spec["version"]
                )

            with (
                patch.object(acceptance.subprocess, "check_output", side_effect=output),
                patch("airs_bundle.verify_bundle") as bundle,
            ):
                acceptance.installed_identity(spec, packages, prefix, target, {})
                bundle.assert_called_once()
                module.write_text("mutated launcher implementation")
                with self.assertRaisesRegex(ValueError, "launcher tooling bytes"):
                    acceptance.installed_identity(spec, packages, prefix, target, {})
                module.write_text("fixture implementation")
                native.write_bytes(b"mutated native")
                with self.assertRaisesRegex(ValueError, "native bytes changed"):
                    acceptance.installed_identity(spec, packages, prefix, target, {})

    def test_tooling_transfer_binds_nested_fixture_files_and_detects_mutation(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            committed = {
                "scripts/check.py": b"# controlled fixture validator\n",
                "scripts/fixtures/npm-command-shims/node.json": b'{"variants": []}',
                "scripts/fixtures/npm-command-shims/sources.json": b"{}",
                "scripts/fixtures/npm-command-shims/README.md": b"Controlled fixture provenance",
                "codex-rs/airs-identity/src/fixtures/test-only-private.pem": b"synthetic test-only key fixture",
            }
            for relative, content in committed.items():
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(content)

            def git_show(command, **kwargs):
                self.assertEqual(["git", "show"], command[:2])
                return SimpleNamespace(stdout=committed[command[2].split(":", 1)[1]])

            with patch("airs_release_receipts.subprocess.run", side_effect=git_show):
                manifest = snapshot_tooling(root / "scripts", "b" * 40)
                self.assertEqual(set(committed), set(manifest["files"]))
                self.assertEqual(
                    canonical_digest(manifest), verify_tooling(root, manifest, "b" * 40)
                )
                wrapper = root / "scripts/fixtures/npm-command-shims/node.json"
                wrapper.write_bytes(b"changed wrapper reference")
                with self.assertRaisesRegex(ValueError, "tooling bytes changed"):
                    verify_tooling(root, manifest, "b" * 40)
                with self.assertRaisesRegex(ValueError, "differs from commit"):
                    snapshot_tooling(root / "scripts", "b" * 40)
                wrapper.unlink()
                with self.assertRaisesRegex(ValueError, "inventory changed"):
                    verify_tooling(root, manifest, "b" * 40)
                wrapper.write_bytes(
                    committed["scripts/fixtures/npm-command-shims/node.json"]
                )
                (wrapper.parent / "unexpected.json").write_bytes(b"{}")
                with self.assertRaisesRegex(ValueError, "inventory changed"):
                    verify_tooling(root, manifest, "b" * 40)

    def test_registry_install_and_upgrade_receive_explicit_registry(self):
        spec = validate_spec(sample_spec())
        spec["registry"] = "https://another-registry.example.com"
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            command, result = acceptance.invocation(
                "install",
                spec,
                next(iter(TARGETS)),
                SCRIPTS,
                root / "packages",
                root,
                root / "prefix",
                None,
                "registry",
            )
            self.assertEqual(
                "validate_airs_test_registry_install.py", Path(command[1]).name
            )
            self.assertEqual(spec, load_json(root / "SPEC.json"))
            command, result = acceptance.invocation(
                "upgrade",
                spec,
                next(iter(TARGETS)),
                SCRIPTS,
                root / "packages",
                root,
                root / "prefix",
                (root / "airs", root / "native", root / "launcher"),
            )
            self.assertEqual(spec["registry"], command[command.index("--registry") + 1])


if __name__ == "__main__":
    unittest.main()
