#!/usr/bin/env python3
"""Validate exact scoped bundled review archives and emit a publication plan.

No registry lookup, mutation, lifecycle script or native command is executed.
A publisher must revalidate these bytes, preflight immutable versions, publish
native packages before the launcher, and verify downloaded bytes/tag/access.
"""

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import stat
import tarfile

from airs_bundle import verify_bundle
from airs_bundle_archive import safe_path
from airs_review_release import SCOPE, validate_release, require

REGISTRY = "https://npm.pkg.github.com"
LAUNCHER = "@cdot65/prisma-airs-harness"
NATIVES = {
    LAUNCHER + "-linux-x64": ("x86_64-unknown-linux-musl", "linux", "x64"),
    LAUNCHER + "-darwin-arm64": ("aarch64-apple-darwin", "darwin", "arm64"),
}
LIMIT = 512 * 1024 * 1024


def regular_digest(path, *, limit=LIMIT):
    metadata = path.lstat()
    require(
        stat.S_ISREG(metadata.st_mode)
        and not getattr(metadata, "st_file_attributes", 0) & 0x400,
        "Publication input must be a regular non-reparse file",
    )
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0)
    with os.fdopen(os.open(path, flags), "rb") as stream:
        opened = os.fstat(stream.fileno())
        require(
            stat.S_ISREG(opened.st_mode)
            and opened.st_size <= limit
            and (metadata.st_dev, metadata.st_ino) == (opened.st_dev, opened.st_ino),
            "Publication input changed or exceeds its bound",
        )
        sha256, sha512 = hashlib.sha256(), hashlib.sha512()
        count = 0
        while chunk := stream.read(65536):
            count += len(chunk)
            require(count <= limit, "Publication input exceeded its read bound")
            sha256.update(chunk)
            sha512.update(chunk)
    return sha256.hexdigest(), "sha512-" + base64.b64encode(sha512.digest()).decode()


def read_json(path):
    regular_digest(path, limit=4 * 1024 * 1024)
    return json.loads(path.read_bytes())


def no_candidate(document):
    require(isinstance(document, dict), "Expected publication metadata object")
    require(
        "release_status" not in document
        and all(
            key not in document or document[key] is False
            for key in ("private", "private_candidate", "unsigned_candidate")
        )
        and ("publishable" not in document or document["publishable"] is True),
        "Private or unvalidated candidates cannot enter the review publisher",
    )


def archive_snapshot(archive, directory):
    """Inspect regular members only; bind every archived byte to its staged source."""
    observed, total = {}, 0
    with tarfile.open(archive, mode="r|gz") as stream:
        for member in stream:
            parts = safe_path(member.name.rstrip("/")).parts
            require(
                parts[0] == "package" and len(parts) > 1, "Unexpected npm archive root"
            )
            name = "/".join(parts[1:])
            require(
                name not in observed and len(observed) < 20_000,
                "Duplicate or excessive npm archive members",
            )
            require(
                member.isfile() or member.isdir(),
                "Npm archives cannot contain links or devices",
            )
            if member.isdir():
                continue
            total += member.size
            require(
                0 <= member.size <= LIMIT and total <= LIMIT,
                "Npm archive exceeds its expanded payload limit",
            )
            source = directory
            for part in parts[1:]:
                source /= part
                metadata = source.lstat()
                require(
                    not stat.S_ISLNK(metadata.st_mode)
                    and not getattr(metadata, "st_file_attributes", 0) & 0x400,
                    "Staged npm payload cannot traverse links",
                )
            expected, _ = regular_digest(source)
            digest = hashlib.sha256()
            with stream.extractfile(member) as payload:
                while chunk := payload.read(65536):
                    digest.update(chunk)
            require(
                digest.hexdigest() == expected,
                "Archived bytes differ from staged payload",
            )
            observed[name] = expected
    require("package.json" in observed, "Npm archive has no manifest")
    return observed


def plan_publication(packages, dist_tag):
    require(
        dist_tag in {"auth-review", "review", "preview"},
        "Only explicit review tags are supported",
    )
    packages = Path(packages).resolve(strict=True)
    receipt = read_json(packages / "NPM-PACKAGES.json")
    no_candidate(receipt)
    require(
        receipt.get("published") is False and receipt.get("registry") == REGISTRY,
        "Expected unpublished GitHub-scoped staging receipt",
    )
    records = receipt.get("publish_order")
    require(
        isinstance(records, list)
        and len(records) == 3
        and {record.get("name") for record in records} == {LAUNCHER, *NATIVES},
        "Exactly Linux x64, Apple Silicon and the scoped launcher are required",
    )
    versions = {record.get("version") for record in records}
    require(len(versions) == 1, "All staged package versions must match")
    version = versions.pop()
    summaries, planned, snapshots = {}, [], {}
    for record in records:
        no_candidate(record)
        name = record["name"]
        directory = packages / name
        require(
            directory.is_dir() and not directory.is_symlink(),
            "Missing regular staged package directory",
        )
        filename = record.get("filename")
        require(
            isinstance(filename, str) and len(safe_path(filename).parts) == 1,
            "Archive filename must be a basename",
        )
        archive = packages / "tarballs" / filename
        sha256, integrity = regular_digest(archive, limit=256 * 1024 * 1024 - 1)
        require(
            sha256 == record.get("sha256") and integrity == record.get("integrity"),
            "Staged archive SHA256 or SRI differs from receipt",
        )
        observed = archive_snapshot(archive, directory)
        snapshots[name] = observed
        manifest = read_json(directory / "package.json")
        no_candidate(manifest)
        require(
            manifest.get("name") == name
            and manifest.get("version") == version
            and manifest.get("publishConfig") == {"registry": REGISTRY}
            and manifest.get("repository", {}).get("url", "").removeprefix("git+")
            == "https://github.com/cdot65/airs-harness.git",
            "Staged npm identity, registry or repository differs",
        )
        if name in NATIVES:
            target, platform, arch = NATIVES[name]
            require(
                manifest.get("os") == [platform] and manifest.get("cpu") == [arch],
                "Native package platform metadata differs",
            )
            info = read_json(directory / "BUILD-INFO.json")
            no_candidate(info)
            require(
                info.get("product") == "Prisma AIRS Harness"
                and info.get("version") == version
                and info.get("target") == target
                and info.get("source_commit") == receipt.get("source_commit")
                and info.get("release_scope") == SCOPE,
                "Native release provenance differs from the review scope",
            )
            validation = read_json(directory / "VALIDATION.json")
            require(
                observed.get("VALIDATION.json") == info.get("validation_receipt_sha256")
                and observed.get("bin/airs-harness") == info.get("binary_sha256"),
                "Archived validation or executable differs from provenance",
            )
            summaries[target] = validate_release(
                validation,
                binary_sha256=info["binary_sha256"],
                target=target,
                version=version,
                source_commit=receipt["source_commit"],
                evidence_root=directory / "validation-evidence",
            )
            if target == "aarch64-apple-darwin":
                signing = read_json(directory / "SIGNING.json")
                declared = validation["signing"]
                require(
                    observed.get("SIGNING.json") == info.get("signing_receipt_sha256")
                    and observed.get("SIGNING.json") is not None
                    and signing.get("binary_sha256") == info["binary_sha256"]
                    and signing.get("source_commit") == receipt["source_commit"]
                    and signing.get("target") == target
                    and signing.get("team_id") == declared["team_identifier"]
                    and signing.get("codesign_verified") is True
                    and signing.get("hardened_runtime") is True
                    and signing.get("notarization_verified") is True
                    and signing.get("archive_sha256")
                    == declared["notarization"]["archive_sha256"]
                    and signing.get("owner_reported_submission_id")
                    == declared["notarization"]["submission_id"],
                    "Archived native signing verification differs from release attestation",
                )
            for evidence in validation["evidence"]:
                require(
                    observed.get("validation-evidence/" + evidence["path"])
                    == evidence["sha256"],
                    "Required validation evidence is absent from native archive",
                )
        planned.append(
            {
                "name": name,
                "version": version,
                "filename": filename,
                "sha256": sha256,
                "integrity": integrity,
            }
        )
    launcher = packages / LAUNCHER
    manifest = read_json(launcher / "package.json")
    bundle = receipt.get("cli_bundle")
    require(
        isinstance(bundle, dict)
        and manifest.get("optionalDependencies") == {name: version for name in NATIVES}
        and manifest.get("dependencies") == {"@cdot65/prisma-airs-cli": "5.2.0"}
        and set(manifest.get("bundleDependencies", [])) == {"@cdot65/prisma-airs-cli"},
        "Launcher must use exact scoped native pins and the bundled managed CLI",
    )
    require(
        bundle.get("required_pins")
        == {"@cdot65/prisma-airs-cli": "5.2.0", "@cdot65/prisma-airs-sdk": "0.28.0"}
        and set(bundle.get("targets", [])) == {value[0] for value in NATIVES.values()}
        and snapshots[LAUNCHER].get("BUNDLE-INVENTORY.json")
        == bundle.get("inventory_sha256"),
        "Bundle target, pin or inventory identity differs",
    )
    require(
        manifest.get("bin") == {"airs-harness": "bin/airs-harness.js"}
        and manifest.get("type") == "module"
        and not manifest.get("scripts"),
        "Launcher entrypoint or lifecycle scripts differ",
    )
    tooling = read_json(launcher / "PACKAGE-TOOLING.json")
    require(
        tooling == receipt.get("package_tooling")
        and tooling.get("cli_bundle") == bundle
        and tooling.get("native_source_commit") == receipt["source_commit"],
        "Archived package tooling differs from staging provenance",
    )
    required_launcher_sources = {
        "bin/airs-harness.js",
        "lib/child.js",
        "lib/launcher.js",
        "lib/prisma-cli.js",
        "managed-cli/airs",
        "managed-cli/airs.cmd",
        "managed-cli/empty.env",
    }
    for name in required_launcher_sources:
        require(
            snapshots[LAUNCHER].get(name)
            == tooling.get("files", {}).get("npm/airs-harness/" + name)
            and name in snapshots[LAUNCHER],
            "Launcher code differs from recorded packaging source",
        )
    inventory = read_json(launcher / "BUNDLE-INVENTORY.json")
    bundle_result = verify_bundle(launcher, inventory)
    archived_bundle_files = set()
    for package in inventory["packages"]:
        for item in package["files"]:
            require(
                package["path"] + "/" + item in snapshots[LAUNCHER],
                "Required bundled dependency file missing from archive",
            )
        for item in {**package["files"], **package.get("optional_files", {})}:
            path = package["path"] + "/" + item
            if path in snapshots[LAUNCHER]:
                archived_bundle_files.add(path)
    # npm can omit declared optional changelogs/lockfiles. Keep strict staging
    # verification above, but bind the plan count to the exact archived payload.
    bundle_result = {**bundle_result, "files": len(archived_bundle_files)}
    return {
        "schema_version": 1,
        "scope": SCOPE,
        "passed": True,
        "published": False,
        "registry_preflight_required": True,
        "registry": REGISTRY,
        "dist_tag": dist_tag,
        "version": version,
        "source_commit": receipt["source_commit"],
        "packages": sorted(
            planned, key=lambda row: (row["name"] == LAUNCHER, row["name"])
        ),
        "native_validation": summaries,
        "bundle_verification": bundle_result,
        "full_authentication_release_ready": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument(
        "--dist-tag", required=True, choices=["auth-review", "review", "preview"]
    )
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    plan = plan_publication(args.packages, args.dist_tag)
    with args.output.open("x") as output:
        json.dump(plan, output, indent=2)
        output.write("\n")


if __name__ == "__main__":
    main()
