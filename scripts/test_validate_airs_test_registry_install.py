"""Installed test-release bytes and native-host boundaries."""

import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch

import validate_airs_test_registry_install as validator


class InstalledBytes(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.package = self.root / "package"
        (self.package / "bin").mkdir(parents=True)
        self.binary = self.package / "bin/airs"
        self.binary.write_bytes(b"synthetic shipped executable\n")
        self.binary.chmod(0o755)
        self.inventory = {"members": {
            "package/bin": {"type": "dir"},
            "package/bin/airs": {
                "type": "file", "mode": 0o755,
                "sha256": hashlib.sha256(self.binary.read_bytes()).hexdigest(),
            },
        }}

    def test_exact_bytes_pass_but_corruption_and_missing_executable_mode_fail(self):
        validator.verify_installed_files(self.package, self.inventory)
        self.binary.chmod(0o644)
        with self.assertRaisesRegex(ValueError, "executable mode"):
            validator.verify_installed_files(self.package, self.inventory)
        self.binary.chmod(0o755)
        self.binary.write_bytes(b"changed executable")
        with self.assertRaisesRegex(ValueError, "staged bytes"):
            validator.verify_installed_files(self.package, self.inventory)

    def test_file_or_parent_symlink_cannot_substitute_installed_bytes(self):
        saved = self.root / "saved"
        self.binary.rename(saved)
        self.binary.symlink_to(saved)
        with self.assertRaisesRegex(ValueError, "Linked"):
            validator.verify_installed_files(self.package, self.inventory)
        self.binary.unlink()
        saved.rename(self.binary)
        linked = self.root / "linked"
        linked.symlink_to(self.package, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "Linked"):
            validator.verify_installed_files(linked, self.inventory)

    def test_missing_file_and_traversal_inventory_are_rejected(self):
        self.binary.unlink()
        with self.assertRaisesRegex(ValueError, "missing"):
            validator.verify_installed_files(self.package, self.inventory)
        self.inventory["members"]["package/../../escape"] = self.inventory["members"].pop("package/bin/airs")
        with self.assertRaises(ValueError):
            validator.verify_installed_files(self.package, self.inventory)

    def test_linked_output_is_rejected_before_any_installation(self):
        linked = self.root / "linked"
        linked.symlink_to(self.package, target_is_directory=True)
        spec = {"platforms": [{"target": "aarch64-apple-darwin", "binary_sha256": "a" * 64}]}
        with patch.object(validator, "host_target", return_value="aarch64-apple-darwin"):
            with self.assertRaisesRegex(ValueError, "Linked"):
                validator.install(spec, self.root / "archives", linked / "new-prefix")
        self.assertFalse((self.package / "new-prefix").exists())

    def test_host_architecture_requires_a_supported_native_platform(self):
        with patch.object(validator.platform, "system", return_value="Darwin"):
            with patch.object(validator.platform, "machine", return_value="x86_64"):
                with self.assertRaisesRegex(ValueError, "supported native host"):
                    validator.host_target()
            with patch.object(validator.platform, "machine", return_value="arm64"):
                self.assertEqual(validator.host_target(), "aarch64-apple-darwin")

    def test_exact_version_is_insufficient_when_advertised_channel_differs(self):
        spec = {"version": "0.1.0-alpha.22.mcp.3", "tag": "mcp", "registry": "https://npm.example.test"}
        row = {"name": "airs-harness", "integrity": "sha512-synthetic"}
        published = {"name": row["name"], "version": spec["version"], "dist": {
            "integrity": row["integrity"], "tarball": "https://npm.example.test/airs-harness/-/release.tgz",
        }}
        document = {"versions": {spec["version"]: published}, "dist-tags": {"mcp": spec["version"]}}
        registry = Mock()
        registry.metadata.return_value = document
        validator.verify_registry_metadata(spec, [row], registry)
        document["dist-tags"]["mcp"] = "0.1.0-alpha.22.mcp.2"
        with self.assertRaisesRegex(ValueError, "mcp channel"):
            validator.verify_registry_metadata(spec, [row], registry)
        document["dist-tags"]["mcp"] = spec["version"]
        published["dist"]["tarball"] = "https://unrelated.example.test/release.tgz"
        with self.assertRaisesRegex(ValueError, "selected HTTPS registry"):
            validator.verify_registry_metadata(spec, [row], registry)


if __name__ == "__main__":
    unittest.main()
