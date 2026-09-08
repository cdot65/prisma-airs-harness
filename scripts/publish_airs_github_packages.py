#!/usr/bin/env python3
"""Publish GitHub-scoped copies of checksum-verified, already released npm archives.

Only package manifests change. Native dependencies use immutable GitHub tarball
URLs so public CLI dependencies can resolve from npmjs without a scope collision.
Requires an npm configuration authenticated to GitHub Packages; never reads tokens.
This legacy copy path only accepts previously published releases and review tags.
It does not authorize private candidate publication or production promotion.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile

REGISTRY = "https://npm.pkg.github.com"
NAMES = {"airs-harness", "airs-harness-linux-x64", "airs-harness-darwin-arm64"}


def npm(*args, cwd=None):
    return subprocess.check_output(["npm", *args], cwd=cwd, text=True).strip()


def require_released_source(document):
    """Reject explicit candidate/failure markers, including malformed booleans."""
    if not isinstance(document, dict):
        raise ValueError("Expected release metadata object")
    if (
        any(
            field in document and document[field] is not False
            for field in ("private", "private_candidate", "unsigned_candidate")
        )
        or any(
            field in document and document[field] is not True
            for field in ("publishable", "release_ready", "passed", "published")
        )
        or "release_status" in document
        or (
            "status" in document
            and document["status"] not in ("published", "validated", "complete")
        )
    ):
        raise ValueError(
            "Private or unvalidated sources cannot be copied to a registry"
        )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--archives", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--dist-tag",
        required=True,
        choices=("auth-review", "review", "preview"),
        help="Explicit review channel; this legacy publisher cannot promote latest",
    )
    args = parser.parse_args()
    receipt = json.loads(args.receipt.read_text())
    require_released_source(receipt)
    records = receipt["packages"]
    for record in records:
        require_released_source(record)
    if (
        receipt["complete"] is not True
        or receipt.get("published") is not True
        or len(records) != len(NAMES)
        or {r["name"] for r in records} != NAMES
    ):
        raise ValueError("Expected a complete Linux x64 / Apple Silicon publication")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    staged = []
    # Verify every source before performing any registry mutation.
    for record in records:
        archive = args.archives / record["filename"]
        with archive.open("rb") as stream:
            if hashlib.file_digest(stream, "sha256").hexdigest() != record["sha256"]:
                raise ValueError("Published source archive checksum mismatch")
        directory = output / record["name"]
        directory.mkdir()
        with tarfile.open(archive) as tar:
            tar.extractall(directory, filter="data")
        package = directory / "package"
        manifest = json.loads((package / "package.json").read_text())
        require_released_source(manifest)
        provenance = package / "BUILD-INFO.json"
        if provenance.exists():
            require_released_source(json.loads(provenance.read_text()))
        if record["name"] == "airs-harness" and manifest.get(
            "optionalDependencies"
        ) != {name: receipt["version"] for name in NAMES - {"airs-harness"}}:
            raise ValueError("Native dependency set differs from publication")
        if (manifest["name"], manifest["version"]) != (
            record["name"],
            receipt["version"],
        ):
            raise ValueError("Source archive identity mismatch")
        if (
            manifest["repository"]["url"]
            != "git+https://github.com/cdot65/airs-harness.git"
        ):
            raise ValueError("Unexpected repository association")
        manifest["name"] = "@cdot65/prisma-" + record["name"]
        manifest["repository"] = {
            "type": "git",
            "url": "https://github.com/cdot65/airs-harness.git",
        }
        manifest["publishConfig"] = {"registry": REGISTRY}
        staged.append((record, package, manifest))
    native_urls = {}
    published = []
    existing_specs = []
    for record, package, manifest in sorted(
        staged, key=lambda item: (item[0]["name"] == "airs-harness", item[0]["name"])
    ):
        if record["name"] == "airs-harness":
            if set(manifest["optionalDependencies"]) != set(native_urls):
                raise ValueError("Native dependency set differs from publication")
            manifest["optionalDependencies"] = native_urls
        spec = manifest["name"] + "@" + manifest["version"]
        # A retry may encounter an already published immutable version. Accept
        # it only if its integrity equals the exact staged artifact.
        existing = subprocess.run(
            ["npm", "view", spec, "dist", "--json", "--registry", REGISTRY],
            text=True,
            capture_output=True,
        )
        if existing.returncode and "E404" not in existing.stderr:
            raise RuntimeError("Cannot establish whether package version exists")
        if existing.returncode == 0:
            # Preserve an existing version's exact repository URL spelling.
            # npm accepts both https and git+https; neither changes repository
            # identity. Every other field/file is still checked by full SRI.
            prior = output / (record["name"] + "-prior")
            prior.mkdir()
            fetched = json.loads(
                npm(
                    "pack",
                    spec,
                    "--ignore-scripts",
                    "--json",
                    "--registry",
                    REGISTRY,
                    cwd=prior,
                )
            )
            fetched = (
                next(iter(fetched.values()))
                if isinstance(fetched, dict)
                else fetched[0]
            )
            with tarfile.open(prior / fetched["filename"]) as tar:
                old = json.load(tar.extractfile("package/package.json"))["repository"]
            if (
                old.get("type") != "git"
                or old.get("url", "").removeprefix("git+")
                != "https://github.com/cdot65/airs-harness.git"
            ):
                raise ValueError("Existing package declares another repository")
            manifest["repository"] = old
        (package / "package.json").write_text(json.dumps(manifest, indent=2) + "\n")
        packed = json.loads(npm("pack", "--ignore-scripts", "--json", cwd=package))
        packed = next(iter(packed.values())) if isinstance(packed, dict) else packed[0]
        if packed["size"] >= 256 * 1024 * 1024:
            raise ValueError("Package exceeds GitHub npm archive size limit")
        if existing.returncode:
            npm(
                "publish",
                packed["filename"],
                "--ignore-scripts",
                "--tag",
                args.dist_tag,
                "--registry",
                REGISTRY,
                cwd=package,
            )
        dist = json.loads(npm("view", spec, "dist", "--json", "--registry", REGISTRY))
        if dist["integrity"] != packed["integrity"]:
            raise ValueError("GitHub version differs from staged package")
        # npm's download verifies registry integrity; separately compare bytes.
        download = output / (record["name"] + "-download")
        download.mkdir()
        fetched = json.loads(
            npm(
                "pack",
                spec,
                "--ignore-scripts",
                "--json",
                "--registry",
                REGISTRY,
                cwd=download,
            )
        )
        fetched = (
            next(iter(fetched.values())) if isinstance(fetched, dict) else fetched[0]
        )
        if (download / fetched["filename"]).read_bytes() != (
            package / packed["filename"]
        ).read_bytes():
            raise ValueError("Published download differs from staged archive")
        if record["name"] != "airs-harness":
            native_urls[record["name"]] = dist["tarball"]
        else:
            (output / "launcher-url.txt").write_text(dist["tarball"] + "\n")
        association = json.loads(
            subprocess.check_output(
                [
                    "gh",
                    "api",
                    "users/cdot65/packages/npm/" + manifest["name"].split("/")[1],
                ],
                text=True,
            )
        )
        linked = (association.get("repository") or {}).get("full_name")
        if linked and linked != "cdot65/airs-harness":
            raise ValueError("Package is linked to an unexpected repository")
        if existing.returncode == 0:
            existing_specs.append(spec)
        published.append(
            {
                "name": manifest["name"],
                "version": manifest["version"],
                "integrity": dist["integrity"],
                "tarball": dist["tarball"],
                "source_sha256": record["sha256"],
                "download_verified": True,
                "html_url": association["html_url"],
                "visibility": association["visibility"],
                "repository": linked,
                "declared_repository": manifest["repository"]["url"],
            }
        )
    # Existing immutable versions are tagged only after every byte check passes.
    for spec in existing_specs:
        npm("dist-tag", "add", spec, args.dist_tag, "--registry", REGISTRY)
    for record in published:
        tags = json.loads(
            npm("view", record["name"], "dist-tags", "--json", "--registry", REGISTRY)
        )
        if tags.get(args.dist_tag) != record["version"]:
            raise ValueError("Registry review tag does not select the verified version")
    (output / "PUBLICATION.json").write_text(
        json.dumps(
            {"registry": REGISTRY, "dist_tag": args.dist_tag, "packages": published},
            indent=2,
        )
        + "\n"
    )
    print(json.dumps(published, indent=2))


if __name__ == "__main__":
    main()
