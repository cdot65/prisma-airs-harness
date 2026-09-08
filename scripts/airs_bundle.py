"""Build and verify the locked Prisma AIRS CLI dependency bundle; never publish."""

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import tempfile

from airs_bundle_archive import MAX_EXTRACTED, download, safe_path, unpack
from airs_bundle_tree import verify_tree

CLI = "@cdot65/prisma-airs-cli"
SDK = "@cdot65/prisma-airs-sdk"
MAX_TOTAL = 256 * 1024 * 1024
MAX_FILES = 20000
MAX_PATH_BYTES = 2 * 1024 * 1024
NAME = r"(?:@[a-z0-9][a-z0-9._-]*/)?[a-z0-9][a-z0-9._-]*"
TARGETS = {
    "x86_64-unknown-linux-musl": ("linux", "x64"),
    "aarch64-unknown-linux-musl": ("linux", "arm64"),
    "aarch64-apple-darwin": ("darwin", "arm64"),
    "x86_64-pc-windows-msvc": ("win32", "x64"),
    "aarch64-pc-windows-msvc": ("win32", "arm64"),
}


def read_file(path, limit):
    if path.is_symlink() or getattr(path.lstat(), "st_file_attributes", 0) & 0x400:
        raise ValueError("Bundle inputs must be regular files")
    descriptor = os.open(
        path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0)
    )
    with os.fdopen(descriptor, "rb") as stream:
        metadata = os.fstat(stream.fileno())
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_size > limit
            or getattr(metadata, "st_file_attributes", 0) & 0x400
        ):
            raise ValueError("Bundle input is not a bounded regular file")
        content = stream.read(limit + 1)
    if len(content) > limit:
        raise ValueError("Bundle input exceeds limit")
    return content


def sharp_packages(targets):
    if not targets or len(set(targets)) != len(targets):
        raise ValueError("Bundle requires distinct supported native targets")
    selected = set()
    for target in targets:
        if target not in TARGETS:
            raise ValueError("Unsupported native target for CLI bundle: " + target)
        platform, arch = TARGETS[target]
        variants = [f"{platform}-{arch}"]
        if platform == "linux":
            variants.append(f"linuxmusl-{arch}")
        for variant in variants:
            selected.add("@img/sharp-" + variant)
            if platform != "win32":
                selected.add("@img/sharp-libvips-" + variant)
    return selected


def bundle_cli(lock_path, launcher, targets):
    wanted = sharp_packages(targets)
    raw = read_file(lock_path, 8 * 1024 * 1024)
    lock = json.loads(raw)
    packages = lock.get("packages", {})
    if lock.get("lockfileVersion") != 3 or not 1 <= len(packages) <= 512:
        raise ValueError("Bundle requires a bounded npm v3 lockfile")
    for path in packages:
        if path and (
            not re.fullmatch(rf"node_modules/{NAME}(?:/node_modules/{NAME})*", path)
            or not safe_path(path)
        ):
            raise ValueError("Invalid dependency path in lock")

    def resolve(owner, name):
        if not re.fullmatch(NAME, name):
            raise ValueError("Invalid dependency name in lock")
        for prefix in [PurePosixPath(owner), *PurePosixPath(owner).parents]:
            if prefix.name == "node_modules":
                continue
            candidate = str(prefix / "node_modules" / name)
            if candidate in packages:
                return candidate
        raise ValueError("Missing locked dependency: " + name)

    cli = resolve("", CLI)
    sdk = resolve(cli, SDK)
    manifest = json.loads(read_file(launcher / "package.json", 1024 * 1024))
    pins = {CLI: packages[cli]["version"], SDK: packages[sdk]["version"]}
    if (
        manifest.get("dependencies", {}).get(CLI) != pins[CLI]
        or packages[""].get("dependencies", {}).get(CLI) != pins[CLI]
        or packages[cli].get("dependencies", {}).get(SDK) != pins[SDK]
    ):
        raise ValueError("Managed CLI and SDK must have matching literal lock pins")
    pending = [cli, *(resolve(resolve(cli, "sharp"), name) for name in wanted)]
    selected = set()
    while pending:
        path = pending.pop()
        if path in selected:
            continue
        selected.add(path)
        record = packages[path]
        if record.get("link"):
            raise ValueError("Linked dependencies cannot be bundled")
        required_peers = {
            name: version
            for name, version in record.get("peerDependencies", {}).items()
            if not record.get("peerDependenciesMeta", {}).get(name, {}).get("optional")
        }
        for name in {
            **record.get("dependencies", {}),
            **record.get("optionalDependencies", {}),
            **required_peers,
        }:
            if name.startswith("@img/sharp-") and name not in wanted:
                continue
            pending.append(resolve(path, name))
    if (launcher / "node_modules").exists() or (launcher / "node_modules").is_symlink():
        raise ValueError("Bundle destination already contains dependencies")
    records, remaining, members, paths = [], MAX_TOTAL, MAX_FILES, MAX_PATH_BYTES
    with tempfile.TemporaryDirectory(
        prefix=".airs-bundle-", dir=launcher.parent
    ) as temporary:
        staging = Path(temporary)
        for path in sorted(selected):
            locked = packages[path]
            result = unpack(
                download(locked["resolved"]),
                locked["integrity"],
                staging / path,
                path.rsplit("node_modules/", 1)[-1],
                locked["version"],
                remaining,
                remaining_members=members,
                remaining_path_bytes=paths,
            )
            remaining -= result["unpacked_bytes"]
            members -= result["member_count"]
            paths -= result["path_bytes"]
            records.append({"path": path, "resolved": locked["resolved"], **result})
        inventory = {
            "schema_version": 1,
            "source_lock_sha256": hashlib.sha256(raw).hexdigest(),
            "targets": sorted(targets),
            "required_pins": pins,
            "packages": records,
        }
        verify_bundle(staging, inventory)
        encoded = json.dumps(inventory, sort_keys=True, indent=2) + "\n"
        if len(encoded.encode()) > 8 * 1024 * 1024:
            raise ValueError("Bundle inventory exceeds limit")
        (staging / "node_modules").rename(launcher / "node_modules")
        (launcher / "BUNDLE-INVENTORY.json").write_text(encoded)
    return inventory


def verify_bundle(directory, inventory):
    """Check actual packed/installed manifests, runtime files, native payloads and licenses."""
    if (
        inventory.get("schema_version") != 1
        or not 1 <= len(inventory.get("packages", [])) <= 512
    ):
        raise ValueError("Invalid bundle inventory")
    if set(inventory.get("required_pins", {})) != {CLI, SDK} or not re.fullmatch(
        "[0-9a-f]{64}", inventory.get("source_lock_sha256", "")
    ):
        raise ValueError("Bundle pin or lock evidence missing")
    wanted = sharp_packages(inventory["targets"])
    root = directory.resolve(strict=True)
    found, files, licenses = {}, 0, 0
    for package in inventory["packages"]:
        path = safe_path(package["path"])
        if (
            not re.fullmatch(
                rf"node_modules/{NAME}(?:/node_modules/{NAME})*", str(path)
            )
            or str(path) in found
        ):
            raise ValueError("Invalid or duplicate inventory package path")
        base = root.joinpath(*path.parts)
        if not base.resolve().is_relative_to(root):
            raise ValueError("Bundle package escapes installation directory")
        actual = json.loads(read_file(base / "package.json", 1024 * 1024))
        if (actual.get("name"), actual.get("version")) != (
            package["name"],
            package["version"],
        ):
            raise ValueError("Installed bundle package identity differs")
        found[str(path)] = actual
        if "package.json" not in package["files"] or not set(
            package["license_files"]
        ).issubset(package["files"]):
            raise ValueError("Bundle manifest or license evidence missing")
        optional = package.get("optional_files", {})
        if set(optional) & set(package["files"]) or any(
            PurePosixPath(name).name not in ("CHANGELOG.md", "yarn.lock")
            for name in optional
        ):
            raise ValueError("Invalid optional bundle file evidence")
        transforms = {item["path"]: item for item in package.get("normalizations", [])}
        commands = actual.get("bin", {})
        declared_bins = (
            [commands] if isinstance(commands, str) else list(commands.values())
        )
        if len(transforms) != len(package.get("normalizations", [])) or not set(
            transforms
        ).issubset({value.removeprefix("./") for value in declared_bins}):
            raise ValueError("Normalization requires a unique declared executable")
        for relative, expected in {**package["files"], **optional}.items():
            file = base.joinpath(*safe_path(relative).parts)
            if not file.resolve().is_relative_to(base.resolve()) or not re.fullmatch(
                "[0-9a-f]{64}", expected
            ):
                raise ValueError("Invalid bundle file evidence")
            if relative in optional and not file.exists() and not file.is_symlink():
                continue
            payload = read_file(file, MAX_EXTRACTED)
            if hashlib.sha256(payload).hexdigest() != expected:
                raise ValueError(
                    "Installed bundle file differs: " + str(path / relative)
                )
            if relative in transforms:
                change = transforms[relative]
                newline = payload.find(b"\n")
                if (
                    change.get("transform") != "npm-bin-shebang-crlf-to-lf-v1"
                    or change.get("normalized_sha256") != expected
                    or not payload.startswith(b"#!")
                    or newline < 0
                    or payload[newline - 1 : newline] == b"\r"
                    or hashlib.sha256(
                        payload[:newline] + b"\r" + payload[newline:]
                    ).hexdigest()
                    != change.get("original_sha256")
                ):
                    raise ValueError("Bundle shebang normalization provenance differs")
            files += 1
        licenses += len(package["license_files"])
    verify_tree(root, inventory, found, read_file)
    actual_names = {p["name"] for p in found.values()}
    if not wanted.issubset(actual_names) or any(
        name.startswith("@img/sharp-") and name not in wanted for name in actual_names
    ):
        raise ValueError(
            "Bundled native image payload matrix is incomplete or unsupported"
        )
    for name, version in inventory["required_pins"].items():
        if not any(
            p["name"] == name and p["version"] == version for p in found.values()
        ):
            raise ValueError("Required bundled CLI/SDK pin missing")
    return {
        "packages": len(found),
        "files": files,
        "license_files": licenses,
        "native_payload_packages": sorted(wanted),
        "source_lock_sha256": inventory["source_lock_sha256"],
    }
