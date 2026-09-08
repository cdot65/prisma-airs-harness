"""Exercise the real bundler boundary with deterministic synthetic npm archives."""

import base64
import copy
import hashlib
import io
import json
import os
from pathlib import Path
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import airs_bundle as bundle


class BundleTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.launcher = self.root / "launcher"
        self.launcher.mkdir()
        self.manifest = {
            "name": "test-harness",
            "version": "1.0.0",
            "private": True,
            "dependencies": {bundle.CLI: "5.2.0"},
        }
        (self.launcher / "package.json").write_text(json.dumps(self.manifest))
        self.targets = ["aarch64-apple-darwin"]
        self.packages = {"": {"dependencies": {bundle.CLI: "5.2.0"}}}
        self.archives = {}
        self.add_package(
            bundle.CLI,
            "5.2.0",
            {bundle.SDK: "0.28.0", "sharp": "1.0.0"},
            bin={"airs": "run.js"},
        )
        self.add_package(bundle.SDK, "0.28.0")
        self.add_package(
            "sharp",
            "1.0.0",
            {name: "1.0.0" for name in bundle.sharp_packages(self.targets)},
        )
        for name in bundle.sharp_packages(self.targets):
            self.add_package(name, "1.0.0")
        self.lock = self.root / "package-lock.json"

    def add_package(self, name, version, dependencies=None, contents=None, **extra):
        manifest = {"name": name, "version": version, **extra}
        if dependencies:
            manifest["dependencies"] = dependencies
        archive = io.BytesIO()
        with tarfile.open(fileobj=archive, mode="w:gz") as stream:
            for relative, content in {
                "package.json": json.dumps(manifest).encode(),
                "LICENSE": b"synthetic license",
                "run.js": b"console.log('test')",
                "CHANGELOG.md": b"source history",
                **(contents or {}),
            }.items():
                member = tarfile.TarInfo("package/" + relative)
                member.size = len(content)
                stream.addfile(member, io.BytesIO(content))
        blob = archive.getvalue()
        url = "https://registry.npmjs.org/" + name + "/-/test.tgz"
        self.archives[url] = blob
        self.packages["node_modules/" + name] = {
            "version": version,
            "resolved": url,
            "integrity": "sha512-"
            + base64.b64encode(hashlib.sha512(blob).digest()).decode(),
            **({"dependencies": dependencies} if dependencies else {}),
            **extra,
        }

    def build(self):
        self.lock.write_text(
            json.dumps({"lockfileVersion": 3, "packages": self.packages})
        )
        with patch.object(bundle, "download", side_effect=self.archives.__getitem__):
            return bundle.bundle_cli(self.lock, self.launcher, self.targets)

    def test_build_is_deterministic_and_preserves_pins_licenses_and_private(self):
        inventory = self.build()
        raw = (self.launcher / "BUNDLE-INVENTORY.json").read_bytes()
        self.assertEqual(
            json.loads((self.launcher / "package.json").read_text()), self.manifest
        )
        result = bundle.verify_bundle(self.launcher, inventory)
        self.assertEqual(result["packages"], 5)
        self.assertEqual(result["license_files"], 5)
        self.assertEqual(
            inventory["required_pins"], {bundle.CLI: "5.2.0", bundle.SDK: "0.28.0"}
        )
        self.launcher = self.root / "second"
        self.launcher.mkdir()
        (self.launcher / "package.json").write_text(json.dumps(self.manifest))
        self.assertEqual(self.build(), inventory)
        self.assertEqual((self.launcher / "BUNDLE-INVENTORY.json").read_bytes(), raw)

    def test_rejects_added_files_and_intel_payload(self):
        inventory = self.build()
        for relative in (
            "node_modules/@cdot65/prisma-airs-cli/unexpected-native.node",
            "node_modules/@img/sharp-darwin-x64/package.json",
        ):
            with self.subTest(relative=relative):
                path = self.launcher / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("unexpected")
                with self.assertRaisesRegex(ValueError, "Unexpected installed"):
                    bundle.verify_bundle(self.launcher, inventory)
                path.unlink()

    def test_missing_modified_and_optional_source_files(self):
        inventory = self.build()
        for relative in ("LICENSE", "package.json", "run.js", "CHANGELOG.md"):
            path = self.launcher / "node_modules" / bundle.CLI / relative
            original = path.read_bytes()
            path.write_bytes(b"{}")
            with self.assertRaises(ValueError):
                bundle.verify_bundle(self.launcher, inventory)
            path.write_bytes(original)
        optional = self.launcher / "node_modules" / bundle.CLI / "CHANGELOG.md"
        optional.unlink()
        bundle.verify_bundle(self.launcher, inventory)
        (self.launcher / "node_modules/@img/sharp-darwin-arm64/run.js").unlink()
        with self.assertRaises(FileNotFoundError):
            bundle.verify_bundle(self.launcher, inventory)

    def test_only_declared_bin_shebang_is_normalized_with_reversible_provenance(self):
        original = b"#!/usr/bin/env node\r\nconsole.log('body');\r\n"
        canonical = b"#!/usr/bin/env node\nconsole.log('body');\r\n"
        self.add_package(
            bundle.CLI,
            "5.2.0",
            {bundle.SDK: "0.28.0", "sharp": "1.0.0"},
            bin={"airs": "./run.js"},
            contents={"run.js": original, "other.js": original, "LICENSE": original},
        )
        inventory = self.build()
        base = self.launcher / "node_modules" / bundle.CLI
        self.assertEqual((base / "run.js").read_bytes(), canonical)
        self.assertEqual((base / "other.js").read_bytes(), original)
        self.assertEqual((base / "LICENSE").read_bytes(), original)
        package = next(
            record for record in inventory["packages"] if record["name"] == bundle.CLI
        )
        self.assertEqual(
            package["normalizations"],
            [
                {
                    "path": "run.js",
                    "transform": "npm-bin-shebang-crlf-to-lf-v1",
                    "original_sha256": hashlib.sha256(original).hexdigest(),
                    "normalized_sha256": hashlib.sha256(canonical).hexdigest(),
                }
            ],
        )
        bundle.verify_bundle(self.launcher, inventory)
        for changed in (
            original,
            canonical.replace(b"body", b"tampered"),
            canonical.replace(b";\r\n", b";\n"),
        ):
            (base / "run.js").write_bytes(changed)
            with self.assertRaisesRegex(ValueError, "Installed bundle file differs"):
                bundle.verify_bundle(self.launcher, inventory)
        (base / "run.js").write_bytes(canonical)
        package["normalizations"][0]["original_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "normalization provenance differs"):
            bundle.verify_bundle(self.launcher, inventory)

    def test_unstable_or_non_utf8_shebang_transforms_fail_closed(self):
        for script in (b"#!/usr/bin/node\r\r\nbody\n", b"#!/usr/bin/node\r\n\xff"):
            with self.subTest(script=repr(script)):
                self.add_package(
                    bundle.CLI,
                    "5.2.0",
                    {bundle.SDK: "0.28.0", "sharp": "1.0.0"},
                    bin={"airs": "run.js"},
                    contents={"run.js": script},
                )
                with self.assertRaises(ValueError):
                    self.build()
                self.assertFalse((self.launcher / "node_modules").exists())

    @unittest.skipIf(
        os.name == "nt", "POSIX symlink acceptance; Windows shim rejects separately"
    )
    def test_generated_bin_link_has_exact_target(self):
        inventory = self.build()
        link = self.launcher / "node_modules/.bin/airs"
        link.parent.mkdir()
        link.symlink_to("../@cdot65/prisma-airs-cli/run.js")
        bundle.verify_bundle(self.launcher, inventory)
        link.unlink()
        link.symlink_to("../@cdot65/prisma-airs-sdk/run.js")
        with self.assertRaisesRegex(ValueError, "declared target"):
            bundle.verify_bundle(self.launcher, inventory)

    def test_windows_generated_shim_fails_explicitly(self):
        inventory = self.build()
        path = self.launcher / "node_modules/.bin/airs.cmd"
        path.parent.mkdir()
        path.write_text("unverified generated executable")
        with self.assertRaisesRegex(
            ValueError, "Windows generated command shim validation is unsupported"
        ):
            bundle.verify_bundle(self.launcher, inventory)

    def test_windows_reparse_metadata_rejected_before_enumeration_or_read(self):
        inventory = self.build()
        original = Path.lstat
        target = self.launcher / "node_modules"

        def reparse(path, *args, **kwargs):
            info = original(path, *args, **kwargs)
            if path == target:
                return SimpleNamespace(st_mode=info.st_mode, st_file_attributes=0x400)
            return info

        with patch.object(Path, "lstat", reparse):
            with self.assertRaisesRegex(ValueError, "directories cannot"):
                bundle.verify_bundle(self.launcher, inventory)
        target = self.launcher / "package.json"
        with patch.object(Path, "lstat", reparse):
            with self.assertRaisesRegex(ValueError, "regular files"):
                bundle.read_file(target, 1024 * 1024)

    def test_locked_name_integrity_budget_and_peer_fail_closed(self):
        original = copy.deepcopy(self.packages)
        for failure in ("name", "integrity", "budget", "peer", "pin"):
            with self.subTest(failure=failure):
                self.packages = copy.deepcopy(original)
                cli = self.packages["node_modules/" + bundle.CLI]
                if failure == "name":
                    cli["resolved"] = self.packages["node_modules/" + bundle.SDK][
                        "resolved"
                    ]
                    cli["integrity"] = self.packages["node_modules/" + bundle.SDK][
                        "integrity"
                    ]
                elif failure == "integrity":
                    cli["integrity"] = "sha512-AAAA"
                elif failure == "peer":
                    cli["peerDependencies"] = {"missing-peer": "1.0.0"}
                elif failure == "pin":
                    cli["dependencies"][bundle.SDK] = "^0.28.0"
                with patch.object(
                    bundle, "MAX_TOTAL", 1 if failure == "budget" else bundle.MAX_TOTAL
                ):
                    with self.assertRaises(ValueError):
                        self.build()
                self.assertFalse((self.launcher / "node_modules").exists())
                self.assertFalse((self.launcher / "BUNDLE-INVENTORY.json").exists())

    def test_target_closure_and_unsupported_target(self):
        selected = bundle.sharp_packages(list(bundle.TARGETS))
        self.assertIn("@img/sharp-linuxmusl-arm64", selected)
        self.assertIn("@img/sharp-libvips-linux-arm64", selected)
        self.assertIn("@img/sharp-win32-arm64", selected)
        self.assertNotIn("@img/sharp-darwin-x64", selected)
        self.targets = ["x86_64-apple-darwin"]
        with patch.object(bundle, "download") as fetch:
            with self.assertRaisesRegex(ValueError, "Unsupported native target"):
                self.build()
            fetch.assert_not_called()


if __name__ == "__main__":
    unittest.main()
