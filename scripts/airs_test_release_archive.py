"""Streaming USTAR inspection and metadata-only rewriting of native npm packages."""

import base64
import copy
import gzip
import hashlib
import io
from pathlib import Path
import tarfile
import unicodedata

from airs_bundle_archive import safe_path
from airs_test_release_spec import (
    MAX_ARCHIVE,
    MAX_JSON,
    digest_file,
    json_bytes,
    regular_file,
    require,
)

MAX_DECODED = 1024 * 1024 * 1024
MAX_MEMBERS = 20000
MAX_PATH_BYTES = 16 * 1024 * 1024
METADATA = {
    "package/package.json",
    "package/BUILD-INFO.json",
    "package/SIGNING.json",
    "package/VALIDATION.json",
    "package/BUNDLE-INVENTORY.json",
    "package/PACKAGE-TOOLING.json",
    "package/validation-evidence/original-build-candidate.json",
}
# Shipped text whose content the release gate reads; captured like metadata, not parsed.
TEXT = {"package/scripts/prepare_airs_ubuntu.sh"}
MUTABLE = {
    "package/package.json",
    "package/BUILD-INFO.json",
    "package/VALIDATION.json",
    "package/validation-evidence/original-build-candidate.json",
}


class _Reader:
    def __init__(self, stream):
        self.stream, self.total = stream, 0

    def read(self, count):
        require(0 <= count <= 65536, "Unbounded archive read")
        data = self.stream.read(count)
        self.total += len(data)
        require(self.total <= MAX_DECODED, "Decoded archive exceeds limit")
        return data

    def exact(self, count):
        data = self.read(count)
        require(len(data) == count, "Truncated archive")
        return data


class _Payload:
    def __init__(self, reader, size, capture):
        self.reader, self.remaining = reader, size
        self.digest, self.captured = hashlib.sha256(), bytearray() if capture else None

    def read(self, count=-1):
        require(count >= 0, "Unbounded payload read")
        data = self.reader.exact(min(count, self.remaining, 65536))
        self.remaining -= len(data)
        self.digest.update(data)
        if self.captured is not None:
            self.captured.extend(data)
        return data

    def drain(self):
        while self.remaining:
            self.read(65536)


def _scan(path, visitor=None):
    result = {"members": {}, "metadata": {}, "json": {}, "text": {}}
    seen, required_dirs, path_bytes = {}, set(), 0
    with (
        regular_file(path, MAX_ARCHIVE) as raw,
        gzip.GzipFile(fileobj=raw, mode="rb") as compressed,
    ):
        reader = _Reader(compressed)
        while True:
            header = reader.exact(512)
            if header == bytes(512):
                require(
                    reader.exact(512) == bytes(512), "Missing second tar terminator"
                )
                while trailing := reader.read(65536):
                    require(not any(trailing), "Unexpected trailing archive payload")
                break
            require(
                header[257:265] == b"ustar\x0000", "Only USTAR archives are supported"
            )
            info = tarfile.TarInfo.frombuf(header, "utf-8", "strict")
            require(
                info.type in (tarfile.REGTYPE, tarfile.AREGTYPE, tarfile.DIRTYPE),
                "Archive links, extensions and special files are prohibited",
            )
            require(
                not info.linkname and 0 <= info.size <= MAX_ARCHIVE,
                "Invalid archive member",
            )
            require(info.mode & ~0o777 == 0, "Unsafe archive mode")
            directory = info.isdir()
            name = info.name.rstrip("/") if directory else info.name
            parts = safe_path(name).parts
            require(
                parts[0] == "package" and (len(parts) > 1 or directory),
                "Archive requires package root",
            )
            key = unicodedata.normalize("NFC", name).casefold()
            require(key not in seen, "Duplicate or colliding archive member")
            require(
                directory or key not in required_dirs,
                "File conflicts with member parent",
            )
            for length in range(1, len(parts)):
                parent = unicodedata.normalize(
                    "NFC", "/".join(parts[:length])
                ).casefold()
                require(seen.get(parent) != "file", "Member parent is a file")
                required_dirs.add(parent)
            seen[key] = "dir" if directory else "file"
            path_bytes += len(name.encode("utf-8"))
            require(
                len(seen) <= MAX_MEMBERS and path_bytes <= MAX_PATH_BYTES,
                "Archive inventory exceeds limit",
            )
            require(not directory or info.size == 0, "Directory contains payload")
            capture = name in METADATA or name in TEXT
            require(
                not capture or (not directory and info.size <= MAX_JSON),
                "Metadata exceeds limit",
            )
            payload = _Payload(reader, info.size, capture)
            if visitor:
                visitor(info, payload)
            else:
                payload.drain()
            require(payload.remaining == 0, "Archive visitor did not consume member")
            padding = (-info.size) % 512
            require(not any(reader.exact(padding)), "Nonzero archive member padding")
            result["members"][name] = {
                "size": info.size,
                "mode": info.mode,
                "type": "dir" if directory else "file",
                "sha256": payload.digest.hexdigest(),
            }
            if name in TEXT:
                result["text"][name] = bytes(payload.captured)
            elif capture:
                raw_metadata = bytes(payload.captured)
                result["metadata"][name] = raw_metadata
                result["json"][name] = json_bytes(raw_metadata)
    return result


def inspect_archive(path):
    """Inventory without extracting files or allocating native payloads."""
    before = digest_file(path)
    result = _scan(path)
    result["sha256"] = digest_file(path)
    require(result["sha256"] == before, "Archive changed during inspection")
    result["integrity"] = "sha512-" + base64.b64encode(
        bytes.fromhex(digest_file(path, "sha512"))
    ).decode("ascii")
    return result


def rewrite_archive(source, destination, replacements):
    """Change only bounded allowlisted metadata, preserving every other payload."""
    require(set(replacements) <= MUTABLE, "Only release metadata may change")
    for value in replacements.values():
        require(
            isinstance(value, bytes) and len(value) <= MAX_JSON,
            "Invalid replacement metadata",
        )
        json_bytes(value)
    source_inventory = inspect_archive(source)
    destination = Path(destination)
    remaining = dict(replacements)
    created = False
    try:
        with destination.open("xb") as raw:
            created = True
            with gzip.GzipFile(
                fileobj=raw, mode="wb", filename="", mtime=0
            ) as compressed:
                with tarfile.open(
                    fileobj=compressed, mode="w|", format=tarfile.USTAR_FORMAT
                ) as writer:

                    def visit(info, payload):
                        replacement = remaining.pop(info.name, None)
                        if replacement is not None:
                            payload.drain()
                            info = copy.copy(info)
                            info.size = len(replacement)
                            writer.addfile(info, io.BytesIO(replacement))
                        else:
                            writer.addfile(info, payload)

                    _scan(source, visit)
                    for name, content in remaining.items():
                        info = tarfile.TarInfo(name)
                        info.mode, info.size = 0o644, len(content)
                        writer.addfile(info, io.BytesIO(content))
        require(
            digest_file(source) == source_inventory["sha256"],
            "Source changed during rewrite",
        )
        result = inspect_archive(destination)

        def unchanged(inventory):
            return {
                name: entry
                for name, entry in inventory["members"].items()
                if name not in replacements
            }

        require(
            unchanged(result) == unchanged(source_inventory),
            "Runtime payload changed during staging",
        )
        return result
    except BaseException:
        if created:
            destination.unlink(missing_ok=True)
        raise
