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
    def package(
        self,
        directory,
        target,
        candidate=False,
        build_command=None,
        signing_overrides=None,
        validated=False,
        package_version="0.1.0-alpha.10",
        emulator=None,
    ):
        root = directory / "source"
        root.mkdir()
        (root / "codex-rs").mkdir()
        (root / "codex-rs/Cargo.lock").write_text("fixture lock")
        (root / "scripts").mkdir()
        (root / "npm/airs-harness").mkdir(parents=True)
        (root / "npm/airs-harness/package.json").write_text(
            json.dumps({"version": package_version})
        )
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
            "test_airs_harness_mcp_login.py",
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
        vendored = root / "third_party/wezterm"
        vendored.mkdir(parents=True)
        (vendored / "LICENSE").write_text("vendored MIT notice")
        for name in ["vendor/bubblewrap", "bwrap"]:
            source = root / "codex-rs" / name
            source.mkdir(parents=True, exist_ok=True)
            (source / "COPYING").write_text("fixture source and license")
        (root / "codex-rs/rust-toolchain.toml").write_text("fixture toolchain")
        (root / "codex-rs/Cargo.toml").write_text("fixture workspace")
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

        probe = [str(binary), "--version"]
        if emulator is not None:
            probe.insert(0, emulator)

        def command(argv, **kwargs):
            if argv == probe:
                return "airs-harness 0.1.0-alpha.10\n"
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
        if emulator is not None:
            args.extend(["--emulator", emulator])
        if validated:
            evidence = directory / "evidence"
            evidence.mkdir()
            records = []
            for role in [
                "installed-runtime",
                "package-integrity",
                "independent-review",
                "native-provenance",
            ]:
                path = evidence / (role + ".txt")
                path.write_text("Disposable packaging test evidence, not a real review")
                records.append(
                    {"role": role, "path": path.name, "sha256": PACKAGER.digest(path)}
                )
            validation = {
                "schema_version": 1,
                "scope": "signed-prerelease-distribution",
                "passed": True,
                "release_ready": True,
                "full_authentication_release_ready": False,
                "binary_sha256": PACKAGER.digest(binary),
                "target": target,
                "product_version": "0.1.0-alpha.10",
                "source_commit": "a" * 40,
                "independent_review": {
                    "scope": "signed-prerelease-distribution",
                    "reviewer": "unit fixture",
                    "verdict": "pass",
                    "score": 9,
                },
                "evidence": records,
                "signing": {"kind": "linux-provenance-checksums"},
            }
            receipt = directory / "validation.json"
            receipt.write_text(json.dumps(validation))
            args.extend(
                [
                    "--validation",
                    str(receipt),
                    "--validation-evidence-root",
                    str(evidence),
                ]
            )
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

    def test_mismatched_native_version_is_rejected_before_packaging(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaisesRegex(ValueError, "versions differ"):
                self.package(
                    Path(temporary),
                    "aarch64-apple-darwin",
                    candidate=True,
                    package_version="0.1.0-alpha.20",
                )

    def test_review_archive_retains_bound_evidence_without_full_release_claim(self):
        with tempfile.TemporaryDirectory() as directory:
            archive, binary = self.package(
                Path(directory), "x86_64-unknown-linux-musl", validated=True
            )
            with tarfile.open(archive) as tar:
                root = tar.getnames()[0]
                info = json.load(tar.extractfile(root + "/BUILD-INFO.json"))
                self.assertEqual(
                    tar.extractfile(root + "/licenses/vendored/wezterm-LICENSE").read(),
                    b"vendored MIT notice",
                )
                self.assertEqual(
                    tar.extractfile(
                        root + "/licenses/bubblewrap-source/vendor/bubblewrap/COPYING"
                    ).read(),
                    b"fixture source and license",
                )
                validation_bytes = tar.extractfile(root + "/VALIDATION.json").read()
                self.assertEqual(
                    info["validation_receipt_sha256"],
                    PACKAGER.hashlib.sha256(validation_bytes).hexdigest(),
                )
                validation = json.loads(validation_bytes)
                self.assertFalse(validation["full_authentication_release_ready"])
                self.assertEqual(
                    info["release_scope"], "signed-prerelease-distribution"
                )
                for record in validation["evidence"]:
                    data = tar.extractfile(
                        root + "/validation-evidence/" + record["path"]
                    ).read()
                    self.assertEqual(
                        PACKAGER.hashlib.sha256(data).hexdigest(), record["sha256"]
                    )

    def test_verified_signature_is_preserved_without_claiming_release_acceptance(self):
        with tempfile.TemporaryDirectory() as directory:
            archive, binary = self.package(
                Path(directory),
                "aarch64-apple-darwin",
                candidate=True,
                signing_overrides={},
            )
            with tarfile.open(archive) as tar:
                root = tar.getnames()[0]
                self.assertEqual(tar.extractfile(root + "/airs-harness").read(), binary)
                info = json.load(tar.extractfile(root + "/BUILD-INFO.json"))
                validation = json.load(tar.extractfile(root + "/VALIDATION.json"))
                signing = tar.extractfile(root + "/SIGNING.json").read()
                self.assertEqual(
                    info["signing_receipt_sha256"],
                    PACKAGER.hashlib.sha256(signing).hexdigest(),
                )
                self.assertEqual(info["release_status"], "signed-unvalidated-candidate")
                self.assertFalse(info["publishable"])
                self.assertFalse(validation["release_ready"])
                self.assertNotIn("production signing", validation["remaining"])

    def test_signing_proof_must_match_binary_source_team_and_success(self):
        for field, value in [
            ("binary_sha256", "0" * 64),
            ("source_commit", "b" * 40),
            ("team_id", "OTHERTEAM"),
            ("codesign_verified", False),
            ("hardened_runtime", 1),
            ("notarization_verified", "true"),
        ]:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                with self.assertRaisesRegex(ValueError, "Signing receipt"):
                    self.package(
                        Path(directory),
                        "aarch64-apple-darwin",
                        candidate=True,
                        signing_overrides={field: value},
                    )

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

    def test_unix_candidates_keep_existing_binary_and_block_publication(self):
        for target in [
            "x86_64-unknown-linux-musl",
            "aarch64-unknown-linux-musl",
            "aarch64-apple-darwin",
        ]:
            with (
                self.subTest(target=target),
                tempfile.TemporaryDirectory() as directory,
            ):
                archive, binary = self.package(Path(directory), target, candidate=True)
                with tarfile.open(archive) as tar:
                    root = tar.getnames()[0]
                    self.assertEqual(root, archive.name.removesuffix(".tar.gz"))
                    self.assertEqual(
                        tar.extractfile(root + "/airs-harness").read(), binary
                    )
                    info = json.load(tar.extractfile(root + "/BUILD-INFO.json"))
                    self.assertEqual(
                        info["release_status"], "unsigned-unvalidated-candidate"
                    )
                    self.assertFalse(info["publishable"])
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

    def test_linux_arm64_candidate_records_emulated_version_probe(self):
        with tempfile.TemporaryDirectory() as directory:
            archive, binary = self.package(
                Path(directory),
                "aarch64-unknown-linux-musl",
                candidate=True,
                emulator="qemu-aarch64",
            )
            self.assertTrue(archive.name.endswith("-linux-aarch64-musl.tar.gz"))
            with tarfile.open(archive) as tar:
                root = tar.getnames()[0]
                self.assertEqual(tar.extractfile(root + "/airs-harness").read(), binary)
                info = json.load(tar.extractfile(root + "/BUILD-INFO.json"))
                self.assertEqual(info["target"], "aarch64-unknown-linux-musl")
                self.assertEqual(info["emulated_version_probe"], "qemu-aarch64")
                self.assertFalse(info["publishable"])

    def test_native_probe_records_no_emulator(self):
        with tempfile.TemporaryDirectory() as directory:
            archive, _ = self.package(
                Path(directory), "x86_64-unknown-linux-musl", candidate=True
            )
            with tarfile.open(archive) as tar:
                root = tar.getnames()[0]
                info = json.load(tar.extractfile(root + "/BUILD-INFO.json"))
                self.assertNotIn("emulated_version_probe", info)

    def test_missing_validation_cannot_inherit_historical_release_claim(self):
        for target in [
            "x86_64-unknown-linux-musl",
            "aarch64-unknown-linux-musl",
            "aarch64-apple-darwin",
        ]:
            with (
                self.subTest(target=target),
                tempfile.TemporaryDirectory() as directory,
                contextlib.redirect_stderr(io.StringIO()),
            ):
                with self.assertRaises(SystemExit) as error:
                    self.package(Path(directory), target)
                self.assertEqual(error.exception.code, 2)


class NpmCandidatePackaging(unittest.TestCase):
    def test_signed_npm_candidate_preserves_evidence_and_rejects_tampering(self):
        template = json.loads((ROOT / "npm/airs-harness/package.json").read_text())
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            release = root / "native"
            release.mkdir()
            (release / "airs-harness").write_bytes(b"signed fixture bytes")
            for name in ["LICENSE", "NOTICE", "DEPENDENCIES.json"]:
                (release / name).write_text("{}")
            (release / "licenses").mkdir()
            signing = release / "SIGNING.json"
            signing.write_text('{"fixture":true}\n')
            info = {
                "version": template["version"],
                "product": "Prisma AIRS Harness",
                "source_commit": "a" * 40,
                "target": "aarch64-apple-darwin",
                "binary_sha256": PACKAGER.digest(release / "airs-harness"),
                "signing_receipt_sha256": PACKAGER.digest(signing),
                "publishable": False,
                "release_status": "signed-unvalidated-candidate",
            }
            (release / "BUILD-INFO.json").write_text(json.dumps(info))

            def package(output):
                return subprocess.run(
                    [
                        sys.executable,
                        str(ROOT / "scripts/package_airs_npm.py"),
                        "--release-directory",
                        str(release),
                        "--scoped",
                        "--output-directory",
                        str(output),
                        "--registry",
                        "https://npm.pkg.github.com",
                    ],
                    capture_output=True,
                    text=True,
                    timeout=60,
                )

            output = root / "valid"
            result = package(output)
            self.assertEqual(result.returncode, 0, result.stderr)
            for archive in (output / "tarballs").glob("*darwin*.tgz"):
                with tarfile.open(archive) as tar:
                    self.assertEqual(
                        tar.extractfile("package/SIGNING.json").read(),
                        signing.read_bytes(),
                    )
                    manifest = json.load(tar.extractfile("package/package.json"))
                    self.assertTrue(manifest["private"])
                    self.assertEqual(
                        tar.extractfile("package/bin/airs-harness").read(),
                        (release / "airs-harness").read_bytes(),
                    )
            self.assertEqual(len(list((output / "tarballs").glob("*darwin*.tgz"))), 1)
            signing.write_text('{"tampered":true}\n')
            result = package(root / "tampered")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Signing receipt differs", result.stderr)

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
                            "@cdot65/prisma-airs-harness" if scoped else "prisma-airs-harness"
                        ):
                            self.assertEqual(
                                manifest["dependencies"], template["dependencies"]
                            )
                            self.assertNotIn(
                                "package/AUTHENTICATION-ONBOARDING.md", tar.getnames()
                            )
                            for guide in [
                                "GETTING-STARTED.md",
                                "MACOS.md",
                                "PRISMA-AIRS-CLI.md",
                                "UBUNTU-TEST-HOST.md",
                                "scripts/prepare_airs_ubuntu.sh",
                            ]:
                                self.assertIn(guide, manifest["files"])
                                self.assertEqual(
                                    tar.extractfile("package/" + guide).read(),
                                    (ROOT / guide).read_bytes(),
                                )
                            self.assertEqual(
                                tar.extractfile("package/README.md").read(),
                                (ROOT / "npm/airs-harness/README.md").read_bytes(),
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
