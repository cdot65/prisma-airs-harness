"""Portable, fail-closed stage evidence for owner-authorized test releases."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from airs_bundle_archive import safe_path
from airs_test_release_spec import canonical_digest, digest_file, load_json, require


def safe_destination(path):
    path = Path(path).absolute()
    for parent in [path, *path.parents]:
        require(not parent.is_symlink(), "Linked evidence destination is prohibited")
    return path


def evidence_path(root, relative):
    parts = safe_path(relative).parts
    path = safe_destination(root)
    for part in parts:
        path = path / part
        if path.exists() or path.is_symlink():
            require(not path.is_symlink(), "Linked evidence is prohibited")
    require(path.is_file(), "Evidence file is missing")
    return path


def atomic_json(path, value):
    path = safe_destination(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    require(not path.is_symlink(), "Linked receipt destination is prohibited")
    payload = json.dumps(value, indent=2, sort_keys=True, allow_nan=False).encode() + b"\n"
    require(len(payload) <= 1024 * 1024, "Receipt exceeds size limit")
    descriptor, temporary = tempfile.mkstemp(prefix=".receipt-", dir=path.parent)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        Path(temporary).unlink(missing_ok=True)


def output_records(root, paths):
    result = {}
    for relative in paths:
        path = evidence_path(root, relative)
        result[relative] = {"sha256": digest_file(path), "size": path.stat().st_size}
    require(result, "A successful stage needs retained output evidence")
    return result


def write_stage(root, name, inputs, dependencies, command, paths):
    receipt = {
        "schema_version": 1, "stage": name, "inputs": inputs,
        "input_sha256": canonical_digest(inputs), "dependencies": dependencies,
        "command": command, "command_sha256": canonical_digest(command), "exit_code": 0, "outputs": output_records(root, paths),
    }
    atomic_json(Path(root) / "receipts" / f"{name}.json", receipt)
    return receipt


def verify_stage(root, name, inputs, dependencies):
    path = evidence_path(root, f"receipts/{name}.json")
    receipt = load_json(path)
    require(isinstance(receipt, dict) and set(receipt) == {
        "schema_version", "stage", "inputs", "input_sha256", "dependencies",
        "command", "command_sha256", "exit_code", "outputs",
    }, "Invalid stage receipt schema")
    require(receipt["schema_version"] == 1 and receipt["stage"] == name, "Stage identity mismatch")
    require(type(receipt["exit_code"]) is int and receipt["exit_code"] == 0, "Stage did not succeed")
    require(receipt["inputs"] == inputs and receipt["input_sha256"] == canonical_digest(inputs), "Stale or mismatched stage inputs")
    require(receipt["dependencies"] == dependencies, "Stage dependency evidence changed")
    commands = receipt["command"]
    if isinstance(commands, list) and commands and isinstance(commands[0], str):
        commands = [commands]
    require(isinstance(commands, list) and commands and all(isinstance(command, list) and command and all(isinstance(v, str) for v in command) for command in commands), "Missing stage invocation")
    require(receipt["command_sha256"] == canonical_digest(receipt["command"]), "Stage invocation changed")
    outputs = receipt["outputs"]
    require(isinstance(outputs, dict) and outputs, "Missing stage outputs")
    require(output_records(root, outputs) == outputs, "Stage output evidence changed")
    return receipt


def tooling_files(root):
    root = Path(root)
    paths = list((root / "scripts").glob("*.py"))
    paths.extend(path for path in (root / "scripts/fixtures").rglob("*") if path.is_file() or path.is_symlink())
    paths.extend((root / "codex-rs/airs-identity/src/fixtures").glob("test-only-*"))
    return sorted(paths)


def snapshot_tooling(scripts, commit):
    """Bind transfer-ready validators and synthetic identity keys to git bytes."""
    scripts = Path(scripts).resolve(strict=True)
    root = scripts.parent
    paths = tooling_files(root)
    files = {}
    for path in paths:
        relative = path.relative_to(root).as_posix()
        committed = subprocess.run(["git", "show", f"{commit}:{relative}"], cwd=root, capture_output=True, check=True, timeout=20).stdout
        digest = hashlib.sha256(committed).hexdigest()
        require(digest_file(path) == digest, f"Validation tooling differs from commit: {relative}")
        files[relative] = digest
    require(files and any(key.endswith("test-only-private.pem") for key in files), "Validation tooling lacks native identity fixtures")
    return {"schema_version": 1, "tooling_commit": commit, "files": files}


def verify_tooling(root, manifest, commit):
    require(isinstance(manifest, dict) and manifest.get("schema_version") == 1 and manifest.get("tooling_commit") == commit, "Validation tooling source mismatch")
    files = manifest.get("files")
    require(isinstance(files, dict) and files, "Missing validation tooling files")
    root = Path(root)
    actual = {path.relative_to(root).as_posix() for path in tooling_files(root)}
    require(actual == set(files), "Validation tooling inventory changed")
    for relative, expected in files.items():
        require(digest_file(evidence_path(root, relative)) == expected, "Validation tooling bytes changed")
    return canonical_digest(manifest)
