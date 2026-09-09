#!/usr/bin/env python3
"""Package an AIRS native binary with locked dependency and license provenance.

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

from airs_review_release import validate_release


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument(
        "--source-directory", type=Path, default=Path(__file__).resolve().parents[1]
    )
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--output-directory", type=Path, required=True)
    parser.add_argument(
        "--binary-processing",
        default="none",
        choices=["none", "strip --strip-debug", "strip -S; codesign --force --sign -"],
    )
    targets = {
        "x86_64-unknown-linux-musl": "linux-x86_64-musl",
        "aarch64-apple-darwin": "darwin-arm64",
        "x86_64-pc-windows-msvc": "windows-x86_64-msvc",
    }
    parser.add_argument(
        "--target", choices=targets, default="x86_64-unknown-linux-musl"
    )
    parser.add_argument("--validation", type=Path)
    parser.add_argument("--validation-evidence-root", type=Path)
    parser.add_argument(
        "--signing-receipt", type=Path,
        help="Mac verification receipt bound to these exact signed executable bytes",
    )
    parser.add_argument(
        "--unvalidated-candidate",
        action="store_true",
        help="Create a non-publishable candidate, not a validated release",
    )
    parser.add_argument("--profile", default="release; upstream defaults")
    parser.add_argument(
        "--build-command",
        default="cargo build --locked --release -p codex-cli --bin airs-harness",
        help="Nonsecret build command recorded as provenance text only; never executed",
    )
    args = parser.parse_args()
    windows = args.target == "x86_64-pc-windows-msvc"
    if windows and not args.unvalidated_candidate:
        parser.error(
            "Windows requires --unvalidated-candidate until signing and acceptance are implemented"
        )
    if args.unvalidated_candidate and args.validation:
        parser.error(
            "An unvalidated candidate cannot include a release validation receipt"
        )
    if not args.unvalidated_candidate and not args.validation:
        parser.error("Provide bound review validation or --unvalidated-candidate")
    if bool(args.validation) != bool(args.validation_evidence_root):
        parser.error("Validation requires --validation-evidence-root")
    binary_name = "airs-harness.exe" if windows else "airs-harness"
    repo = args.source_directory.resolve(strict=True)
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
    if not version_output.startswith("airs-harness "):
        raise ValueError("Expected an AIRS Harness binary")
    version = version_output.removeprefix("airs-harness ")
    commit = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=repo, text=True
    ).strip()
    source_status = subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=repo, text=True
    )
    if source_status.strip():
        raise ValueError("Commit the reviewed source before packaging")
    signing = None
    if args.signing_receipt:
        signing = json.loads(args.signing_receipt.read_text())
        required = {
            "binary_sha256": digest(binary),
            "target": "aarch64-apple-darwin",
            "source_commit": commit,
            "team_id": "G5QLZ5A8TA",
            "codesign_verified": True,
            "hardened_runtime": True,
            "notarization_verified": True,
        }
        if (
            args.target != "aarch64-apple-darwin"
            or args.binary_processing != "none"
            or any(type(signing.get(k)) is not type(v) or signing[k] != v
                   for k, v in required.items())
        ):
            raise ValueError("Signing receipt does not verify this unchanged Mac binary/source")
    candidate_status = (
        "signed-unvalidated-candidate" if signing else "unsigned-unvalidated-candidate"
    )
    args.output_directory.mkdir(parents=True, exist_ok=True)
    name = f"airs-harness-{version}-{targets[args.target]}"
    archive = args.output_directory / (name + ".tar.gz")
    if archive.exists():
        raise FileExistsError(archive)
    with tempfile.TemporaryDirectory(prefix="airs-package-") as temporary:
        root = Path(temporary) / name
        root.mkdir()
        shutil.copy2(binary, root / binary_name)
        (root / binary_name).chmod(0o755)
        if signing is not None:
            shutil.copy2(args.signing_receipt, root / "SIGNING.json")
        for filename in [
            "LICENSE",
            "NOTICE",
            "README.md",
            "RELEASE.md",
            "RENAME.md",
            "MACOS.md",
            "PRISMA-AIRS-CLI.md",
            "UPSTREAM.md",
            "BASELINE.json",
            "VALIDATION.json",
            "AGENTS.md",
            "README.upstream.md",
            "IMPLEMENTATION.md",
            "PLAN.md",
        ]:
            shutil.copy2(repo / filename, root / filename)
        if args.validation:
            validation = json.loads(args.validation.read_text())
            validate_release(
                validation, binary_sha256=digest(binary), target=args.target,
                version=version, source_commit=commit,
                evidence_root=args.validation_evidence_root,
            )
            if args.target == "aarch64-apple-darwin" and signing is None:
                raise ValueError("Mac review release requires verified signing receipt")
            shutil.copy2(args.validation, root / "VALIDATION.json")
            for record in validation["evidence"]:
                destination = root / "validation-evidence" / record["path"]
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(args.validation_evidence_root / record["path"], destination)
                if digest(destination) != record["sha256"]:
                    raise ValueError("Release evidence changed during packaging")
        if args.unvalidated_candidate:
            (root / "VALIDATION.json").write_text(
                json.dumps(
                    {
                        "passed": False,
                        "release_ready": False,
                        "status": candidate_status,
                        "target": args.target,
                        "product_version": version,
                        "binary_sha256": digest(binary),
                        "remaining": [
                            *([] if signing else ["production signing"]),
                            "consumer-platform authentication E2E",
                            "independent release review",
                        ],
                    },
                    indent=2,
                )
                + "\n"
            )
        # Ignored local files can contain secrets. Bundle only reviewed source.
        tracked = subprocess.check_output(
            [
                "git",
                "ls-files",
                "-z",
                "--",
                "validation",
                "administration",
                "mcp-scanner",
                "gateway-identity",
                "gateway-compatibility",
            ],
            cwd=repo,
        )
        for filename in tracked.decode().split("\0"):
            if not filename:
                continue
            source = repo / filename
            if source.is_symlink() or not source.resolve().is_relative_to(repo):
                raise ValueError(
                    "Bundle source must be a regular file inside the repository"
                )
            destination = root / filename
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
        (root / "scripts").mkdir()
        for filename in [
            "test_airs_harness.py",
            "test_airs_harness_pty.py",
            "airs_harness_pty.py",
            "validate_live_agent.py",
            "validate_live_model_switch.py",
            "validate_live_gateway.py",
            "validate_mcp_sessions.py",
            "verify_airs_release.py",
            "check_airs_endpoints.py",
            "validate_auth_cli.py",
            "validate_auth_cli.py.lock",
            "validate_rust_oidc.py",
            "validate_rust_oidc.py.lock",
            "validate_native_credentials.py",
            "airs_oidc_interactive.py",
            "verify_persisted_airs_audit.py",
            "verify_management_airs_audit.mjs",
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
                        record["notices"].append(relative.as_posix())
            inventory.append(record)
        (root / "DEPENDENCIES.json").write_text(
            json.dumps(
                {
                    "scope": f"resolved {args.target} normal/build dependency closure; includes build-only packages",
                    "rust_distribution_notices": "licenses/rust-toolchain; includes standard-library and build-tool notices",
                    "packages": inventory,
                },
                indent=2,
            )
            + "\n"
        )
        provenance = {
            "product": "Prisma AIRS Harness",
            "version": version,
            "source_commit": commit,
            "source_repository": "https://github.com/cdot65/airs-harness",
            "upstream_commit": "3d2ee51ca2d5db578f328aa75e20aa22c0197c9a",
            "target": args.target,
            "profile": args.profile,
            "binary_processing": args.binary_processing,
            "binary_sha256": digest(root / binary_name),
            "cargo_lock_sha256": digest(repo / "codex-rs/Cargo.lock"),
            "rust": subprocess.check_output(
                ["rustc", "--version"], cwd=repo / "codex-rs", text=True
            ).strip(),
            "build_command": args.build_command,
        }
        if args.unvalidated_candidate:
            provenance["release_status"] = candidate_status
            provenance["publishable"] = False
        if signing is not None:
            provenance["signing_receipt_sha256"] = digest(root / "SIGNING.json")
        if args.validation:
            provenance["validation_receipt_sha256"] = digest(root / "VALIDATION.json")
            provenance["release_scope"] = validation["scope"]
        (root / "BUILD-INFO.json").write_text(json.dumps(provenance, indent=2) + "\n")
        files = sorted(path for path in root.rglob("*") if path.is_file())
        (root / "SHA256SUMS").write_text(
            "".join(
                f"{digest(path)}  {path.relative_to(root).as_posix()}\n"
                for path in files
            ),
            newline="\n",
        )
        with tarfile.open(archive, "w:gz") as tar:
            tar.add(root, arcname=name)
    archive.with_suffix(archive.suffix + ".sha256").write_text(
        f"{digest(archive)}  {archive.name}\n", newline="\n"
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
