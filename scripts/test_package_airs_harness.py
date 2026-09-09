"""Packaging contracts using disposable binaries; no native E2E claim."""

import contextlib
import importlib.util
import io
import json
import itertools
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "package_airs_harness", ROOT / "scripts/package_airs_harness.py"
)
PACKAGER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PACKAGER)


class NativePackaging(unittest.TestCase):
    def package(self, directory, target, candidate=False, build_command=None,
                signing_overrides=None):
        root = directory / "source"
        root.mkdir()
        (root / "codex-rs").mkdir()
        (root / "codex-rs/Cargo.lock").write_text("fixture lock")
        (root / "scripts").mkdir()
        for name in [
            "LICENSE",
            "NOTICE",
            "README.md",
            "RELEASE.md",
            "RENAME.md",
            "MACOS.md",
            "PRISMA-AIRS-CLI.md",
            "UPSTREAM.md",
            "BASELINE.json",
            "VALIDATION.json",
            "AGENTS.md",
            "README.upstream.md",
            "IMPLEMENTATION.md",
            "PLAN.md",
        ]:
            (root / name).write_text('{"passed":true}')
        for name in [
            "test_airs_harness.py",
            "test_airs_harness_pty.py",
            "airs_harness_pty.py",
            "validate_live_agent.py",
            "validate_live_model_switch.py",
            "validate_live_gateway.py",
            "validate_mcp_sessions.py",
            "verify_airs_release.py",
            "check_airs_endpoints.py",
            "validate_auth_cli.py",
            "validate_auth_cli.py.lock",
            "validate_rust_oidc.py",
            "validate_rust_oidc.py.lock",
            "validate_native_credentials.py",
            "airs_oidc_interactive.py",
            "verify_persisted_airs_audit.py",
            "verify_management_airs_audit.mjs",
        ]:
            (root / "scripts" / name).touch()
        notices = directory / "sysroot/share/doc/rust"
        (notices / "licenses").mkdir(parents=True)
        for name in ["COPYRIGHT.html", "COPYRIGHT-library.html", "licenses/LICENSE"]:
            (notices / name).write_text("test license")
        binary = directory / "disposable-native-fixture"
        binary.write_bytes(b"not an executable: packaging contract only")
        # The packer resolves Windows short-name and macOS temporary-path aliases.
        # Match the same existing file without relaxing the command whitelist.
        binary = binary.resolve(strict=True)
        metadata = directory / "metadata.json"
        metadata.write_text(json.dumps({"packages": [], "resolve": {"nodes": []}}))

        def command(argv, **kwargs):
            if argv == [str(binary), "--version"]:
                return "airs-harness 0.1.0-test\n"
            if argv[:3] == ["git", "rev-parse", "HEAD"]:
                return "a" * 40
            if argv[:2] == ["git", "status"]:
                return ""
            if argv[:2] == ["git", "ls-files"]:
                return b""
            if argv == ["rustc", "--print", "sysroot"]:
                return str(directory / "sysroot")
            if argv == ["rustc", "--version"]:
                return "rustc fixture"
            raise AssertionError(f"Unexpected external command: {argv}")

        args = [
            "package",
            "--binary",
            str(binary),
            "--source-directory",
            str(root),
            "--metadata",
            str(metadata),
            "--output-directory",
            str(directory / "output"),
            "--target",
            target,
        ]
        if candidate:
            args.append("--unvalidated-candidate")
        if build_command is not None:
            args.extend(["--build-command", build_command])
        if signing_overrides is not None:
            signing = {
                "binary_sha256": PACKAGER.digest(binary),
                "target": "aarch64-apple-darwin",
                "source_commit": "a" * 40,
                "team_id": "G5QLZ5A8TA",
                "codesign_verified": True,
                "hardened_runtime": True,
                "notarization_verified": True,
            }
            signing.update(signing_overrides)
            receipt = directory / "signing.json"
            receipt.write_text(json.dumps(signing))
            args.extend(["--signing-receipt", str(receipt)])
        with (
            patch.object(sys, "argv", args),
            patch.object(PACKAGER.subprocess, "check_output", command),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            PACKAGER.main()
        return next((directory / "output").glob("*.tar.gz")), binary.read_bytes()

    def test_verified_signature_is_preserved_without_claiming_release_acceptance(self):
        with tempfile.TemporaryDirectory() as directory:
            archive, binary = self.package(
                Path(directory), "aarch64-apple-darwin", candidate=True,
                signing_overrides={},
            )
            with tarfile.open(archive) as tar:
                root = tar.getnames()[0]
                self.assertEqual(tar.extractfile(root + "/airs-harness").read(), binary)
                info = json.load(tar.extractfile(root + "/BUILD-INFO.json"))
                validation = json.load(tar.extractfile(root + "/VALIDATION.json"))
                signing = tar.extractfile(root + "/SIGNING.json").read()
                self.assertEqual(info["signing_receipt_sha256"],
                                 PACKAGER.hashlib.sha256(signing).hexdigest())
                self.assertEqual(info["release_status"], "signed-unvalidated-candidate")
                self.assertFalse(info["publishable"])
                self.assertFalse(validation["release_ready"])
                self.assertNotIn("production signing", validation["remaining"])

    def test_signing_proof_must_match_binary_source_team_and_success(self):
        for field, value in [
            ("binary_sha256", "0" * 64), ("source_commit", "b" * 40),
            ("team_id", "OTHERTEAM"), ("codesign_verified", False),
            ("hardened_runtime", 1), ("notarization_verified", "true"),
        ]:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                with self.assertRaisesRegex(ValueError, "Signing receipt"):
                    self.package(Path(directory), "aarch64-apple-darwin",
                                 candidate=True, signing_overrides={field: value})

    def test_windows_archive_preserves_exe_and_marks_inherited_validation_unusable(
        self,
    ):
        with tempfile.TemporaryDirectory() as directory:
            archive, binary = self.package(
                Path(directory), "x86_64-pc-windows-msvc", candidate=True
            )
            with tarfile.open(archive) as tar:
                root = tar.getnames()[0]
                self.assertEqual(
                    tar.extractfile(root + "/airs-harness.exe").read(), binary
                )
                self.assertNotIn(root + "/airs-harness", tar.getnames())
                info = json.load(tar.extractfile(root + "/BUILD-INFO.json"))
                validation = json.load(tar.extractfile(root + "/VALIDATION.json"))
                self.assertFalse(info["publishable"])
                self.assertEqual(
                    info["release_status"], "unsigned-unvalidated-candidate"
                )
                self.assertFalse(validation["passed"])
                self.assertFalse(validation["release_ready"])
                self.assertEqual(validation["binary_sha256"], info["binary_sha256"])
                sums = tar.extractfile(root + "/SHA256SUMS").read().decode()
                self.assertNotIn("\r", sums)
                self.assertIn(info["binary_sha256"] + "  airs-harness.exe\n", sums)
                checksum = archive.with_suffix(archive.suffix + ".sha256").read_bytes()
                self.assertNotIn(b"\r", checksum)

    def test_unix_archives_keep_existing_binary_and_release_semantics(self):
        for target in ["x86_64-unknown-linux-musl", "aarch64-apple-darwin"]:
            with (
                self.subTest(target=target),
                tempfile.TemporaryDirectory() as directory,
            ):
                archive, binary = self.package(Path(directory), target)
                with tarfile.open(archive) as tar:
                    root = tar.getnames()[0]
                    self.assertEqual(
                        tar.extractfile(root + "/airs-harness").read(), binary
                    )
                    info = json.load(tar.extractfile(root + "/BUILD-INFO.json"))
                    self.assertNotIn("release_status", info)
                    self.assertNotIn("publishable", info)
                    self.assertEqual(
                        info["build_command"],
                        "cargo build --locked --release -p codex-cli --bin airs-harness",
                    )

    def test_explicit_build_command_is_literal_metadata_and_preserves_binary(self):
        command = (
            "cargo --config profile.release.package.codex-cli.opt-level=1 "
            "build --locked --release -p codex-cli --bin airs-harness --timings "
            "# literal $(metadata-only); `metadata-only`"
        )
        with tempfile.TemporaryDirectory() as directory:
            archive, binary = self.package(
                Path(directory),
                "x86_64-unknown-linux-musl",
                candidate=True,
                build_command=command,
            )
            with tarfile.open(archive) as tar:
                root = tar.getnames()[0]
                info = json.load(tar.extractfile(root + "/BUILD-INFO.json"))
                self.assertEqual(info["build_command"], command)
                self.assertEqual(tar.extractfile(root + "/airs-harness").read(), binary)
                self.assertFalse(info["publishable"])

    def test_windows_requires_explicit_nonrelease_mode(self):
        with (
            tempfile.TemporaryDirectory() as directory,
            contextlib.redirect_stderr(io.StringIO()),
        ):
            with self.assertRaises(SystemExit) as error:
                self.package(Path(directory), "x86_64-pc-windows-msvc")
            self.assertEqual(error.exception.code, 2)


class NpmCandidatePackaging(unittest.TestCase):
    def test_candidate_tarballs_block_publication_and_keep_exact_cli_dependency(self):
        template = json.loads((ROOT / "npm/airs-harness/package.json").read_text())
        for candidate, scoped in itertools.product([False, True], repeat=2):
            with (
                self.subTest(candidate=candidate, scoped=scoped),
                tempfile.TemporaryDirectory() as directory,
            ):
                root = Path(directory)
                release = root / "native"
                release.mkdir()
                binary = release / "airs-harness.exe"
                binary.write_bytes(b"Windows packaging fixture only")
                for name in ["LICENSE", "NOTICE", "DEPENDENCIES.json"]:
                    (release / name).write_text("{}")
                (release / "licenses").mkdir()
                info = {
                    "version": template["version"],
                    "product": "Prisma AIRS Harness",
                    "source_commit": "a" * 40,
                    "target": "x86_64-pc-windows-msvc",
                    "binary_sha256": PACKAGER.digest(binary),
                }
                if candidate:
                    info.update(
                        publishable=False,
                        release_status="unsigned-unvalidated-candidate",
                    )
                (release / "BUILD-INFO.json").write_text(json.dumps(info))
                release_args = ["--release-directory", str(release)]
                dependencies = {"airs-harness-win32-x64": template["version"]}
                if candidate:
                    earlier = root / "already-validated-native"
                    shutil.copytree(release, earlier)
                    (earlier / "airs-harness.exe").rename(earlier / "airs-harness")
                    earlier_info = {
                        key: value
                        for key, value in info.items()
                        if key not in {"publishable", "release_status"}
                    }
                    earlier_info["target"] = "x86_64-unknown-linux-musl"
                    (earlier / "BUILD-INFO.json").write_text(json.dumps(earlier_info))
                    release_args = ["--release-directory", str(earlier), *release_args]
                    dependencies["airs-harness-linux-x64"] = template["version"]
                if scoped:
                    dependencies = {
                        "@cdot65/prisma-" + name: version
                        for name, version in dependencies.items()
                    }
                output = root / "npm"
                result = subprocess.run(
                    [
                        sys.executable,
                        str(ROOT / "scripts/package_airs_npm.py"),
                        *release_args,
                        *(["--scoped"] if scoped else []),
                        "--output-directory",
                        str(output),
                        "--registry",
                        "https://registry.example.invalid",
                    ],
                    capture_output=True,
                    text=True,
                    timeout=60,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                receipt = json.loads((output / "NPM-PACKAGES.json").read_text())
                self.assertEqual(
                    receipt.get("publishable"), False if candidate else None
                )
                for archive in (output / "tarballs").glob("*.tgz"):
                    with tarfile.open(archive) as tar:
                        manifest = json.load(tar.extractfile("package/package.json"))
                        self.assertEqual(manifest.get("private", False), candidate)
                        if manifest["name"] == (
                            "@cdot65/prisma-airs-harness" if scoped else "airs-harness"
                        ):
                            self.assertEqual(
                                manifest["dependencies"], template["dependencies"]
                            )
                            tooling = json.load(
                                tar.extractfile("package/PACKAGE-TOOLING.json")
                            )
                            self.assertEqual(tooling, receipt["package_tooling"])
                            self.assertEqual(tooling["native_source_commit"], "a" * 40)
                            self.assertEqual(
                                tooling["files"]["npm/airs-harness/lib/launcher.js"],
                                PACKAGER.digest(
                                    ROOT / "npm/airs-harness/lib/launcher.js"
                                ),
                            )
                            self.assertEqual(
                                manifest["optionalDependencies"],
                                dependencies,
                            )
                        else:
                            binary_member = (
                                "package/bin/airs-harness.exe"
                                if manifest["os"] == ["win32"]
                                else "package/bin/airs-harness"
                            )
                            self.assertEqual(
                                tar.extractfile(binary_member).read(),
                                binary.read_bytes(),
                            )
                            self.assertEqual(
                                manifest["name"].startswith("@cdot65/prisma-"), scoped
                            )


if __name__ == "__main__":
    unittest.main()
