"""Exercise the real bundler boundary with deterministic synthetic npm archives."""

import base64
import copy
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import airs_bundle as bundle
from airs_bundle_shims import EXTENSIONS, TEMPLATES


class BundleTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
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

    def windows_wrappers(self, version=0, target="../@cdot65/prisma-airs-cli/run.js"):
        templates = json.loads(TEMPLATES.read_text())["variants"][version]["templates"]
        paths = {}
        for suffix in EXTENSIONS:
            path = self.launcher / ("node_modules/.bin/airs" + suffix)
            path.parent.mkdir(exist_ok=True)
            path.write_bytes(
                templates[suffix]
                .replace(
                    "{{TARGET}}",
                    target.replace("/", "\\") if suffix == ".cmd" else target,
                )
                .encode()
            )
            paths[suffix] = path
        return paths

    def windows_bundle(self, script=b"#!/usr/bin/env node\nconsole.log('test');\n"):
        self.add_package(
            bundle.CLI,
            "5.2.0",
            {bundle.SDK: "0.28.0", "sharp": "1.0.0"},
            bin={"airs": "run.js"},
            contents={"run.js": script},
        )
        return self.build()

    def test_windows_complete_wrapper_families_and_receipts(self):
        inventory = self.windows_bundle()
        for version, expected in enumerate(("7.0.0", "9.0.2")):
            paths = self.windows_wrappers(version)
            receipt = bundle.verify_bundle(self.launcher, inventory)[
                "windows_command_wrappers"
            ]
            self.assertEqual(len(receipt), 1)
            self.assertEqual(receipt[0]["template_cmd_shim"], expected)
            self.assertEqual(
                receipt[0]["files"],
                {
                    str(p.relative_to(self.launcher)): hashlib.sha256(
                        p.read_bytes()
                    ).hexdigest()
                    for p in paths.values()
                },
            )

    def test_windows_wrapper_mutations_and_partial_sets_rejected(self):
        inventory = self.windows_bundle()
        for version in (0, 1):
            for suffix in EXTENSIONS:
                paths = self.windows_wrappers(version)
                original = paths[suffix].read_bytes()
                mutations = (
                    original + b"\necho injected\n",
                    original[:-1],
                    b"\xef\xbb\xbf" + original,
                    original.replace(b"run.js", b"other.js"),
                    original.replace(b"node", b"evil", 1),
                )
                for payload in mutations:
                    with self.subTest(
                        version=version, suffix=suffix, payload=payload[-30:]
                    ):
                        paths[suffix].write_bytes(payload)
                        with self.assertRaisesRegex(ValueError, "wrappers differ"):
                            bundle.verify_bundle(self.launcher, inventory)
                paths[suffix].unlink()
                with self.assertRaisesRegex(ValueError, "Incomplete Windows"):
                    bundle.verify_bundle(self.launcher, inventory)

    def test_windows_mixed_families_and_nonregular_members_rejected(self):
        inventory = self.windows_bundle()
        older = self.windows_wrappers(0)[".cmd"].read_bytes()
        paths = self.windows_wrappers(1)
        paths[".cmd"].write_bytes(older)
        with self.assertRaisesRegex(ValueError, "wrappers differ"):
            bundle.verify_bundle(self.launcher, inventory)
        paths = self.windows_wrappers()
        paths[".cmd"].unlink()
        paths[".cmd"].mkdir()
        with self.assertRaisesRegex(ValueError, "regular file"):
            bundle.verify_bundle(self.launcher, inventory)

    def test_windows_unsupported_shebang_rejected(self):
        inventory = self.windows_bundle(b"#!/usr/bin/env node --inspect\nbody\n")
        self.windows_wrappers()
        with self.assertRaisesRegex(ValueError, "Unsupported bundled command shebang"):
            bundle.verify_bundle(self.launcher, inventory)

    def test_declared_wrapper_namespace_collisions_rejected_during_staging(self):
        for names in (("Tool", "tool"), ("airs", "airs.cmd"), ("é", "e\u0301")):
            with self.subTest(names=names):
                self.add_package(
                    bundle.CLI,
                    "5.2.0",
                    {bundle.SDK: "0.28.0", "sharp": "1.0.0"},
                    bin={name: "run.js" for name in names},
                )
                with self.assertRaisesRegex(ValueError, "wrapper paths conflict"):
                    self.build()
                self.assertFalse((self.launcher / "node_modules").exists())

    def test_windows_target_metacharacters_and_wrapper_bounds_rejected(self):
        self.add_package(
            bundle.CLI,
            "5.2.0",
            {bundle.SDK: "0.28.0", "sharp": "1.0.0"},
            bin={"airs": "run$bad.js"},
            contents={"run$bad.js": b"#!/usr/bin/env node\nbody\n"},
        )
        inventory = self.build()
        self.windows_wrappers(target="../@cdot65/prisma-airs-cli/run$bad.js")
        with self.assertRaisesRegex(ValueError, "Unsupported bundled command path"):
            bundle.verify_bundle(self.launcher, inventory)
        # Restore a supported hashed target before isolating the wrapper size check.
        for path in (self.launcher / "node_modules/.bin").iterdir():
            path.unlink()
        shutil.rmtree(self.launcher / "node_modules")
        inventory = self.windows_bundle()
        paths = self.windows_wrappers()
        paths[".cmd"].write_bytes(b"x" * (16 * 1024 + 1))
        with self.assertRaises(ValueError):
            bundle.verify_bundle(self.launcher, inventory)

    @unittest.skipIf(os.name == "nt", "POSIX synthetic symlink/FIFO wrapper rejection")
    def test_windows_wrapper_symlink_and_fifo_rejected_without_read(self):
        inventory = self.windows_bundle()
        paths = self.windows_wrappers()
        paths[".cmd"].unlink()
        paths[".cmd"].symlink_to("airs.ps1")
        with self.assertRaisesRegex(ValueError, "regular file"):
            bundle.verify_bundle(self.launcher, inventory)
        paths[".cmd"].unlink()
        os.mkfifo(paths[".cmd"])
        with self.assertRaisesRegex(ValueError, "regular file"):
            bundle.verify_bundle(self.launcher, inventory)

    def test_windows_generated_shim_fails_explicitly(self):
        inventory = self.build()
        path = self.launcher / "node_modules/.bin/airs.cmd"
        path.parent.mkdir()
        path.write_text("unverified generated executable")
        with self.assertRaisesRegex(ValueError, "Unsupported bundled command shebang"):
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

    @unittest.skipIf(os.name == "nt", "POSIX temporary-directory alias regression")
    def test_reparse_fixture_handles_aliased_temporary_directory(self):
        directory = self.root / "actual-temp"
        directory.mkdir()
        alias = self.root / "aliased-temp"
        alias.symlink_to(directory, target_is_directory=True)
        result = subprocess.run(
            [
                sys.executable,
                "-m",
                "unittest",
                "test_airs_bundle.BundleTests.test_windows_reparse_metadata_rejected_before_enumeration_or_read",
            ],
            cwd=Path(__file__).resolve().parent,
            env=os.environ
            | {"TMPDIR": str(alias), "TMP": str(alias), "TEMP": str(alias)},
            capture_output=True,
            text=True,
            timeout=30,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

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
