"""Native installed identity checks and acceptance-stage invocations."""

import base64
import os
from pathlib import Path
import platform
import subprocess
import sys

from airs_release_contract import PATTERNS, RESULTS, package_summary
from airs_release_receipts import atomic_json, evidence_path
from airs_test_release_spec import (
    TARGETS,
    package_order,
    canonical_digest,
    digest_file,
    load_json,
    require,
)


def observed_target():
    host = (platform.system(), platform.machine())
    known = {
        ("Linux", "x86_64"): "x86_64-unknown-linux-musl",
        ("Linux", "aarch64"): "aarch64-unknown-linux-musl",
        ("Linux", "arm64"): "aarch64-unknown-linux-musl",
        ("Darwin", "arm64"): "aarch64-apple-darwin",
    }
    require(
        host in known, "Acceptance requires native Linux x64/ARM64 or Apple Silicon"
    )
    return known[host]


def candidate_identity(spec, packages):
    manifest = load_json(packages / "NPM-PACKAGES.json")
    summary = package_summary(spec, manifest)
    for row in manifest["publish_order"]:
        require(row.get("version") == spec["version"], "Candidate version mismatch")
        archive = evidence_path(packages, "tarballs/" + row["filename"])
        digest = digest_file(archive)
        integrity = (
            "sha512-"
            + base64.b64encode(bytes.fromhex(digest_file(archive, "sha512"))).decode()
        )
        require(
            digest == row.get("sha256") and integrity == row.get("integrity"),
            "Candidate archive integrity mismatch",
        )
    return {
        "candidate_metadata_sha256": digest_file(packages / "NPM-PACKAGES.json"),
        "candidate_packages_sha256": canonical_digest(summary),
    }


def clean_environment():
    allowed = {
        "PATH",
        "HOME",
        "USER",
        "USERPROFILE",
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        "TMPDIR",
        "TMP",
        "TEMP",
        "SHELL",
        "DISPLAY",
        "WAYLAND_DISPLAY",
        "DBUS_SESSION_BUS_ADDRESS",
        "XDG_RUNTIME_DIR",
        "GNOME_KEYRING_CONTROL",
    }
    environment = {key: value for key, value in os.environ.items() if key in allowed}
    environment.update(TERM="xterm-256color", PYTHONOPTIMIZE="0")
    return environment


def installed_identity(spec, packages, prefix, target, environment):
    from airs_bundle import verify_bundle

    launcher = prefix / "lib/node_modules/airs-harness"
    manifest = load_json(launcher / "package.json")
    require(
        manifest.get("name") == "airs-harness"
        and manifest.get("version") == spec["version"],
        "Installed launcher identity mismatch",
    )
    require(
        manifest.get("optionalDependencies")
        == {name: spec["version"] for name in package_order(spec)[:-1]},
        "Installed launcher native dependencies mismatch",
    )
    if spec["scope"] == "owner-authorized-mac-preview":
        require(
            manifest.get("os") == ["darwin"] and manifest.get("cpu") == ["arm64"],
            "Mac preview launcher must reject other platforms",
        )
    native_manifest = subprocess.check_output(
        [
            "node",
            "-e",
            "const r=require('module').createRequire(process.argv[1]);console.log(r.resolve(process.argv[2]+'/package.json'));",
            str(launcher / "package.json"),
            TARGETS[target],
        ],
        env=environment,
        text=True,
        timeout=20,
    ).strip()
    native_package = Path(native_manifest).resolve().parent
    require(
        native_package.is_relative_to(prefix.resolve()),
        "Installed native escapes its prefix",
    )
    native = native_package / "bin/airs-harness"
    info = load_json(native_package / "BUILD-INFO.json")
    native_info = load_json(native_package / "package.json")
    expected = next(
        row["binary_sha256"] for row in spec["platforms"] if row["target"] == target
    )
    require(
        native_info.get("name") == TARGETS[target]
        and native_info.get("version") == spec["version"],
        "Installed native package identity mismatch",
    )
    require(
        info.get("source_commit") == spec["source_commit"]
        and info.get("target") == target
        and info.get("version") == spec["version"]
        and info.get("binary_sha256") == expected,
        "Installed native provenance mismatch",
    )
    require(digest_file(native) == expected, "Actual installed native bytes changed")
    candidate = load_json(packages / "NPM-PACKAGES.json")
    tooling = load_json(launcher / "PACKAGE-TOOLING.json")
    require(
        tooling == candidate.get("package_tooling")
        and tooling.get("packaging_commit") == spec["packaging_commit"],
        "Installed packaging tooling mismatch",
    )
    installed_files = {
        "npm/airs-harness/" + path.relative_to(launcher).as_posix()
        for directory in ("lib", "bin", "managed-cli")
        for path in (launcher / directory).iterdir()
        if path.is_file()
    }
    expected_files = {
        relative
        for relative in tooling.get("files", {})
        if relative.startswith(
            (
                "npm/airs-harness/lib/",
                "npm/airs-harness/bin/",
                "npm/airs-harness/managed-cli/",
            )
        )
    }
    require(
        installed_files and installed_files == expected_files,
        "Installed launcher tooling inventory changed",
    )
    for relative, expected_hash in tooling.get("files", {}).items():
        if relative.startswith(
            (
                "npm/airs-harness/lib/",
                "npm/airs-harness/bin/",
                "npm/airs-harness/managed-cli/",
            )
        ):
            require(
                digest_file(
                    evidence_path(launcher, relative.removeprefix("npm/airs-harness/"))
                )
                == expected_hash,
                "Installed launcher tooling bytes changed",
            )
    inventory = launcher / "BUNDLE-INVENTORY.json"
    require(
        digest_file(inventory)
        == candidate.get("cli_bundle", {}).get("inventory_sha256"),
        "Installed managed CLI inventory mismatch",
    )
    verify_bundle(launcher, load_json(inventory), require_windows_wrappers=False)
    command = prefix / "bin/airs"
    require(
        command.resolve(strict=True).is_relative_to(launcher.resolve()),
        "Installed launcher command escapes package",
    )
    version = subprocess.check_output(
        [str(command), "--version"], env=environment, text=True, timeout=20
    ).strip()
    require(
        version == "airs " + spec["version"],
        "Actual installed executable version mismatch",
    )
    return command, native, launcher


def invocation(
    name,
    spec,
    target,
    scripts,
    packages,
    work,
    prefix,
    installed,
    installation="candidate",
):
    python = (
        sys.executable
    )  # No -O; clean_environment forces assertions on in legacy validators.
    if name == "install":
        if installation == "registry":
            atomic_json(work / "SPEC.json", spec)
            return [
                python,
                str(scripts / "validate_airs_test_registry_install.py"),
                "--spec",
                str(work / "SPEC.json"),
                "--packages",
                str(packages),
                "--prefix",
                str(prefix),
            ], prefix
        return [
            python,
            str(scripts / "validate_airs_npm.py"),
            "--packages",
            str(packages),
            "--prefix",
            str(prefix),
        ], prefix
    command, native, launcher = installed
    if name in PATTERNS:
        return [
            python,
            str(scripts / "airs_release_unittest.py"),
            "--scripts",
            str(scripts),
            "--pattern",
            PATTERNS[name],
            "--receipt",
            str(work / RESULTS[name]),
        ], work
    if name == "onboarding":
        validator = (
            "validate_airs_onboarding_macos.py"
            if target == "aarch64-apple-darwin"
            else "validate_airs_onboarding.py"
        )
        command_args = [
            python,
            str(scripts / validator),
            "--binary",
            str(command),
            "--native-binary",
            str(native),
            "--output",
            str(work),
        ]
        return (
            command_args
            if target == "aarch64-apple-darwin"
            else ["dbus-run-session", "--", *command_args]
        ), work
    if name == "terminals":
        args = [
            python,
            str(scripts / "validate_airs_onboarding_terminals.py"),
            "--binary",
            str(command),
            "--native-binary",
            str(native),
            "--output",
            str(work),
        ]
        return (
            args
            + (["--shells", "bash", "zsh"] if target == "aarch64-apple-darwin" else []),
            work,
        )
    if name == "managed-cli":
        return [
            python,
            str(scripts / "validate_prisma_cli.py"),
            "--launcher",
            str(launcher / "bin/airs.js"),
            "--receipt",
            str(work / RESULTS[name]),
        ], work
    if name == "upgrade":
        arguments = [
            python,
            str(scripts / "validate_airs_npm_upgrade.py"),
            "--packages",
            str(packages),
            "--previous",
            spec["previous_version"],
            "--registry",
            spec["registry"],
            "--output",
            str(work / "upgrade"),
        ]
        if "previous_release" in spec:
            baseline = spec["previous_release"]
            arguments.extend(
                [
                    "--previous-source-commit",
                    baseline["source_commit"],
                    "--previous-native-sha256",
                    next(
                        row["binary_sha256"]
                        for row in baseline["platforms"]
                        if row["target"] == target
                    ),
                ]
            )
        return arguments, work / "upgrade"
    if name == "command-output":
        return [
            python,
            str(scripts / "validate_airs_command_output.py"),
            "--binary",
            str(command),
            "--native",
            str(native),
            "--scripts",
            str(scripts),
            "--receipt",
            str(work / RESULTS[name]),
        ], work
    require(name == "mac-signature", "Unknown acceptance stage")
    requirement = (
        '=anchor apple generic and certificate leaf[subject.OU] = "'
        + spec["developer_id_team"]
        + '" and certificate 1[field.1.2.840.113635.100.6.2.6] exists and certificate leaf[field.1.2.840.113635.100.6.1.13] exists'
    )
    return [
        [
            "/usr/bin/codesign",
            "--verify",
            "--strict",
            "--verbose=2",
            "-R",
            requirement,
            str(native),
        ],
        [
            "/usr/bin/codesign",
            "--verify",
            "--strict",
            "--verbose=4",
            "--check-notarization",
            "-R",
            "=notarized",
            str(native),
        ],
    ], work
