#!/usr/bin/env python3
"""Install the bundled AIRS review candidate into a new, separate directory."""

import argparse
import hashlib
import json
import platform
import shutil
import subprocess
import sys
from pathlib import Path, PurePosixPath


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prefix", type=Path, required=True)
    args = parser.parse_args()
    if sys.version_info < (3, 11):
        parser.error("Python 3.11 or newer is required")
    if (platform.system(), platform.machine()) not in {
        ("Linux", "x86_64"),
        ("Linux", "aarch64"),
        ("Linux", "arm64"),
        ("Darwin", "arm64"),
    }:
        parser.error("This candidate supports Linux x64/ARM64 and Apple Silicon")
    if not shutil.which("node") or not shutil.which("npm"):
        parser.error("Install Node.js 22.14+ (22.x) or Node.js 24+ with npm first")
    prefix = args.prefix.expanduser().resolve()
    if prefix.exists():
        parser.error("Choose a new directory for --prefix")
    root = Path(__file__).resolve().parent
    manifest = json.loads((root / "REVIEW-CONTENTS.json").read_text())
    for relative, expected in manifest["files"].items():
        path = PurePosixPath(relative)
        if path.is_absolute() or ".." in path.parts or "\\" in relative:
            raise ValueError("Invalid review manifest path")
        source = root / path
        if source.is_symlink() or root not in source.resolve().parents:
            raise ValueError("Review contents must stay inside the extracted bundle")
        with source.open("rb") as stream:
            actual = hashlib.file_digest(stream, "sha256").hexdigest()
        if actual != expected:
            raise ValueError(
                f"Review contents failed integrity verification: {relative}"
            )
    print(
        "Installing the bundled AIRS candidate and checking its integrity…", flush=True
    )
    result = subprocess.run(
        [
            sys.executable,
            str(root / "tools/validate_airs_npm.py"),
            "--packages",
            str(root / "packages"),
            "--prefix",
            str(prefix),
        ],
        capture_output=True,
        text=True,
        timeout=420,
        check=False,
    )
    log = prefix / "review-install.log"
    if prefix.is_dir():
        log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise SystemExit(f"Installation failed; inspect {log}")
    receipt = json.loads((prefix / "INSTALL-VERIFICATION.json").read_text())
    print(
        f"Installed {receipt['version']}; bundled CLI {receipt['prisma_airs_cli_version']}"
    )
    print(f"Command: {prefix / 'bin/airs'}")
    print("Use the README's separate review home to try first-run onboarding.")


if __name__ == "__main__":
    main()
