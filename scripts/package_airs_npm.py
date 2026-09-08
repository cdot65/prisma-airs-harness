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
        "--registry", required=True, help="Destination Verdaccio HTTPS URL"
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
    output = args.output_directory.resolve()
    output.mkdir(parents=True, exist_ok=False)
    packages = []
    dependencies = {}
    source_commit = None
    for release in args.release_directory:
        release = release.resolve(strict=True)
        info = json.loads((release / "BUILD-INFO.json").read_text())
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
            ],
            "publishConfig": {"registry": args.registry},
        }
        (package / "package.json").write_text(
            json.dumps(native_manifest, indent=2) + "\n"
        )
        packages.append(package)
    launcher = output / "airs-harness"
    launcher.mkdir()
    for name in ["bin", "lib", "managed-cli"]:
        shutil.copytree(template / name, launcher / name)
    for name in ["LICENSE", "NOTICE", "README.md"]:
        shutil.copy2(template / name, launcher / name)
    for name in ["MACOS.md", "PRISMA-AIRS-CLI.md"]:
        shutil.copy2(root / name, launcher / name)
    manifest.pop("private", None)
    manifest["optionalDependencies"] = dependencies
    manifest["publishConfig"] = {"registry": args.registry}
    (launcher / "package.json").write_text(json.dumps(manifest, indent=2) + "\n")
    packages.append(launcher)
    tarballs = output / "tarballs"
    tarballs.mkdir()
    receipts = []
    for package in packages:
        result = subprocess.check_output(
            [
                "npm",
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
            or records[0].get("name") != package.name
        ):
            raise ValueError("npm pack did not return the expected single package")
        record = records[0]
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
        "registry": args.registry,
        "publish_order": receipts,
    }
    (output / "NPM-PACKAGES.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
