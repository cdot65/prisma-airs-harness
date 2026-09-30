#!/usr/bin/env python3
"""Check signing identity and disposable Keychain reuse in a desktop session."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import subprocess
import sys

from airs_signed_macos_artifact import DESIGNATED_REQUIREMENT, check_details


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--previous", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    if sys.platform != "darwin":
        raise RuntimeError("This check requires macOS Keychain")
    identities = {}
    for label, binary in [("previous", args.previous), ("candidate", args.candidate)]:
        result = subprocess.run(
            ["codesign", "--verify", "--strict", str(binary)],
            capture_output=True,
            text=True,
            check=True,
            timeout=30,
        )
        result = subprocess.run(
            ["codesign", "-d", "-r-", "--verbose=4", str(binary)],
            capture_output=True,
            text=True,
            check=True,
            timeout=30,
        )
        details = result.stdout + result.stderr
        check_details(details)
        requirement = re.search(r"^designated => (.+)$", details, re.MULTILINE)
        if not requirement or requirement[1] != DESIGNATED_REQUIREMENT:
            raise ValueError("Signing requirement changed between releases")
        identities[label] = {
            "version": subprocess.check_output(
                [str(binary), "--version"], text=True
            ).strip(),
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "designated_requirement": requirement[1],
        }
    os.environ["AIRS_HARNESS_BIN"] = str(args.previous.resolve(strict=True))
    from test_airs_harness import TerminalIntegration

    fixture = TerminalIntegration()
    fixture.setUp()
    key = secrets.token_urlsafe(32)

    def run(binary, *command, stdin=None):
        return subprocess.run(
            [str(binary), *command],
            input=stdin,
            env=fixture.env,
            cwd=fixture.work,
            capture_output=True,
            text=True,
            timeout=30,
        )

    try:
        fixture.configure()
        fixture.env.pop("AIRS_TEST_CREDENTIAL", None)
        result = run(args.previous, "login", "--with-api-key", stdin=key + "\n")
        if result.returncode:
            raise RuntimeError(
                "Previous release could not save the test credential: "
                + result.stderr.replace(key, "[REDACTED]")[-2000:]
            )
        binding = json.loads((fixture.home / "credential-binding.json").read_text())
        helper = ("credential", "--home", str(fixture.home), "--binding", binding["id"])
        for binary in [args.previous, args.candidate, args.previous, args.candidate]:
            result = run(binary, *helper)
            if result.returncode or result.stdout != key + "\n":
                raise RuntimeError("Cross-release Keychain read failed")
        args.receipt.write_text(
            json.dumps(
                {
                    "passed": True,
                    "scope": "disposable credential; no owner credentials",
                    "identities": identities,
                    "same_designated_requirement": True,
                    "previous_write_candidate_read": True,
                    "rollback_read": True,
                },
                indent=2,
            )
            + "\n"
        )
    finally:
        run(args.previous, "logout")
        fixture.doCleanups()


if __name__ == "__main__":
    main()
