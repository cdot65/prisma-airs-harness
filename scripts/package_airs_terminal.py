#!/usr/bin/env python3
"""Package an AIRS Linux binary with locked dependency and license provenance.

Generate metadata with cargo metadata --locked --filter-platform
x86_64-unknown-linux-musl --format-version 1. The inventory is the resolved normal
and build dependency closure, not a claim that every package is linked at runtime.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--output-directory", type=Path, required=True)
    parser.add_argument(
        "--binary-processing", default="none", choices=["none", "strip --strip-debug"]
    )
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    binary = args.binary.resolve(strict=True)
    metadata = json.loads(args.metadata.read_text())
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    pending = [
        package["id"] for package in packages.values() if package["name"] == "codex-cli"
    ]
    selected = set()
    while pending:
        identifier = pending.pop()
        if identifier in selected:
            continue
        selected.add(identifier)
        pending.extend(
            dep["pkg"]
            for dep in nodes[identifier]["deps"]
            if any(kind["kind"] != "dev" for kind in dep["dep_kinds"])
        )
    version_output = subprocess.check_output(
        [str(binary), "--version"], text=True
    ).strip()
    if not version_output.startswith("airs-terminal "):
        raise ValueError("Expected an AIRS Terminal binary")
    version = version_output.removeprefix("airs-terminal ")
    commit = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=repo, text=True
    ).strip()
    source_status = subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=repo, text=True
    )
    if source_status.strip():
        raise ValueError("Commit the reviewed source before packaging")
    args.output_directory.mkdir(parents=True, exist_ok=True)
    name = f"airs-terminal-{version}-linux-x86_64-musl"
    archive = args.output_directory / (name + ".tar.gz")
    if archive.exists():
        raise FileExistsError(archive)
    with tempfile.TemporaryDirectory(prefix="airs-package-") as temporary:
        root = Path(temporary) / name
        root.mkdir()
        shutil.copy2(binary, root / "airs-terminal")
        (root / "airs-terminal").chmod(0o755)
        for filename in [
            "LICENSE",
            "NOTICE",
            "README.md",
            "RELEASE.md",
            "UPSTREAM.md",
            "BASELINE.json",
            "VALIDATION.json",
            "AGENTS.md",
            "README.upstream.md",
            "IMPLEMENTATION.md",
            "PLAN.md",
        ]:
            shutil.copy2(repo / filename, root / filename)
        for directory in ["validation", "administration"]:
            shutil.copytree(repo / directory, root / directory)
        shutil.copytree(
            repo / "mcp-scanner",
            root / "mcp-scanner",
            ignore=shutil.ignore_patterns("node_modules", "dist", "__pycache__"),
        )
        (root / "scripts").mkdir()
        for filename in [
            "test_airs_terminal.py",
            "test_airs_terminal_pty.py",
            "validate_live_agent.py",
            "validate_live_gateway.py",
            "validate_mcp_sessions.py",
            "verify_airs_release.py",
        ]:
            shutil.copy2(repo / "scripts" / filename, root / "scripts" / filename)
        rust_sysroot = Path(
            subprocess.check_output(
                ["rustc", "--print", "sysroot"], cwd=repo / "codex-rs", text=True
            ).strip()
        )
        rust_notices = rust_sysroot / "share/doc/rust"
        runtime_notices = root / "licenses/rust-toolchain"
        runtime_notices.mkdir(parents=True)
        for filename in ["COPYRIGHT.html", "COPYRIGHT-library.html"]:
            shutil.copy2(rust_notices / filename, runtime_notices / filename)
        shutil.copytree(rust_notices / "licenses", runtime_notices / "licenses")
        inventory = []
        for identifier in sorted(selected):
            package = packages[identifier]
            record = {
                key: package.get(key)
                for key in ["name", "version", "source", "license", "repository"]
            }
            record["notices"] = []
            if package["source"]:
                package_root = Path(package["manifest_path"]).parent
                for directory, subdirectories, files in os.walk(package_root):
                    subdirectories[:] = [
                        d for d in subdirectories if d not in {".git", "target"}
                    ]
                    for filename in sorted(files):
                        if not filename.upper().startswith(
                            (
                                "LICENSE",
                                "LICENCE",
                                "COPYING",
                                "NOTICE",
                                "COPYRIGHT",
                                "UNLICENSE",
                                "PATENTS",
                            )
                        ):
                            continue
                        source = Path(directory) / filename
                        if source.is_symlink() or not source.is_file():
                            continue
                        relative = (
                            Path("licenses")
                            / (package["name"] + "-" + package["version"])
                            / source.relative_to(package_root)
                        )
                        destination = root / relative
                        destination.parent.mkdir(parents=True, exist_ok=True)
                        shutil.copyfile(source, destination)
                        record["notices"].append(str(relative))
            inventory.append(record)
        (root / "DEPENDENCIES.json").write_text(
            json.dumps(
                {
                    "scope": "resolved Linux normal/build dependency closure; includes build-only packages",
                    "rust_distribution_notices": "licenses/rust-toolchain; includes standard-library and build-tool notices",
                    "packages": inventory,
                },
                indent=2,
            )
            + "\n"
        )
        provenance = {
            "product": "Prisma AIRS Terminal",
            "version": version,
            "source_commit": commit,
            "upstream_commit": "3d2ee51ca2d5db578f328aa75e20aa22c0197c9a",
            "target": "x86_64-unknown-linux-musl",
            "profile": "release; upstream defaults",
            "binary_processing": args.binary_processing,
            "binary_sha256": digest(root / "airs-terminal"),
            "cargo_lock_sha256": digest(repo / "codex-rs/Cargo.lock"),
            "rust": subprocess.check_output(
                ["rustc", "--version"], cwd=repo / "codex-rs", text=True
            ).strip(),
            "build_command": "cargo build --locked --release -p codex-cli --bin airs-terminal",
        }
        (root / "BUILD-INFO.json").write_text(json.dumps(provenance, indent=2) + "\n")
        files = sorted(path for path in root.rglob("*") if path.is_file())
        (root / "SHA256SUMS").write_text(
            "".join(f"{digest(path)}  {path.relative_to(root)}\n" for path in files)
        )
        with tarfile.open(archive, "w:gz") as tar:
            tar.add(root, arcname=name)
    archive.with_suffix(archive.suffix + ".sha256").write_text(
        f"{digest(archive)}  {archive.name}\n"
    )
    print(
        json.dumps(
            {
                "archive": str(archive),
                "sha256": digest(archive),
                "source_commit": commit,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
