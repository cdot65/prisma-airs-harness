"""Verify a pinned signed intake without executing or modifying it before trust checks."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import urllib.error
import urllib.request
import zipfile

TEAM = "G5QLZ5A8TA"
MEMBER = "prisma-airs-harness-alpha10-signing/airs-harness"
SIDECAR = "prisma-airs-harness-alpha10-signing/._airs-harness"
MAX_ARCHIVE = 200 * 1024 * 1024
MAX_BINARY = 400 * 1024 * 1024


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def download(asset_id, archive):
    """Keep the GitHub token on the API request, never on the signed blob redirect."""
    repository = os.environ["GITHUB_REPOSITORY"]
    if not re.fullmatch(r"[\w.-]+/[\w.-]+", repository) or not asset_id.isdecimal():
        raise ValueError("Invalid repository or asset identifier")
    request = urllib.request.Request(
        f"https://api.github.com/repos/{repository}/releases/assets/{asset_id}",
        headers={
            "Authorization": "Bearer " + os.environ["GH_TOKEN"],
            "Accept": "application/octet-stream",
        },
    )

    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *args):
            return None

    try:
        response = urllib.request.build_opener(NoRedirect).open(request, timeout=60)
    except urllib.error.HTTPError as error:
        if error.code != 302:
            raise ValueError(
                f"GitHub asset download returned HTTP {error.code}"
            ) from None
        location = error.headers["Location"]
        from urllib.parse import urlparse

        url = urlparse(location)
        if (
            url.scheme != "https"
            or not url.hostname
            or not url.hostname.endswith(".githubusercontent.com")
            or url.username is not None
            or url.password is not None
        ):
            raise ValueError("Unexpected GitHub asset redirect") from None
        response = urllib.request.build_opener(NoRedirect).open(location, timeout=60)
    archive.parent.mkdir(parents=True, exist_ok=True)
    size = 0
    with response, archive.open("xb") as output:
        while chunk := response.read(1024 * 1024):
            size += len(chunk)
            if size > MAX_ARCHIVE:
                raise ValueError("Signed archive exceeds size limit")
            output.write(chunk)


def restore(archive, directory, archive_sha256, binary_sha256):
    if archive.stat().st_size > MAX_ARCHIVE or digest(archive) != archive_sha256:
        raise ValueError("Signed archive hash or size mismatch")
    with zipfile.ZipFile(archive) as source:
        items = source.infolist()
        if len(items) != 2 or {item.filename for item in items} != {MEMBER, SIDECAR}:
            raise ValueError("Unexpected signed archive members")
        for item in items:
            kind = stat.S_IFMT(item.external_attr >> 16)
            limit = MAX_BINARY if item.filename == MEMBER else 1024 * 1024
            if kind not in (0, stat.S_IFREG) or not 0 < item.file_size <= limit:
                raise ValueError("Invalid signed archive member type or size")
        directory.mkdir(parents=True, exist_ok=False)
        binary = directory / "airs-harness"
        with source.open(MEMBER) as stream, binary.open("xb") as output:
            size = 0
            while chunk := stream.read(1024 * 1024):
                size += len(chunk)
                if size > MAX_BINARY:
                    raise ValueError("Signed executable exceeds size limit")
                output.write(chunk)
        if digest(binary) != binary_sha256:
            binary.unlink()
            raise ValueError("Signed executable hash mismatch")
        binary.chmod(0o755)
        # The exact AppleDouble member is intentionally not installed. Embedded
        # Mach-O signing remains intact and must pass actual native verification.
        return binary


def check_details(details):
    flags = re.search(r"\bflags=0x([0-9a-fA-F]+)\(", details)
    if not flags or not int(flags[1], 16) & 0x10000:
        raise ValueError("Hardened runtime is absent")
    if not re.search(rf"^TeamIdentifier={TEAM}$", details, re.MULTILINE):
        raise ValueError("Unexpected signing team")
    if not re.search(
        rf"^Authority=Developer ID Application: .+ \({TEAM}\)$", details, re.MULTILINE
    ):
        raise ValueError("Developer ID Application certificate is absent")
    if not re.search(r"^Timestamp=.+$", details, re.MULTILINE):
        raise ValueError("Secure signing timestamp is absent")


def verify(
    binary, binary_sha256, receipt, source_commit, asset_id, archive_sha256, submission
):
    if digest(binary) != binary_sha256:
        raise ValueError("Signed executable hash changed before verification")
    receipt.parent.mkdir(parents=True, exist_ok=True)
    logs = {}

    def run(name, command):
        result = subprocess.run(command, capture_output=True, text=True, timeout=120)
        output = result.stdout + result.stderr
        if len(output.encode()) > 256 * 1024:
            raise ValueError("Native verification output exceeds limit")
        log = receipt.with_name(receipt.stem + "-" + name + ".log")
        log.write_text(output)
        logs[log.name] = digest(log)
        if result.returncode:
            raise ValueError(f"Native {name} verification failed; inspect retained log")
        return output

    architecture = run("architecture", ["/usr/bin/lipo", "-archs", str(binary)])
    if architecture.strip() != "arm64":
        raise ValueError("Only thin Apple Silicon executables are accepted")
    requirement = f'=anchor apple generic and certificate leaf[subject.OU] = "{TEAM}" and certificate 1[field.1.2.840.113635.100.6.2.6] exists and certificate leaf[field.1.2.840.113635.100.6.1.13] exists'
    run(
        "codesign",
        [
            "/usr/bin/codesign",
            "--verify",
            "--strict",
            "--verbose=2",
            "-R",
            requirement,
            str(binary),
        ],
    )
    details = run("details", ["/usr/bin/codesign", "-d", "--verbose=4", str(binary)])
    check_details(details)
    # Apple WWDC2019 session703 prescribes an explicit notarized requirement
    # for non-app code. spctl's app assessment rejects standalone Mach-O tools.
    # Exit status is authoritative; successful codesign can produce no output.
    run(
        "notarization",
        [
            "/usr/bin/codesign",
            "--verify",
            "--strict",
            "--verbose=4",
            "-R",
            "=notarized",
            str(binary),
        ],
    )
    if digest(binary) != binary_sha256:
        raise ValueError("Signed executable changed during verification")
    result = {
        "schema_version": 1,
        "binary_sha256": binary_sha256,
        "target": "aarch64-apple-darwin",
        "team_id": TEAM,
        "codesign_verified": True,
        "hardened_runtime": True,
        "notarization_verified": True,
        "notarization_method": "codesign-explicit-notarized-requirement",
        "gatekeeper_app_assessment": "not-applicable-raw-cli",
        "source_commit": source_commit,
        "asset_id": asset_id,
        "archive_sha256": archive_sha256,
        "archive_member": MEMBER,
        "owner_reported_submission_id": submission,
        "submission_api_queried": False,
        "verification_log_sha256": logs,
        "published": False,
        "release_ready": False,
    }
    with receipt.open("x") as output:
        output.write(json.dumps(result, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["download", "restore", "verify"])
    parser.add_argument("--asset-id", required=True)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--directory", type=Path)
    parser.add_argument("--archive-sha256", required=True)
    parser.add_argument("--binary-sha256", required=True)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--receipt", type=Path)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--submission", default="dc835ddf-8841-49bc-a7ee-2f370dcb3457")
    args = parser.parse_args()
    if not re.fullmatch(r"[0-9a-f]{40}", args.source_commit) or any(
        not re.fullmatch(r"[0-9a-f]{64}", value)
        for value in [args.archive_sha256, args.binary_sha256]
    ):
        parser.error("Expected exact source and SHA256 identities")
    if args.mode == "download":
        download(args.asset_id, args.archive)
    elif args.mode == "restore":
        restore(args.archive, args.directory, args.archive_sha256, args.binary_sha256)
    else:
        verify(
            args.binary,
            args.binary_sha256,
            args.receipt,
            args.source_commit,
            args.asset_id,
            args.archive_sha256,
            args.submission,
        )


if __name__ == "__main__":
    main()
