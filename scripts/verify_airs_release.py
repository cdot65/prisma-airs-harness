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
    roots = list(args.directory.glob("airs-terminal-*-linux-x86_64-musl"))
    if len(roots) != 1:
        raise ValueError("Expected exactly one extracted release directory")
    root = roots[0].resolve()
    subprocess.run(
        ["sha256sum", "-c", "SHA256SUMS"],
        cwd=root,
        check=True,
        stdout=subprocess.DEVNULL,
    )
    info = json.loads((root / "BUILD-INFO.json").read_text())
    with (root / "airs-terminal").open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    if digest != info["binary_sha256"]:
        raise ValueError("Binary differs from build provenance")
    version = subprocess.check_output(
        [str(root / "airs-terminal"), "--version"], text=True
    ).strip()
    if version != "airs-terminal " + info["version"]:
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
