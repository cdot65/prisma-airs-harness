"""Reject unlisted installed bundle contents, including undeclared native payloads."""

import json
import os
from pathlib import PurePosixPath
import re
import stat
import unicodedata

from airs_bundle_archive import safe_path
from airs_bundle_shims import EXTENSIONS, verify_shims


def verify_tree(root, inventory, manifests, read_file):
    expected, bins = set(), {}
    for package in inventory["packages"]:
        base = PurePosixPath(package["path"])
        expected.update(
            str(base / name)
            for name in {**package["files"], **package.get("optional_files", {})}
        )
        commands = manifests[str(base)].get("bin", {})
        if isinstance(commands, str):
            commands = {package["name"].split("/")[-1]: commands}
        bin_root = base.parts[: len(base.parts) - len(package["name"].split("/"))]
        for name, relative in commands.items():
            if len(safe_path(name).parts) != 1:
                raise ValueError("Invalid bundled command name")
            # npm package bin paths commonly carry an innocuous './' prefix.
            relative = relative.removeprefix("./")
            target = str(base / safe_path(relative))
            if target not in expected and relative not in package["files"]:
                raise ValueError("Bundled command target lacks file evidence")
            link = str(PurePosixPath(*bin_root) / ".bin" / name)
            if link in bins and bins[link] != target:
                raise ValueError("Bundled command names conflict")
            bins[link] = target

    # Harness native packages are verified separately against BUILD-INFO by the
    # release validator; exclude only exact, explicitly declared native packages.
    excluded = set()
    if (root / "package.json").exists():
        launcher = json.loads(read_file(root / "package.json", 1024 * 1024))
        for name, version in launcher.get("optionalDependencies", {}).items():
            if re.fullmatch(
                r"(?:@cdot65/prisma-)?airs-harness-(?:linux-(?:x64|arm64)|darwin-arm64|win32-(?:x64|arm64))",
                name,
            ):
                if version != launcher.get("version"):
                    raise ValueError(
                        "Harness native dependency must be an exact matching version"
                    )
                excluded.add("node_modules/" + name)

    wrappers = {}
    wrapper_names = {unicodedata.normalize("NFC", path).casefold() for path in expected}
    for command, target in bins.items():
        for suffix in EXTENSIONS:
            path = command + suffix
            canonical = unicodedata.normalize("NFC", path).casefold()
            if canonical in wrapper_names:
                raise ValueError("Bundled command wrapper paths conflict")
            wrapper_names.add(canonical)
            wrappers[path] = (command, target)
    generated = {}
    seen, count, path_bytes = set(), 0, 0
    pending = [root / "node_modules"]
    while pending:
        directory = pending.pop()
        if (
            directory.is_symlink()
            or getattr(directory.lstat(), "st_file_attributes", 0) & 0x400
        ):
            raise ValueError("Bundled dependency directories cannot be symbolic links")
        with os.scandir(directory) as entries:
            for entry in entries:
                path = directory / entry.name
                relative = path.relative_to(root).as_posix()
                safe_path(relative)
                count += 1
                path_bytes += len(relative.encode())
                if count > 25000 or path_bytes > 4 * 1024 * 1024:
                    raise ValueError("Installed bundle tree exceeds enumeration limit")
                canonical = unicodedata.normalize("NFC", relative).casefold()
                if canonical in seen:
                    raise ValueError("Installed bundle paths collide across platforms")
                seen.add(canonical)
                metadata = entry.stat(follow_symlinks=False)
                if getattr(metadata, "st_file_attributes", 0) & 0x400:
                    raise ValueError("Bundled entries cannot be Windows reparse points")
                mode = metadata.st_mode
                if relative in excluded:
                    if not stat.S_ISDIR(mode):
                        raise ValueError(
                            "Harness native dependency must be a real directory"
                        )
                    continue
                if stat.S_ISLNK(mode) and relative in bins:
                    if path.resolve(strict=True) != (root / bins[relative]).resolve(
                        strict=True
                    ):
                        raise ValueError(
                            "Generated bundled command link differs from declared target"
                        )
                elif relative in wrappers:
                    if not stat.S_ISREG(mode):
                        raise ValueError(
                            "Generated bundled command wrapper must be a regular file"
                        )
                    command, target = wrappers[relative]
                    generated[command] = target
                elif stat.S_ISDIR(mode):
                    pending.append(path)
                elif stat.S_ISREG(mode) and relative in expected:
                    continue
                else:
                    raise ValueError("Unexpected installed bundle entry: " + relative)
    return verify_shims(root, generated, read_file) if generated else []
