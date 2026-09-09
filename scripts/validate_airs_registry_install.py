#!/usr/bin/env python3
"""Install exact published review packages from GitHub with a fresh npm cache.

This proves the workflow actor's access, not a teammate's read-only identity.
It does not publish, repack, build, or claim that all outbound traffic was observed.
"""

import argparse
import os
from pathlib import Path
import platform
import shutil
import subprocess
from urllib.parse import urlsplit

from airs_bundle import verify_bundle
from airs_review_release import require
from plan_airs_review_publication import (
    LAUNCHER,
    NATIVES,
    REGISTRY,
    archive_snapshot,
    plan_publication,
    read_json,
    regular_digest,
)
from publish_airs_review import Registry, save
from restore_airs_review_stage import restore_stage


def fetch_review(evidence, plan_sha256, output, registry):
    plan_path = evidence / "PUBLICATION-PLAN.json"
    require(regular_digest(plan_path)[0] == plan_sha256, "Approved plan hash mismatch")
    plan = read_json(plan_path)
    publication = read_json(evidence / "PUBLICATION.json")
    require(
        publication.get("complete") is True
        and publication.get("published") is True
        and publication.get("approved_plan_sha256") == plan_sha256
        and publication.get("version") == plan.get("version")
        and publication.get("source_commit") == plan.get("source_commit")
        and publication.get("registry") == REGISTRY
        and publication.get("dist_tag") == plan.get("dist_tag") == "auth-review",
        "A completed matching auth-review publication receipt is required",
    )
    records = plan.get("packages")
    require(
        isinstance(records, list)
        and len(records) == 3
        and {row.get("name") for row in records} == {LAUNCHER, *NATIVES},
        "Expected exactly the approved Linux, Apple Silicon and launcher packages",
    )
    archives = output / "registry-archives"
    archives.mkdir()
    fetched = []
    for record in records:
        name, version = record["name"], record["version"]
        require(version == plan["version"], "Package versions disagree")
        association = registry.association(name)
        require(
            association
            == {"repository": "cdot65/airs-harness", "visibility": "private"},
            "Package is not private and associated with the intended repository",
        )
        require(registry.tags(name).get("auth-review") == version, "Review tag differs")
        spec = name + "@" + version
        metadata = registry.view(spec)
        require(
            isinstance(metadata, dict)
            and metadata.get("integrity") == record["integrity"],
            "Published integrity differs from approved archive",
        )
        url = urlsplit(metadata.get("tarball", ""))
        require(
            url.scheme == "https"
            and url.netloc == "npm.pkg.github.com"
            and not url.query
            and not url.fragment,
            "Published archive is outside the GitHub registry origin",
        )
        archive = registry.download(spec, archives)
        require(
            archive.name == record["filename"]
            and regular_digest(archive) == (record["sha256"], record["integrity"]),
            "Actual registry archive differs from approved bytes",
        )
        fetched.append({**record, **association, "download_verified": True})
    stage = output / "verified-stage"
    restore_stage(archives, evidence / "NPM-PACKAGES.json", stage)
    require(
        plan_publication(stage, "auth-review") == plan, "Registry package plan differs"
    )
    save(output / "REGISTRY-DOWNLOAD.json", {"packages": fetched, "passed": True})
    return plan, stage


def verify_installed(stage, prefix, native_manifest):
    launcher = prefix / "lib/node_modules" / LAUNCHER
    native_manifest = Path(native_manifest).resolve(strict=True)
    require(
        native_manifest.is_relative_to(prefix), "Resolved native package escaped prefix"
    )
    native_directory = native_manifest.parent
    manifest = read_json(native_manifest)
    expected_name = {
        ("Linux", "x86_64"): "@cdot65/prisma-airs-harness-linux-x64",
        ("Darwin", "arm64"): "@cdot65/prisma-airs-harness-darwin-arm64",
    }.get((platform.system(), platform.machine()))
    require(
        expected_name is not None and manifest.get("name") == expected_name,
        "Unsupported host or mismatched platform payload",
    )
    receipt = read_json(stage / "NPM-PACKAGES.json")
    for record in receipt["publish_order"]:
        if record["name"] in {LAUNCHER, expected_name}:
            archive_snapshot(
                stage / "tarballs" / record["filename"],
                launcher if record["name"] == LAUNCHER else native_directory,
            )
    inventory = read_json(launcher / "BUNDLE-INVENTORY.json")
    bundle = verify_bundle(launcher, inventory)
    info = read_json(native_directory / "BUILD-INFO.json")
    native = native_directory / "bin/airs-harness"
    require(
        regular_digest(native)[0] == info["binary_sha256"],
        "Installed native hash differs",
    )
    command = prefix / "bin/airs-harness"
    require(
        command.resolve(strict=True) == launcher / "bin/airs-harness.js",
        "Installed npm command resolves outside the verified launcher",
    )
    return launcher, native, info, bundle


def validate(evidence, plan_sha256, output):
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    output = output.resolve(strict=True)
    require(
        (platform.system(), platform.machine())
        in {("Linux", "x86_64"), ("Darwin", "arm64")},
        "Only Linux x64 and Apple Silicon are accepted",
    )
    registry = Registry(output / "download-cache")
    plan, stage = fetch_review(evidence, plan_sha256, output, registry)
    prefix = output / "fresh-prefix"
    prefix.mkdir()
    environment = {
        key: value
        for key, value in os.environ.items()
        if not key.lower().startswith("npm_config_")
    }
    npmrc = output / "registry.npmrc"
    npmrc.write_text("//npm.pkg.github.com/:_authToken=${NODE_AUTH_TOKEN}\n")
    npmrc.chmod(0o600)
    environment.update(
        NPM_CONFIG_USERCONFIG=str(npmrc),
        NPM_CONFIG_GLOBALCONFIG=str(output / "empty.npmrc"),
        NPM_CONFIG_CACHE=str(output / "install-cache"),
        NPM_CONFIG_UPDATE_NOTIFIER="false",
    )
    (output / "empty.npmrc").write_text("")
    install = subprocess.run(
        [
            shutil.which("npm") or "npm",
            "install",
            "--global",
            "--prefix",
            str(prefix),
            "--ignore-scripts",
            "--include=optional",
            "--no-audit",
            "--no-fund",
            "--registry",
            REGISTRY,
            LAUNCHER + "@" + plan["version"],
        ],
        cwd=prefix,
        env=environment,
        capture_output=True,
        text=True,
        timeout=300,
    )
    # Never retain raw npm output: authentication errors can contain sensitive URLs.
    save(
        output / "INSTALL-ATTEMPT.json",
        {
            "exit_code": install.returncode,
            "fresh_cache": True,
            "installed_by_exact_scoped_name": True,
            "registry": REGISTRY,
            "version": plan["version"],
        },
    )
    require(install.returncode == 0, "Fresh GitHub npm installation failed")
    launcher = prefix / "lib/node_modules" / LAUNCHER
    result = subprocess.run(
        [
            "node",
            "--input-type=module",
            "-e",
            "import {createRequire} from 'node:module'; const r=createRequire(process.argv[1]); "
            "const m=r(process.argv[1]); const names=Object.keys(m.optionalDependencies); "
            "const suffix=process.platform==='darwin'?'-darwin-arm64':'-linux-x64'; "
            "console.log(r.resolve(names.find(n=>n.endsWith(suffix))+'/package.json'));",
            str(launcher / "package.json"),
        ],
        capture_output=True,
        text=True,
        timeout=15,
    )
    require(
        result.returncode == 0, "Installed platform dependency could not be resolved"
    )
    launcher, native, info, bundle = verify_installed(
        stage, prefix, result.stdout.strip()
    )
    if platform.system() == "Darwin":
        from airs_signed_macos_artifact import verify

        signing = read_json(native.parent.parent / "SIGNING.json")
        verify(
            native,
            info["binary_sha256"],
            output / "INSTALLED-SIGNING.json",
            info["source_commit"],
            signing["asset_id"],
            signing["archive_sha256"],
            signing["owner_reported_submission_id"],
        )
    command = prefix / "bin/airs-harness"
    versions = {}
    for label, arguments, expected in [
        ("harness", ["--version"], "airs-harness " + plan["version"]),
        ("managed_cli", ["airs", "--version"], "5.2.0"),
    ]:
        result = subprocess.run(
            [str(command), *arguments], capture_output=True, text=True, timeout=30
        )
        require(
            result.returncode == 0 and result.stdout.strip() == expected,
            "Installed command version differs",
        )
        versions[label] = result.stdout.strip()
    receipt = {
        "passed": True,
        "approved_plan_sha256": plan_sha256,
        "source_commit": info["source_commit"],
        "binary_sha256": info["binary_sha256"],
        "version": plan["version"],
        "versions": versions,
        "bundle_verification": bundle,
        "system": platform.system(),
        "machine": platform.machine(),
        "npm_version": subprocess.check_output(
            ["npm", "--version"], text=True, timeout=15
        ).strip(),
        "command": str(command),
        "launcher": str(launcher / "bin/airs-harness.js"),
        "registry": REGISTRY,
        "credential_principal": "GitHub Actions repository token",
        "teammate_read_only_install_verified": False,
        "owner_device": False,
        "live_api_tested": False,
        "full_authentication_release_ready": False,
    }
    save(output / "REGISTRY-INSTALL.json", receipt)
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--approved-plan-sha256", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    validate(args.evidence, args.approved_plan_sha256, args.output)


if __name__ == "__main__":
    main()
