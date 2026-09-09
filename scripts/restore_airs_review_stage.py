#!/usr/bin/env python3
"""Restore exact review archives for planning, without execution or repacking."""

import argparse
import json
from pathlib import Path
import shutil
import tarfile
import unicodedata

from airs_bundle_archive import safe_path
from airs_review_release import require
from plan_airs_review_publication import (
    LAUNCHER,
    NATIVES,
    LIMIT,
    no_candidate,
    read_json,
    regular_digest,
)


def extract(archive, directory):
    directory.mkdir(parents=True, mode=0o700, exist_ok=False)
    entries, aliases, total = set(), {}, 0
    with tarfile.open(archive, mode="r|gz") as source:
        for item in source:
            parts = safe_path(item.name.rstrip("/")).parts
            require(
                parts[0] == "package" and len(parts) > 1, "Unexpected npm archive root"
            )
            name = "/".join(parts[1:])
            require(
                name not in entries and len(entries) < 20_000,
                "Duplicate or excessive npm archive members",
            )
            entries.add(name)
            require(
                item.isfile() or item.isdir(),
                "Review archives cannot contain links or devices",
            )
            require(
                not item.issparse(),
                "Sparse archive members are not supported",
            )
            total += item.size
            require(
                0 <= item.size <= LIMIT
                and total <= LIMIT
                and (not item.isdir() or item.size == 0),
                "Review archive exceeds expanded size bounds",
            )
            relative_parts = parts[1:]
            for count in range(1, len(relative_parts) + 1):
                prefix = "/".join(relative_parts[:count])
                alias = unicodedata.normalize("NFC", prefix).casefold()
                kind = (
                    "directory"
                    if count < len(relative_parts) or item.isdir()
                    else "file"
                )
                previous = aliases.get(alias)
                require(
                    previous is None
                    or previous == (prefix, "directory")
                    and kind == "directory",
                    "Archive has conflicting or aliased paths",
                )
                aliases[alias] = (prefix, kind)
            destination = directory.joinpath(*relative_parts)
            if item.isdir():
                destination.mkdir(parents=True, mode=0o755, exist_ok=True)
                continue
            destination.parent.mkdir(parents=True, mode=0o755, exist_ok=True)
            written = 0
            with source.extractfile(item) as stream, destination.open("xb") as output:
                while chunk := stream.read(65536):
                    written += len(chunk)
                    require(
                        written <= item.size,
                        "Archive member exceeded its declared size",
                    )
                    output.write(chunk)
            require(written == item.size, "Truncated archive member")
            destination.chmod(item.mode & 0o777)
    require("package.json" in entries, "Review archive has no manifest")


def restore_stage(archives, receipt_path, directory):
    archives, receipt_path, directory = (
        Path(archives),
        Path(receipt_path),
        Path(directory),
    )
    receipt = read_json(receipt_path)
    no_candidate(receipt)
    require(receipt.get("published") is False, "Expected unpublished staging receipt")
    records = receipt.get("publish_order")
    require(
        isinstance(records, list)
        and len(records) == 3
        and {record.get("name") for record in records} == {LAUNCHER, *NATIVES},
        "Exactly three scoped review packages are required",
    )
    filenames = set()
    for record in records:
        no_candidate(record)
        filename = record.get("filename")
        require(
            isinstance(filename, str)
            and len(safe_path(filename).parts) == 1
            and filename not in filenames,
            "Invalid or duplicate archive filename",
        )
        filenames.add(filename)
        require(
            regular_digest(archives / filename, limit=256 * 1024 * 1024 - 1)
            == (record.get("sha256"), record.get("integrity")),
            "Downloaded review archive identity differs from receipt",
        )
    require(
        {path.name for path in archives.glob("*.tgz")} == filenames,
        "Download directory contains an unexpected npm archive set",
    )
    directory.mkdir(mode=0o700, parents=True, exist_ok=False)
    tarballs = directory / "tarballs"
    tarballs.mkdir()
    # Keep the exact downloaded archive bytes for the later approved-plan gate.
    for record in records:
        archive = tarballs / record["filename"]
        shutil.copyfile(archives / record["filename"], archive)
        require(
            regular_digest(archive) == (record["sha256"], record["integrity"]),
            "Archive changed during restoration",
        )
        extract(archive, directory / record["name"])
        manifest = read_json(directory / record["name"] / "package.json")
        no_candidate(manifest)
        require(
            manifest.get("name") == record["name"]
            and manifest.get("version") == record["version"],
            "Restored manifest identity differs from the staging receipt",
        )
    # Parse/serialize only the receipt; archive and package payload bytes are unchanged.
    (directory / "NPM-PACKAGES.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return {
        "restored_packages": len(records),
        "executed": False,
        "repacked": False,
        "publication_plan_still_required": True,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archives", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument("--directory", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(restore_stage(args.archives, args.receipt, args.directory)))


if __name__ == "__main__":
    main()
