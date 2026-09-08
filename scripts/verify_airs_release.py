#!/usr/bin/env python3
"""Verify an extracted AIRS release, including its binary provenance and notices."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    roots = [
        p
        for p in args.directory.glob("airs-harness-*")
        if (p / "BUILD-INFO.json").is_file()
    ]
    if len(roots) != 1:
        raise ValueError("Expected exactly one extracted release directory")
    root = roots[0].resolve()
    for row in (root / "SHA256SUMS").read_text().splitlines():
        expected, name = row.split("  ", 1)
        path = (root / name).resolve(strict=True)
        if not path.is_relative_to(root):
            raise ValueError("Checksum path escapes the release directory")
        with path.open("rb") as stream:
            if hashlib.file_digest(stream, "sha256").hexdigest() != expected:
                raise ValueError(f"Checksum mismatch: {name}")
    info = json.loads((root / "BUILD-INFO.json").read_text())
    with (root / "airs-harness").open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    if digest != info["binary_sha256"]:
        raise ValueError("Binary differs from build provenance")
    version = subprocess.check_output(
        [str(root / "airs-harness"), "--version"], text=True
    ).strip()
    if version != "airs-harness " + info["version"]:
        raise ValueError("Executable version differs from provenance")
    required = [
        "LICENSE",
        "NOTICE",
        "DEPENDENCIES.json",
        "VALIDATION.json",
        "licenses/rust-toolchain/COPYRIGHT-library.html",
    ]
    if not all((root / name).is_file() for name in required):
        raise ValueError("Missing release evidence or license notices")
    args.receipt.write_text(
        json.dumps(
            {
                "passed": True,
                "version": version,
                "binary_sha256": digest,
                "source_commit": info["source_commit"],
                "checksums_verified": True,
                "required_notices_present": True,
                "scope": "extracted package integrity; executable fixtures are a separate check",
            },
            indent=2,
        )
        + "\n"
    )
    print("Release integrity and provenance verified:", version)


if __name__ == "__main__":
    main()
