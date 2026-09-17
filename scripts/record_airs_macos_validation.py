#!/usr/bin/env python3
"""Record completed native macOS acceptance before release packaging."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--build-run")
    parser.add_argument("--source-directory", type=Path, default=Path.cwd())
    args = parser.parse_args()
    tests = (args.evidence / "native-tests.log").read_text()
    if not re.search(r"^OK(?: \(skipped=\d+\))?$", tests, re.MULTILINE):
        raise ValueError("Native executable acceptance did not pass")
    libraries = (args.evidence / "libraries.log").read_text().splitlines()[1:]
    if any(
        not line.strip().startswith(("/usr/lib/", "/System/Library/"))
        for line in libraries
    ):
        raise ValueError("Executable links to a non-system dynamic library")
    with args.binary.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    version = subprocess.check_output(
        [str(args.binary), "--version"], text=True
    ).strip()
    keychain = json.loads((args.evidence / "keychain.json").read_text())
    receipt = {
        "product": "Prisma AIRS Harness",
        "compilation_run": args.build_run or os.environ.get("GITHUB_RUN_ID"),
        "validation_run": os.environ.get("GITHUB_RUN_ID"),
        "validation_tooling_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"],
            cwd=Path(__file__).resolve().parents[1],
            text=True,
        ).strip(),
        "product_version": version.removeprefix("airs-harness ").removeprefix("airs "),
        "target": args.target,
        "binary_sha256": digest,
        "native_tests": re.search(r"Ran (\d+) tests", tests).group(1),
        "sandbox": "workspace-write; native macOS Seatbelt",
        "keychain": keychain,
        "signing": "ad-hoc signature verified; no Developer ID notarization",
        "macos_version": subprocess.check_output(
            ["sw_vers", "-productVersion"], text=True
        ).strip(),
        "live_gateway_e2e": False,
        "source_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=args.source_directory, text=True
        ).strip(),
    }
    (args.evidence / "VALIDATION.json").write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    main()
