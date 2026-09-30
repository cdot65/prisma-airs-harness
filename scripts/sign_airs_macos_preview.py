#!/usr/bin/env python3
"""Sign verified CI input, notarize it, and bind acceptance to the signed bytes."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import zipfile

from airs_signed_macos_artifact import MEMBER, digest, verify

IDENTITY = "9E9F6E0D92526D725FABC1BBDC2F1757049F1805"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    selection_path = args.evidence / "artifact-selection.json"
    selection = json.loads(selection_path.read_text())
    original = digest(args.binary)
    if original != selection["binary_sha256"]:
        raise ValueError("Compiled binary differs from verified CI intake")
    keychain = str(Path.home() / "Library/Keychains/login.keychain-db")
    subprocess.run(
        [
            "codesign",
            "--force",
            "--identifier",
            "airs-harness",
            "--timestamp",
            "--options",
            "runtime",
            "--keychain",
            keychain,
            "--sign",
            IDENTITY,
            str(args.binary),
        ],
        check=True,
        timeout=120,
    )
    archive = args.evidence / "signed.zip"
    with zipfile.ZipFile(archive, "x", compression=zipfile.ZIP_DEFLATED) as output:
        output.write(args.binary, MEMBER)
    notarization = subprocess.run(
        [
            "xcrun",
            "notarytool",
            "submit",
            str(archive),
            "--keychain-profile",
            "prisma-airs-harness-notary",
            "--keychain",
            keychain,
            "--wait",
            "--timeout",
            "30m",
            "--output-format",
            "json",
        ],
        capture_output=True,
        text=True,
        timeout=1900,
    )
    (args.evidence / "NOTARIZATION.json").write_text(notarization.stdout)
    notarization.check_returncode()
    submission = json.loads(notarization.stdout)
    if submission.get("status") != "Accepted":
        raise ValueError("Apple did not accept the signed archive")
    verify(
        args.binary,
        digest(args.binary),
        args.evidence / "SIGNING.json",
        selection["runtime_source"],
        "forgejo-run-" + os.environ["GITHUB_RUN_ID"],
        digest(archive),
        submission["id"],
    )
    selection.update(
        compiled_binary_sha256=original,
        binary_sha256=digest(args.binary),
        signing={"ad_hoc": False, "developer_id": True, "notarized": True},
    )
    selection_path.write_text(json.dumps(selection, indent=2) + "\n")


if __name__ == "__main__":
    main()
