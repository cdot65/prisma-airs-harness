#!/usr/bin/env python3
"""Verify release evidence from immutable Git objects, without using worktree bytes."""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import sys
import threading

from airs_bundle_archive import safe_path
from airs_release_receipts import atomic_json
from airs_test_release_spec import (
    COMMIT,
    MAX_JSON,
    SHA256,
    canonical_digest,
    json_bytes,
    require,
)

MAX_FILES = 4096
MAX_BLOB = 128 * 1024 * 1024
MAX_TOTAL = 512 * 1024 * 1024
MAX_TREE = 8 * 1024 * 1024
GIT_TIMEOUT = 20


def git_bytes(repository, arguments, limit):
    env = {
        key: value for key, value in os.environ.items() if not key.startswith("GIT_")
    }
    env["GIT_LITERAL_PATHSPECS"] = "1"
    env["GIT_TERMINAL_PROMPT"] = "0"
    env["GIT_NO_LAZY_FETCH"] = "1"
    env["GIT_ALLOW_PROTOCOL"] = ""
    command = ["git", "--no-replace-objects", "-C", str(repository), *arguments]
    # Bound reads while the child runs, rather than allocating arbitrary output
    # before checking its length. The deadline only owns this read-only Git child.
    with subprocess.Popen(
        command, env=env, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL
    ) as child:
        timer = threading.Timer(GIT_TIMEOUT, child.kill)
        timer.start()
        try:
            payload = child.stdout.read(limit + 1)
            require(len(payload) <= limit, "Git object output exceeds size limit")
            require(child.wait() == 0, "Git object could not be read")
            return payload
        finally:
            timer.cancel()
            if child.poll() is None:
                child.kill()
            child.wait()
            timer.join()


def digest_reference(records, relative, expected):
    relative = safe_path(relative).as_posix()
    require(
        isinstance(expected, str) and SHA256.fullmatch(expected),
        "Invalid evidence digest",
    )
    require(relative in records, f"Referenced evidence is not committed: {relative}")
    require(
        records[relative]["sha256"] == expected,
        f"Committed evidence digest mismatch: {relative}",
    )


def stage_reference(records, directory, name, expected):
    component = safe_path(name)
    require(len(component.parts) == 1, "Stage name must be one path component")
    digest_reference(records, (directory / f"{name}.json").as_posix(), expected)


def check_references(records, read):
    count = 0
    for relative in sorted(records):
        path = PurePosixPath(relative)
        if path.parent.name != "receipts" or path.suffix != ".json":
            continue
        receipt = json_bytes(read(relative, MAX_JSON))
        require(
            isinstance(receipt, dict)
            and set(receipt)
            == {
                "schema_version",
                "stage",
                "inputs",
                "input_sha256",
                "dependencies",
                "command",
                "command_sha256",
                "exit_code",
                "outputs",
            },
            "Invalid stage receipt schema",
        )
        require(
            type(receipt["schema_version"]) is int
            and receipt["schema_version"] == 1
            and receipt["stage"] == path.stem,
            "Stage receipt identity mismatch",
        )
        require(
            type(receipt["exit_code"]) is int and receipt["exit_code"] == 0,
            "Stage did not succeed",
        )
        require(
            isinstance(receipt["inputs"], dict)
            and canonical_digest(receipt["inputs"]) == receipt["input_sha256"],
            "Stage inputs changed",
        )
        command = receipt["command"]
        commands = (
            [command]
            if isinstance(command, list) and command and isinstance(command[0], str)
            else command
        )
        require(
            isinstance(commands, list)
            and commands
            and all(
                isinstance(row, list)
                and row
                and all(isinstance(value, str) for value in row)
                for row in commands
            )
            and canonical_digest(command) == receipt["command_sha256"],
            "Stage command changed",
        )
        outputs = receipt["outputs"]
        require(isinstance(outputs, dict) and outputs, "Missing stage outputs")
        for output, expected in outputs.items():
            output = (path.parent.parent / safe_path(output)).as_posix()
            require(
                isinstance(expected, dict) and set(expected) == {"sha256", "size"},
                "Invalid stage output",
            )
            digest_reference(records, output, expected["sha256"])
            require(
                type(expected["size"]) is int
                and expected["size"] >= 0
                and records[output]["size"] == expected["size"],
                "Stage output size mismatch",
            )
        dependencies = receipt["dependencies"]
        require(isinstance(dependencies, dict), "Invalid stage dependencies")
        for name, expected in dependencies.items():
            stage_reference(records, path.parent, name, expected)
        count += 1
    for relative in sorted(records):
        path = PurePosixPath(relative)
        if path.name != "ACCEPTANCE.json":
            continue
        acceptance = json_bytes(read(relative, MAX_JSON))
        require(isinstance(acceptance, dict), "Invalid acceptance receipt")
        receipts = acceptance.get("stage_receipts")
        require(
            isinstance(receipts, dict) and receipts,
            "Missing acceptance stage references",
        )
        actual = {
            PurePosixPath(name).stem
            for name in records
            if PurePosixPath(name).parent == path.parent / "receipts"
            and name.endswith(".json")
        }
        require(set(receipts) == actual, "Acceptance stage inventory mismatch")
        for name, expected in receipts.items():
            stage_reference(records, path.parent / "receipts", name, expected)
    return count


def audit(repository, commit, root):
    require(
        isinstance(commit, str) and COMMIT.fullmatch(commit),
        "Use an explicit full commit object ID",
    )
    root = safe_path(root).as_posix()
    require(
        git_bytes(repository, ["cat-file", "-t", commit], 32) == b"commit\n",
        "Evidence revision is not a commit",
    )
    listing = git_bytes(
        repository,
        ["ls-tree", "-r", "-z", "-l", "--full-tree", commit, "--", root],
        MAX_TREE,
    )
    entries = {}
    for row in listing.split(b"\0"):
        if not row:
            continue
        metadata, raw_path = row.split(b"\t", 1)
        mode, kind, oid, size = metadata.split()
        require(
            mode in (b"100644", b"100755") and kind == b"blob",
            "Linked or non-regular evidence is prohibited",
        )
        path = safe_path(raw_path.decode("utf-8"))
        require(
            path.is_relative_to(PurePosixPath(root)) and path != PurePosixPath(root),
            "Evidence root must be a directory",
        )
        relative = path.relative_to(root).as_posix()
        require(
            relative not in entries and len(entries) < MAX_FILES,
            "Duplicate or excessive evidence inventory",
        )
        require(
            COMMIT.fullmatch(oid.decode("ascii")) is not None,
            "Invalid Git blob identity",
        )
        size = int(size)
        require(0 <= size <= MAX_BLOB, "Evidence blob exceeds size limit")
        entries[relative] = {"blob_id": oid.decode("ascii"), "size": size}
    require(
        entries and "SHA256SUMS" in entries, "Committed evidence manifest is missing"
    )
    require(
        sum(row["size"] for row in entries.values()) <= MAX_TOTAL,
        "Evidence exceeds aggregate size limit",
    )
    cache = {}

    def read(relative, limit=MAX_BLOB):
        require(relative in entries, f"Evidence is not committed: {relative}")
        entry = entries[relative]
        require(entry["size"] <= limit, "Evidence exceeds format size limit")
        oid = entry["blob_id"]
        if oid not in cache:
            cache[oid] = git_bytes(repository, ["cat-file", "blob", oid], entry["size"])
        require(len(cache[oid]) == entry["size"], "Git blob size mismatch")
        return cache[oid]

    manifest = read("SHA256SUMS", MAX_JSON)
    expected = {}
    for row in manifest.decode("utf-8").splitlines():
        digest, separator, relative = row.partition("  ")
        require(separator and SHA256.fullmatch(digest), "Invalid checksum manifest row")
        relative = safe_path(relative).as_posix()
        require(
            relative != "SHA256SUMS" and relative not in expected,
            "Duplicate or self-referencing manifest entry",
        )
        expected[relative] = digest
    require(
        expected and set(expected) == set(entries) - {"SHA256SUMS"},
        "Committed manifest inventory is incomplete",
    )
    records = {}
    for relative in sorted(expected):
        digest = hashlib.sha256(read(relative)).hexdigest()
        require(
            digest == expected[relative],
            f"Committed manifest digest mismatch: {relative}",
        )
        records[relative] = {**entries[relative], "sha256": digest}
    stages = check_references(records, read)
    return {
        "schema_version": 1,
        "scope": "committed-release-evidence",
        "passed": True,
        "evidence_commit": commit,
        "evidence_root": root,
        "manifest_blob_id": entries["SHA256SUMS"]["blob_id"],
        "manifest_sha256": hashlib.sha256(manifest).hexdigest(),
        "files_checked": len(records),
        "stage_receipts_checked": stages,
        "aggregate_bytes": sum(row["size"] for row in entries.values()),
        "inventory_sha256": canonical_digest(records),
        "product_acceptance_claimed": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--root", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        result = audit(args.repository, args.commit, args.root)
        atomic_json(args.output, result)
        print(json.dumps(result, sort_keys=True))
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"Release evidence audit failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
