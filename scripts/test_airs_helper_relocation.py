#!/usr/bin/env python3
"""Executable managed-helper relocation checks using disposable, synthetic state.

AIRS_HARNESS_OLD_BIN selects the retained baseline; AIRS_HARNESS_BIN selects the
candidate. MCP stays disabled: this checks actual saved-helper execution and
session pins, not TLS/MCP transport. No native store or live service is touched.
"""

import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tomllib
import unittest

import test_airs_harness as harness


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


@unittest.skipIf(sys.platform == "win32", "POSIX saved shell helper; no Windows credit")
class ManagedHelperRelocation(unittest.TestCase):
    def setUp(self):
        self.fixture = harness.TerminalIntegration()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.root = self.fixture.root
        self.home = self.fixture.home
        self.old_source = Path(os.environ["AIRS_HARNESS_OLD_BIN"]).resolve(strict=True)
        self.new_source = Path(os.environ["AIRS_HARNESS_BIN"]).resolve(strict=True)
        self.old = self.install(self.old_source, "Old install 'quoted'")
        self.new = self.install(self.new_source, "New install 'quoted'")
        self.current = self.old
        self.fixture.run_cli = self.run_cli
        self.checks = {}
        self.env = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith(("AIRS_", "OPENAI_"))
            and key not in ("CODEX_HOME", "CODEX_SQLITE_HOME")
        }
        self.env.update(
            HOME=str(self.root),
            USERPROFILE=str(self.root),
            AIRS_HARNESS_HOME=str(self.home),
            AIRS_TEST_CREDENTIAL="synthetic-inference-relocation-credential",
        )
        self.fixture.env = self.env
        self.fixture.configure()
        # configure() resolves the named environment beneath the application root.
        self.home = self.fixture.home
        self.config = self.home / "config.toml"
        self.revision = self.home / "session-binding.json"
        login = self.run_cli("login", "--credential-env", "AIRS_TEST_CREDENTIAL")
        self.assertEqual(login.returncode, 0, login.stderr)
        self.key = self.root / "synthetic-mcp-key"
        self.key.write_text("synthetic-managed-mcp-relocation-credential")
        self.key.chmod(0o600)
        result = self.run_cli(
            "setup-mcp",
            "--name",
            "scanner",
            "--url",
            "https://mcp.invalid.example/mcp",
            "--credential-file",
            str(self.key),
            "--tool",
            "pan_inline_scan",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.config.write_text(
            self.config.read_text().replace(
                "[mcp_servers.scanner]\n", "[mcp_servers.scanner]\nenabled = false\n"
            )
        )
        self.saved_helper_readback()

    def install(self, source, directory):
        destination = self.root / directory / "airs-harness"
        destination.parent.mkdir()
        try:
            os.link(source, destination)
        except OSError:
            shutil.copy2(source, destination)
        self.assertEqual(digest(source), digest(destination))
        return destination

    def run_cli(self, *args):
        return subprocess.run(
            [str(self.current), *args],
            cwd=self.fixture.work,
            env=self.env,
            capture_output=True,
            text=True,
            timeout=45,
        )

    def saved_helper_readback(self):
        config = tomllib.loads(self.config.read_text())
        command = config["mcp_servers"]["scanner"]["http_headers_helper"]
        arguments = shlex.split(command)
        binding = json.loads(
            next((self.home / "mcp-bindings").glob("*.json")).read_text()
        )
        self.assertEqual(
            arguments,
            [
                str(self.current),
                "mcp-credential",
                "--home",
                str(self.home),
                "--binding",
                binding["id"],
            ],
        )
        # No shell: only the exact fixture-created command is executable here.
        result = subprocess.run(
            arguments,
            cwd=self.fixture.work,
            env=self.env,
            capture_output=True,
            text=True,
            timeout=15,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            json.loads(result.stdout), {"x-portkey-api-key": self.key.read_text()}
        )
        self.assertEqual(self.fixture.mcp_requests, [])

    def seed_session(self):
        result = self.fixture.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            (self.fixture.work / "result.txt").read_text(), "local tool worked\n"
        )
        self.legacy_config = self.config.read_bytes()
        self.legacy_revision = self.revision.read_bytes()
        self.protected = {
            path: path.read_bytes()
            for path in [
                self.home / "credential-binding.json",
                self.home / "auth-generation",
                *list((self.home / "mcp-bindings").glob("*.json")),
            ]
        }
        self.fixture.requests.clear()

    def relocate(self):
        self.old.unlink()
        self.assertFalse(self.old.exists())
        arguments = shlex.split(
            tomllib.loads(self.config.read_text())["mcp_servers"]["scanner"][
                "http_headers_helper"
            ]
        )
        self.assertEqual(arguments[0], str(self.old))
        with self.assertRaises(FileNotFoundError):
            subprocess.run(
                arguments,
                cwd=self.fixture.work,
                env=self.env,
                capture_output=True,
                timeout=15,
                check=False,
            )
        self.checks["old_saved_mcp_helper_unavailable_before_startup"] = True
        self.current = self.new

    def assert_migration(self):
        result = self.fixture.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(self.fixture.requests)
        self.saved_helper_readback()
        config = tomllib.loads(self.config.read_text())
        old = tomllib.loads(self.legacy_config.decode())
        old["model_providers"]["airs"]["auth"]["command"] = str(self.new)
        old["mcp_servers"]["scanner"]["http_headers_helper"] = config["mcp_servers"][
            "scanner"
        ]["http_headers_helper"]
        self.assertEqual(config, old)
        legacy = json.loads(self.legacy_revision)
        current = json.loads(self.revision.read_bytes())
        legacy["mcp_config_revision"] = current["mcp_config_revision"]
        self.assertEqual(legacy, current)
        for path, contents in self.protected.items():
            self.assertEqual(path.read_bytes(), contents)
        self.assertEqual(self.fixture.mcp_requests, [])

    def test_owned_helpers_relocate_with_old_executable_absent(self):
        self.seed_session()
        self.relocate()
        self.assert_migration()
        self.checks["both_helpers_relocated_and_private_readback_passed"] = True

    def test_revision_first_interruption_retries_without_identity_change(self):
        self.seed_session()
        self.relocate()
        self.assert_migration()
        normalized = self.revision.read_bytes()
        # Recreate the durable intermediate state: normalized revision is saved,
        # but the old config has not yet been replaced. This is fault-state
        # replay, not an actual injected process crash or power-loss test.
        self.config.write_bytes(self.legacy_config)
        self.fixture.requests.clear()
        self.assert_migration()
        self.assertEqual(self.revision.read_bytes(), normalized)
        self.checks["revision_first_fault_state_recovered"] = True

    def test_config_first_state_and_policy_tampering_fail_before_send(self):
        self.seed_session()
        self.relocate()
        self.assert_migration()
        migrated = self.config.read_bytes()
        rejected_revision = self.legacy_revision
        revision_label = "config_first"
        if (
            json.loads(rejected_revision)["mcp_config_revision"]
            == json.loads(self.revision.read_bytes())["mcp_config_revision"]
        ):
            # Recent baselines already normalize owned helper paths. Replaying
            # their unchanged pin is valid, so inject a different valid digest
            # to retain the mismatch-before-send check for those baselines.
            revision = json.loads(rejected_revision)
            revision["mcp_config_revision"] = hashlib.sha256(
                b"synthetic-unrelated-mcp-configuration"
            ).hexdigest()
            rejected_revision = json.dumps(revision).encode()
            revision_label = "mismatched_revision"
        for label, altered, pinned in (
            (revision_label, migrated, rejected_revision),
            (
                "tool_policy",
                self.legacy_config.replace(b"pan_inline_scan", b"unexpected_tool"),
                self.legacy_revision,
            ),
            (
                "custom_helper",
                re.sub(
                    rb"(?m)^http_headers_helper = .*?$",
                    b'http_headers_helper = "unowned-custom-helper"',
                    self.legacy_config,
                ),
                self.legacy_revision,
            ),
        ):
            with self.subTest(label=label):
                self.config.write_bytes(altered)
                self.revision.write_bytes(pinned)
                self.fixture.requests.clear()
                result = self.fixture.execute()
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.fixture.requests, [])
                self.assertEqual(self.config.read_bytes(), altered)
                self.assertEqual(self.revision.read_bytes(), pinned)
                self.checks[label + "_rejected_before_send"] = True

    def test_unchanged_custom_helper_keeps_its_raw_session_pin(self):
        custom = "custom-helper --unchanged-user-options"
        self.config.write_text(
            re.sub(
                r"(?m)^http_headers_helper = .*?$",
                "http_headers_helper = " + json.dumps(custom),
                self.config.read_text(),
            )
        )
        self.seed_session()
        self.old.unlink()
        self.current = self.new
        result = self.fixture.execute()
        self.assertEqual(result.returncode, 0, result.stderr)
        config = tomllib.loads(self.config.read_text())
        expected = tomllib.loads(self.legacy_config.decode())
        expected["model_providers"]["airs"]["auth"]["command"] = str(self.new)
        self.assertEqual(config, expected)
        self.assertEqual(self.revision.read_bytes(), self.legacy_revision)
        self.assertEqual(self.fixture.mcp_requests, [])
        self.checks["unowned_helper_preserved_without_normalization"] = True

    def tearDown(self):
        evidence = os.environ.get("AIRS_HELPER_RELOCATION_EVIDENCE")
        if evidence:
            directory = Path(evidence)
            directory.mkdir(parents=True, exist_ok=True)
            record = {
                "old_binary_sha256": digest(self.old_source),
                "candidate_binary_sha256": digest(self.new_source),
                "checks_completed": self.checks,
                "mcp_transport_tested": False,
                "actual_process_crash_tested": False,
                "native_credential_store_tested": False,
                "live_services_used": False,
                "synthetic_saved_mcp_helper_executed": True,
            }
            (directory / (self._testMethodName + ".json")).write_text(
                json.dumps(record, indent=2) + "\n"
            )


if __name__ == "__main__":
    unittest.main()
