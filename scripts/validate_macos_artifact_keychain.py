#!/usr/bin/env python3
"""Privately test one preserved ARM CLI's Keychain lifecycle, without a build."""

import argparse
import contextlib
import hashlib
import io
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def extract_verified(archive, directory, source, expected_sha):
    """Copy only bounded regular members; never apply archive paths or links."""
    with tarfile.open(archive, "r:gz") as tar:
        members = tar.getmembers()
        for name, limit in [
            ("airs-harness", 512 * 1024 * 1024),
            ("runtime-source.txt", 41),
        ]:
            matches = [member for member in members if member.name == name]
            if (
                len(matches) != 1
                or not matches[0].isfile()
                or not 0 < matches[0].size <= limit
            ):
                raise ValueError("Invalid preserved artifact member")
            with (
                tar.extractfile(matches[0]) as incoming,
                (directory / name).open("xb") as outgoing,
            ):
                shutil.copyfileobj(incoming, outgoing)
    if (directory / "runtime-source.txt").read_bytes() != (source + "\n").encode():
        raise ValueError("Preserved runtime source mismatch")
    binary = directory / "airs-harness"
    if digest(binary) != expected_sha:
        raise ValueError("Preserved executable digest mismatch")
    binary.chmod(0o700)
    return binary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--binary-sha256", required=True)
    parser.add_argument("--build-run", required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    receipt = {
        "passed": False,
        "release_ready": False,
        "published": False,
        "owner_device_tested": False,
        "scope": "preserved Apple Silicon CLI Keychain lifecycle on hosted macOS; no build or promotion",
        "phase": "input-validation",
    }
    try:
        if sys.platform != "darwin":
            raise ValueError("Native macOS required")
        for value, pattern in [
            (args.source_commit, r"[0-9a-f]{40}"),
            (args.binary_sha256, r"[0-9a-f]{64}"),
            (args.build_run, r"[0-9]+"),
        ]:
            if not re.fullmatch(pattern, value):
                raise ValueError("Invalid artifact identity")
        receipt.update(
            source_commit=args.source_commit,
            binary_sha256=args.binary_sha256,
            build_run=args.build_run,
        )
        receipt["archive_sha256"] = digest(args.archive)
        with tempfile.TemporaryDirectory(prefix="airs-mac-keychain-") as temporary:
            work = Path(temporary)
            receipt["phase"] = "artifact-verification"
            binary = extract_verified(
                args.archive, work, args.source_commit, args.binary_sha256
            )
            architecture = subprocess.run(
                ["lipo", "-archs", str(binary)],
                capture_output=True,
                text=True,
                check=True,
                timeout=15,
            )
            if architecture.stdout.strip() != "arm64":
                raise ValueError("Apple Silicon executable required")
            subprocess.run(
                ["codesign", "--verify", "--strict", "--verbose=2", str(binary)],
                capture_output=True,
                check=True,
                timeout=30,
            )
            signing = subprocess.run(
                ["codesign", "-dvv", str(binary)],
                capture_output=True,
                text=True,
                check=True,
                timeout=15,
            )
            if "Signature=adhoc" not in signing.stderr.splitlines():
                raise ValueError("Expected preserved ad-hoc signature")
            receipt.update(
                artifact_verified=True,
                architecture="arm64",
                signature="verified ad-hoc; not Developer ID or notarization",
            )
            receipt["phase"] = "actual-cli-keychain-lifecycle"
            import validate_airs_macos_keychain

            nested_receipt = work / "keychain.json"
            # A failure can contain captured credentials. Never retain a traceback,
            # exception message, stdout, or stderr from this fixture invocation.
            with (
                contextlib.redirect_stdout(io.StringIO()),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                sys.argv = [
                    "validate_airs_macos_keychain",
                    "--binary",
                    str(binary),
                    "--receipt",
                    str(nested_receipt),
                ]
                validate_airs_macos_keychain.main()
            result = json.loads(nested_receipt.read_text())
            names = [
                "passed",
                "cli_secure_login",
                "oidc_native_store_preflight",
                "new_process_credential_access",
                "logout_rejects_credential_helper",
                "authenticated_local_tool_loop",
                "plaintext_state_absent",
                "logout_blocks_inference",
            ]
            if not all(result.get(name) is True for name in names):
                raise ValueError("Incomplete native fixture receipt")
            receipt["checks"] = {name: True for name in names if name != "passed"}
            receipt.update(passed=True, phase="completed")
    except Exception as error:
        receipt["failure_class"] = type(error).__name__
    finally:
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))
    return 0 if receipt["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
