"""Installed test-release bytes and native-host boundaries."""

import hashlib
import io
import json
import os
import stat
import subprocess
import tarfile
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
        self.inventory = {
            "members": {
                "package/bin": {"type": "dir"},
                "package/bin/airs": {
                    "type": "file",
                    "mode": 0o755,
                    "sha256": hashlib.sha256(self.binary.read_bytes()).hexdigest(),
                },
            }
        }

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
        self.inventory["members"]["package/../../escape"] = self.inventory[
            "members"
        ].pop("package/bin/airs")
        with self.assertRaises(ValueError):
            validator.verify_installed_files(self.package, self.inventory)

    @unittest.skipIf(
        os.name == "nt", "Native test packages target POSIX executable modes"
    )
    def test_real_npm_normalizes_only_declared_root_and_nested_bundled_bins(self):
        from airs_npm_registry import install_environment
        from airs_test_release_archive import inspect_archive

        root_manifest = {
            "name": "airs-mode-fixture",
            "version": "1.0.0",
            "bin": {"airs-mode-fixture": "bin/root.js"},
            "dependencies": {"bundled-mode-fixture": "1.0.0"},
            "bundleDependencies": ["bundled-mode-fixture"],
        }
        dependency = {
            "name": "bundled-mode-fixture",
            "version": "1.0.0",
            "bin": "./bin/cli.js",
            "dependencies": {"@fixture/nested-mode": "1.0.0"},
            "bundleDependencies": ["@fixture/nested-mode"],
        }
        nested = {
            "name": "@fixture/nested-mode",
            "version": "1.0.0",
            "bin": {"nested-mode": "bin/cli.js"},
        }
        dep_root = "node_modules/bundled-mode-fixture"
        nested_root = dep_root + "/node_modules/@fixture/nested-mode"
        binaries = [
            "bin/root.js",
            dep_root + "/bin/cli.js",
            nested_root + "/bin/cli.js",
        ]
        contents = {
            "package.json": json.dumps(root_manifest).encode(),
            dep_root + "/package.json": json.dumps(dependency).encode(),
            nested_root + "/package.json": json.dumps(nested).encode(),
            "README.md": b"Unrelated non-executable documentation\n",
        }
        contents.update(
            {
                name: b"#!/usr/bin/env node\nconsole.log('controlled bin fixture');\n"
                for name in binaries
            }
        )
        archive = self.root / "fixture.tgz"
        with tarfile.open(archive, "w:gz", format=tarfile.USTAR_FORMAT) as writer:
            for name, payload in contents.items():
                member = tarfile.TarInfo("package/" + name)
                member.size, member.mode = len(payload), 0o644
                writer.addfile(member, io.BytesIO(payload))
        inventory = inspect_archive(archive)
        prefix = self.root / "npm-prefix"
        prefix.mkdir()
        environment = install_environment(prefix, "https://registry.invalid", False)
        result = subprocess.run(
            [
                "npm",
                "install",
                "--global",
                "--prefix",
                str(prefix),
                "--ignore-scripts",
                "--offline",
                "--no-audit",
                "--no-fund",
                str(archive),
            ],
            env=environment,
            cwd=prefix,
            capture_output=True,
            text=True,
            timeout=60,
        )
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        installed = prefix / "lib/node_modules/airs-mode-fixture"
        for name in binaries:
            with self.subTest(bin=name):
                self.assertEqual(
                    0, inventory["members"]["package/" + name]["mode"] & 0o111
                )
                self.assertEqual(
                    0o111, stat.S_IMODE((installed / name).stat().st_mode) & 0o111
                )
        validator.verify_installed_files(installed, inventory)
        (installed / binaries[0]).chmod(0o644)
        with self.assertRaisesRegex(ValueError, "executable mode"):
            validator.verify_installed_files(installed, inventory)
        (installed / binaries[0]).chmod(0o755)
        (installed / "README.md").chmod(0o755)
        with self.assertRaisesRegex(ValueError, "executable mode"):
            validator.verify_installed_files(installed, inventory)
        (installed / "README.md").chmod(0o644)
        (installed / binaries[-1]).write_bytes(b"changed declared bin")
        with self.assertRaisesRegex(ValueError, "staged bytes"):
            validator.verify_installed_files(installed, inventory)

    def test_bin_permission_requires_verified_manifest_and_safe_existing_target(self):
        manifest = self.package / "package.json"
        manifest.write_text(json.dumps({"name": "controlled", "bin": {}}))
        original = manifest.read_bytes()
        self.inventory["members"]["package/package.json"] = {
            "type": "file",
            "mode": 0o644,
            "sha256": hashlib.sha256(original).hexdigest(),
        }
        self.binary.chmod(0o644)
        self.inventory["members"]["package/bin/airs"]["mode"] = 0o644
        validator.verify_installed_files(self.package, self.inventory)
        manifest.write_text(
            json.dumps({"name": "controlled", "bin": {"controlled": "bin/airs"}})
        )
        self.binary.chmod(0o755)
        with self.assertRaisesRegex(ValueError, "staged bytes"):
            validator.verify_installed_files(self.package, self.inventory)
        for target in ("../outside", "/absolute", "bin/missing"):
            with self.subTest(target=target):
                manifest.write_text(
                    json.dumps({"name": "controlled", "bin": {"controlled": target}})
                )
                self.inventory["members"]["package/package.json"]["sha256"] = (
                    hashlib.sha256(manifest.read_bytes()).hexdigest()
                )
                with self.assertRaises(ValueError):
                    validator.verify_installed_files(self.package, self.inventory)

    def test_non_package_asset_manifest_cannot_authorize_executable_mode(self):
        fixture = self.package / "fixtures"
        fixture.mkdir()
        contents = {
            "package.json": json.dumps(
                {"name": "example-only", "bin": {"example": "asset.js"}}
            ).encode(),
            "asset.js": b"fixture asset",
        }
        for name, data in contents.items():
            path = fixture / name
            path.write_bytes(data)
            self.inventory["members"]["package/fixtures/" + name] = {
                "type": "file",
                "mode": 0o644,
                "sha256": hashlib.sha256(data).hexdigest(),
            }
        (fixture / "asset.js").chmod(0o755)
        with self.assertRaisesRegex(ValueError, "executable mode"):
            validator.verify_installed_files(self.package, self.inventory)

    def test_linked_output_is_rejected_before_any_installation(self):
        linked = self.root / "linked"
        linked.symlink_to(self.package, target_is_directory=True)
        spec = {
            "platforms": [{"target": "aarch64-apple-darwin", "binary_sha256": "a" * 64}]
        }
        with patch.object(
            validator, "host_target", return_value="aarch64-apple-darwin"
        ):
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
        spec = {
            "version": "0.1.0-alpha.22.mcp.3",
            "tag": "mcp",
            "registry": "https://npm.example.test",
        }
        row = {"name": "airs-harness", "integrity": "sha512-synthetic"}
        published = {
            "name": row["name"],
            "version": spec["version"],
            "dist": {
                "integrity": row["integrity"],
                "tarball": "https://npm.example.test/airs-harness/-/release.tgz",
            },
        }
        document = {
            "versions": {spec["version"]: published},
            "dist-tags": {"mcp": spec["version"]},
        }
        registry = Mock()
        registry.metadata.return_value = document
        validator.verify_registry_metadata(spec, [row], registry)
        document["dist-tags"]["latest"] = "older-default"
        with self.assertRaisesRegex(ValueError, "Default channel"):
            validator.verify_registry_metadata(
                spec, [row], registry, selection="default"
            )
        document["dist-tags"]["latest"] = spec["version"]
        validator.verify_registry_metadata(spec, [row], registry, selection="default")
        document["dist-tags"]["mcp"] = "0.1.0-alpha.22.mcp.2"
        with self.assertRaisesRegex(ValueError, "candidate channel"):
            validator.verify_registry_metadata(spec, [row], registry)
        document["dist-tags"]["mcp"] = spec["version"]
        published["dist"]["tarball"] = "https://unrelated.example.test/release.tgz"
        with self.assertRaisesRegex(ValueError, "selected HTTPS registry"):
            validator.verify_registry_metadata(spec, [row], registry)


if __name__ == "__main__":
    unittest.main()
