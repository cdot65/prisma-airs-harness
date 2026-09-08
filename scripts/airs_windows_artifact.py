#!/usr/bin/env python3
"""Preserve and restore immutable private Windows build inputs without compiling."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import tarfile


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def identity(value, length):
    if not re.fullmatch(r"[0-9a-f]{" + str(length) + "}", value):
        raise ValueError("Invalid preserved artifact identity")
    return value


def describe(source, binary, fixture, opt_level):
    return {
        "schema_version": 1,
        "private_candidate": True,
        "published": False,
        "release_ready": False,
        "runtime_source": identity(source, 40),
        "build_tooling_source": os.environ.get("GITHUB_SHA"),
        "binary_sha256": binary,
        "fixture_sha256": fixture,
        "cli_opt_level": opt_level,
        "target": "x86_64-pc-windows-msvc",
        "signing": {"unsigned_candidate": True, "production_signing": False},
    }


def create(directory, source, opt_level):
    source = identity(source, 40)
    if opt_level not in {"1", "3"}:
        raise ValueError("Unsupported CLI build profile")
    manifest = describe(
        source,
        digest(directory / "airs-harness.exe"),
        digest(directory / "store_acceptance.exe"),
        opt_level,
    )
    (directory / "runtime-source.txt").write_bytes((source + "\n").encode())
    (directory / "CANDIDATE.json").write_text(json.dumps(manifest, indent=2) + "\n")
    archive = directory / "unvalidated-windows-executables.tar.gz"
    with tarfile.open(archive, "w:gz") as tar:
        for name in [
            "airs-harness.exe",
            "store_acceptance.exe",
            "runtime-source.txt",
            "CANDIDATE.json",
        ]:
            path = directory / name
            if path.is_symlink() or not path.is_file():
                raise ValueError("Build artifact input must be a regular file")
            tar.add(path, arcname=name)
    return manifest | {"archive_sha256": digest(archive)}


def restore(archive, directory, source, archive_sha, binary_sha, fixture_sha):
    source = identity(source, 40)
    for value in [archive_sha, binary_sha, fixture_sha]:
        identity(value, 64)
    if digest(archive) != archive_sha:
        raise ValueError("Preserved archive digest mismatch")
    limits = {
        "airs-harness.exe": 512 * 1024 * 1024,
        "store_acceptance.exe": 512 * 1024 * 1024,
        "runtime-source.txt": 41,
        "CANDIDATE.json": 16384,
    }
    directory.mkdir(parents=True, exist_ok=True)
    seen = set()
    with tarfile.open(archive, "r:gz") as tar:
        for member in tar:
            if (
                member.name not in limits
                or member.name in seen
                or not member.isfile()
                or not 0 < member.size <= limits[member.name]
            ):
                raise ValueError("Invalid preserved artifact member")
            seen.add(member.name)
            with (
                tar.extractfile(member) as incoming,
                (directory / member.name).open("xb") as outgoing,
            ):
                shutil.copyfileobj(incoming, outgoing)
    if not {"airs-harness.exe", "store_acceptance.exe", "runtime-source.txt"}.issubset(
        seen
    ):
        raise ValueError("Incomplete preserved artifact")
    if (directory / "runtime-source.txt").read_bytes() != (source + "\n").encode():
        raise ValueError("Preserved runtime source mismatch")
    if (
        digest(directory / "airs-harness.exe") != binary_sha
        or digest(directory / "store_acceptance.exe") != fixture_sha
    ):
        raise ValueError("Preserved executable digest mismatch")
    if "CANDIDATE.json" in seen:
        manifest = json.loads((directory / "CANDIDATE.json").read_text())
        if (
            manifest.get("schema_version") != 1
            or manifest.get("private_candidate") is not True
            or manifest.get("published") is not False
            or manifest.get("release_ready") is not False
            or manifest.get("runtime_source") != source
            or manifest.get("binary_sha256") != binary_sha
            or manifest.get("fixture_sha256") != fixture_sha
            or manifest.get("cli_opt_level") not in {"1", "3"}
            or manifest.get("target") != "x86_64-pc-windows-msvc"
            or manifest.get("signing")
            != {"unsigned_candidate": True, "production_signing": False}
        ):
            raise ValueError("Invalid private candidate provenance")
    else:
        raise ValueError("Missing private candidate provenance")
    for name in ["airs-harness.exe", "store_acceptance.exe"]:
        (directory / name).chmod(0o700)
    return manifest | {"archive_sha256": archive_sha}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["create", "restore"])
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--cli-opt-level", choices=["1", "3"])
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--archive-sha256")
    parser.add_argument("--binary-sha256")
    parser.add_argument("--fixture-sha256")
    args = parser.parse_args()
    if args.mode == "create":
        result = create(args.directory, args.source_commit, args.cli_opt_level)
    else:
        if any(
            value is None
            for value in [
                args.archive,
                args.archive_sha256,
                args.binary_sha256,
                args.fixture_sha256,
            ]
        ):
            parser.error(
                "Restoring requires the archive and all three independent digests"
            )
        result = restore(
            args.archive,
            args.directory,
            args.source_commit,
            args.archive_sha256,
            args.binary_sha256,
            args.fixture_sha256,
        )
    level = result["cli_opt_level"]
    profile = f"release; codex-cli opt-level={level}; other packages use release settings; lto=false; codegen-units=16; debug=0"
    command = f"cargo --config profile.release.package.codex-cli.opt-level={level} build --locked --release -p codex-cli --bin airs-harness --timings"
    result.update(profile=profile, build_command=command)
    if output := os.environ.get("GITHUB_OUTPUT"):
        with open(output, "a") as stream:
            for name in [
                "runtime_source",
                "binary_sha256",
                "fixture_sha256",
                "archive_sha256",
                "profile",
                "build_command",
            ]:
                stream.write(f"{name}={result[name]}\n")
    (args.directory / "ARTIFACT-VERIFIED.json").write_text(
        json.dumps(result, indent=2) + "\n"
    )
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
