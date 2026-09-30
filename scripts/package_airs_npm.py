#!/usr/bin/env python3
"""Prepare airs-harness npm tarballs from verified native release directories.

This command never publishes. Supply one --release-directory per supported target;
only supplied targets become exact-version optional dependencies of the launcher.
"""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from urllib.parse import urlparse


TARGETS = {
    "x86_64-unknown-linux-musl": ("linux", "x64"),
    "aarch64-unknown-linux-musl": ("linux", "arm64"),
    "aarch64-apple-darwin": ("darwin", "arm64"),
    "x86_64-pc-windows-msvc": ("win32", "x64"),
    "aarch64-pc-windows-msvc": ("win32", "arm64"),
}


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--release-directory", type=Path, action="append", required=True
    )
    parser.add_argument("--output-directory", type=Path, required=True)
    parser.add_argument(
        "--registry", required=True, help="Destination registry HTTPS URL"
    )
    parser.add_argument(
        "--scoped",
        action="store_true",
        help="Stage @cdot65/prisma-airs-harness packages with exact scoped native dependencies",
    )
    parser.add_argument(
        "--bundle-cli",
        action="store_true",
        help="Bundle the exact locked Prisma AIRS CLI/SDK dependency tree (no publication)",
    )
    parser.add_argument(
        "--mac-preview",
        action="store_true",
        help="Restrict a Mac-first launcher to Apple Silicon",
    )
    args = parser.parse_args()
    registry = urlparse(args.registry)
    if (
        registry.scheme != "https"
        or not registry.hostname
        or registry.username
        or registry.password
        or registry.query
        or registry.fragment
    ):
        raise ValueError(
            "Use an HTTPS registry URL without credentials, query or fragment"
        )
    root = Path(__file__).resolve().parents[1]
    template = root / "npm/airs-harness"
    manifest = json.loads((template / "package.json").read_text())
    if args.scoped:
        manifest["name"] = "@cdot65/prisma-airs-harness"
    output = args.output_directory.resolve()
    output.mkdir(parents=True, exist_ok=False)
    packages = []
    dependencies = {}
    # Scoped packages are restricted by default on the public registry.
    publish_config = {"registry": args.registry}
    if args.scoped:
        publish_config["access"] = "public"
    source_commit = None
    releases = [
        (
            release.resolve(strict=True),
            json.loads((release / "BUILD-INFO.json").read_text()),
        )
        for release in args.release_directory
    ]
    if args.mac_preview:
        if len(releases) != 1 or releases[0][1]["target"] != "aarch64-apple-darwin":
            raise ValueError("Mac preview requires exactly one Apple Silicon release")
        manifest.update(os=["darwin"], cpu=["arm64"])
    # A mixed candidate/release set must not publish even an earlier native
    # package before discovering that a later package is private.
    candidate = any(
        info.get("publishable") is False or "release_status" in info
        for _, info in releases
    )
    for release, info in releases:
        if (
            info["version"] != manifest["version"]
            or info["product"] != "Prisma AIRS Harness"
        ):
            raise ValueError(
                "Native release identity/version does not match the npm launcher"
            )
        if source_commit is not None and info["source_commit"] != source_commit:
            raise ValueError(
                "All native packages must come from the same source commit"
            )
        source_commit = info["source_commit"]
        platform, arch = TARGETS[info["target"]]
        name = f"airs-harness-{platform}-{arch}"
        if args.scoped:
            name = "@cdot65/prisma-" + name
        if name in dependencies:
            raise ValueError("Duplicate native target")
        dependencies[name] = manifest["version"]
        binary_name = "airs-harness.exe" if platform == "win32" else "airs-harness"
        if digest(release / binary_name) != info["binary_sha256"]:
            raise ValueError("Native binary differs from its build provenance")
        package = output / name
        (package / "bin").mkdir(parents=True)
        shutil.copy2(release / binary_name, package / "bin" / binary_name)
        for filename in ["LICENSE", "NOTICE", "DEPENDENCIES.json", "BUILD-INFO.json"]:
            shutil.copy2(release / filename, package / filename)
        if "signing_receipt_sha256" in info:
            if digest(release / "SIGNING.json") != info["signing_receipt_sha256"]:
                raise ValueError("Signing receipt differs from native provenance")
            shutil.copy2(release / "SIGNING.json", package / "SIGNING.json")
        if "validation_receipt_sha256" in info:
            if digest(release / "VALIDATION.json") != info["validation_receipt_sha256"]:
                raise ValueError("Validation receipt differs from native provenance")
            shutil.copy2(release / "VALIDATION.json", package / "VALIDATION.json")
            shutil.copytree(
                release / "validation-evidence", package / "validation-evidence"
            )
        shutil.copytree(release / "licenses", package / "licenses")
        native_manifest = {
            "name": name,
            "version": manifest["version"],
            "description": f"Prisma AIRS Harness native executable for {platform}/{arch}",
            "license": manifest["license"],
            "repository": {"type": "git", "url": manifest["repository"]["url"]},
            "os": [platform],
            "cpu": [arch],
            "files": [
                "bin/",
                "licenses/",
                "LICENSE",
                "NOTICE",
                "BUILD-INFO.json",
                "DEPENDENCIES.json",
                "SIGNING.json",
                "VALIDATION.json",
                "validation-evidence/",
            ],
            "publishConfig": publish_config,
        }
        if candidate:
            native_manifest["private"] = True
        (package / "package.json").write_text(
            json.dumps(native_manifest, indent=2) + "\n"
        )
        packages.append(package)
    launcher = output / manifest["name"]
    launcher.mkdir(parents=True)
    for name in ["bin", "lib", "managed-cli"]:
        shutil.copytree(template / name, launcher / name)
    for name in ["LICENSE", "NOTICE", "README.md"]:
        shutil.copy2(template / name, launcher / name)
    for name in [
        "MACOS.md",
        "PRISMA-AIRS-CLI.md",
        "GETTING-STARTED.md",
        "UBUNTU-TEST-HOST.md",
    ]:
        shutil.copy2(root / name, launcher / name)
    (launcher / "scripts").mkdir()
    shutil.copy2(
        root / "scripts/prepare_airs_ubuntu.sh",
        launcher / "scripts/prepare_airs_ubuntu.sh",
    )
    manifest.pop("private", None)
    if candidate:
        manifest["private"] = True
    manifest["optionalDependencies"] = dependencies
    manifest["publishConfig"] = publish_config
    (launcher / "package.json").write_text(json.dumps(manifest, indent=2) + "\n")
    cli_bundle = None
    bundle_sources = []
    if args.bundle_cli:
        from airs_bundle import bundle_cli, verify_bundle

        inventory = bundle_cli(
            template / "package-lock.json",
            launcher,
            [info["target"] for _, info in releases],
        )
        manifest["bundleDependencies"] = ["@cdot65/prisma-airs-cli"]
        manifest["files"].append("BUNDLE-INVENTORY.json")
        cli_bundle = {
            "inventory_sha256": digest(launcher / "BUNDLE-INVENTORY.json"),
            "targets": inventory["targets"],
            "required_pins": inventory["required_pins"],
            **verify_bundle(launcher, inventory),
        }
        bundle_sources = [
            template / "package-lock.json",
            *(
                root / "scripts" / name
                for name in (
                    "airs_bundle.py",
                    "airs_bundle_archive.py",
                    "airs_bundle_tree.py",
                    "airs_bundle_shims.py",
                )
            ),
            *sorted((root / "scripts/fixtures/npm-command-shims").iterdir()),
        ]
    tooling = {
        "packaging_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=root, text=True
        ).strip(),
        "native_source_commit": source_commit,
        "files": {
            str(path.relative_to(root)): digest(path)
            for path in [
                Path(__file__).resolve(),
                *bundle_sources,
                template / "package.json",
                *sorted((template / "lib").glob("*.js")),
                *sorted((template / "bin").glob("*.js")),
                *sorted((template / "managed-cli").iterdir()),
            ]
            if path.is_file()
        },
    }
    if cli_bundle is not None:
        tooling["cli_bundle"] = cli_bundle
    (launcher / "PACKAGE-TOOLING.json").write_text(json.dumps(tooling, indent=2) + "\n")
    manifest["files"].append("PACKAGE-TOOLING.json")
    (launcher / "package.json").write_text(json.dumps(manifest, indent=2) + "\n")
    packages.append(launcher)
    tarballs = output / "tarballs"
    tarballs.mkdir()
    receipts = []
    for package in packages:
        result = subprocess.check_output(
            [
                shutil.which("npm") or "npm",
                "pack",
                "--ignore-scripts",
                "--json",
                "--pack-destination",
                str(tarballs),
            ],
            cwd=package,
            text=True,
        )
        packed = json.loads(result)
        # npm 12 keys JSON output by package name; earlier npm emits an array.
        records = list(packed.values()) if isinstance(packed, dict) else packed
        if (
            not isinstance(records, list)
            or len(records) != 1
            or not isinstance(records[0], dict)
            or records[0].get("name")
            != json.loads((package / "package.json").read_text())["name"]
        ):
            raise ValueError("npm pack did not return the expected single package")
        record = records[0]
        if (
            args.bundle_cli
            and package == launcher
            and (tarballs / record["filename"]).stat().st_size > 64 * 1024 * 1024
        ):
            raise ValueError("Bundled launcher archive exceeds 64 MiB limit")
        receipts.append(
            {
                "name": record["name"],
                "version": record["version"],
                "filename": record["filename"],
                "integrity": record["integrity"],
                "sha256": digest(tarballs / record["filename"]),
            }
        )
    receipt = {
        "published": False,
        "source_commit": source_commit,
        "package_tooling": tooling,
        "packaging_commit": tooling["packaging_commit"],
        "required_dependencies": manifest["dependencies"],
        "registry": args.registry,
        "publish_order": receipts,
    }
    if cli_bundle is not None:
        receipt["cli_bundle"] = cli_bundle
    if candidate:
        receipt["release_status"] = "unvalidated-candidate"
        receipt["publishable"] = False
    (output / "NPM-PACKAGES.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
