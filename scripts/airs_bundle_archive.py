"""Bounded, integrity-checked npm archive handling for the managed CLI bundle."""

import base64
import gzip
import hashlib
import io
import json
from pathlib import PurePosixPath
import re
import tarfile
import unicodedata
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

MAX_ARCHIVE = 64 * 1024 * 1024
MAX_EXTRACTED = 128 * 1024 * 1024
MAX_METADATA = 16 * 1024 * 1024
MAX_MEMBERS = 20000
WINDOWS_RESERVED = {
    "CON",
    "PRN",
    "AUX",
    "NUL",
    *(f"{prefix}{n}" for prefix in ("COM", "LPT") for n in "123456789¹²³"),
}


def safe_path(value):
    """Reject POSIX and Windows escape spellings before path normalization."""
    if (
        not isinstance(value, str)
        or not value
        or len(value) > 4096
        or any(ord(char) < 32 or ord(char) == 127 for char in value)
        or "\\" in value
        or ":" in value
        or any(char in value for char in '<>"|?*')
        or any(part in ("", ".", "..") for part in value.split("/"))
        or any(
            part.endswith((".", " "))
            or part.split(".")[0].rstrip(" .").upper() in WINDOWS_RESERVED
            for part in value.split("/")
        )
        or PurePosixPath(value).is_absolute()
    ):
        raise ValueError("Unsafe bundle member path")
    return PurePosixPath(value)


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *_args, **_kwargs):
        raise ValueError("Dependency archive redirects are prohibited")


def download(url):
    if (
        not isinstance(url, str)
        or not url.startswith("https://registry.npmjs.org/")
        or any(ord(char) < 33 for char in url)
    ):
        raise ValueError("Invalid dependency archive URL")
    parsed = urlsplit(url)
    if (
        parsed.scheme != "https"
        or parsed.netloc != "registry.npmjs.org"
        or parsed.query
        or parsed.fragment
        or not re.fullmatch(r"/[A-Za-z0-9@_./+\-]+\.tgz", parsed.path)
    ):
        raise ValueError("Dependency archives require the public npm HTTPS registry")
    safe_path(parsed.path[1:])
    request = Request(url, headers={"User-Agent": "Prisma-AIRS-Harness-Packager"})
    with build_opener(NoRedirect()).open(request, timeout=60) as response:
        if response.geturl() != url:
            raise ValueError("Dependency archive destination changed")
        body = response.read(MAX_ARCHIVE + 1)
    if len(body) > MAX_ARCHIVE:
        raise ValueError("Dependency archive download exceeds limit")
    return body


def unpack(
    blob,
    integrity,
    destination,
    name,
    version,
    remaining,
    *,
    remaining_members=MAX_MEMBERS,
    remaining_path_bytes=MAX_METADATA,
):
    """Preflight every member and identity before allocating extracted files."""
    try:
        algorithm, encoded = integrity.split("-", 1)
        expected = base64.b64decode(encoded, validate=True)
    except (ValueError, AttributeError):
        raise ValueError("Invalid dependency integrity") from None
    if (
        algorithm != "sha512"
        or len(blob) > MAX_ARCHIVE
        or hashlib.sha512(blob).digest() != expected
    ):
        raise ValueError("Dependency archive integrity or size mismatch")
    limit = min(remaining, MAX_EXTRACTED)
    if limit < 0:
        raise ValueError("Bundle exceeds aggregate extraction limit")
    # Bound decompression, including PAX/long-name metadata, before tarfile can
    # allocate from attacker-controlled header sizes. No unbounded getmembers.
    with gzip.GzipFile(fileobj=io.BytesIO(blob)) as compressed:
        decoded = compressed.read(limit + MAX_METADATA + 1)
    if len(decoded) > limit + MAX_METADATA:
        raise ValueError("Dependency archive decompression exceeds limit")
    with tarfile.open(fileobj=io.BytesIO(decoded), mode="r:") as archive:
        members, seen, directories, files = [], set(), set(), set()
        root, size, path_bytes = None, 0, 0
        for member in archive:
            if len(members) >= min(MAX_MEMBERS, remaining_members) or member.size < 0:
                raise ValueError("Dependency member count or size exceeds limit")
            if (
                member.type not in (tarfile.REGTYPE, tarfile.AREGTYPE, tarfile.DIRTYPE)
                or member.sparse
            ):
                raise ValueError(
                    "Dependency links, sparse files and devices are prohibited"
                )
            raw = (
                member.name[:-1]
                if member.isdir() and member.name.endswith("/")
                else member.name
            )
            path = safe_path(raw)
            path_bytes += len(raw.encode("utf-8"))
            if path_bytes > remaining_path_bytes:
                raise ValueError("Bundle member paths exceed aggregate limit")
            if root is None:
                root = path.parts[0]
            canonical = PurePosixPath(
                unicodedata.normalize("NFC", str(path)).casefold()
            )
            if path.parts[0] != root or canonical in seen:
                raise ValueError("Duplicate member or multiple archive roots")
            if any(parent in files for parent in canonical.parents) or (
                member.isfile() and canonical in directories
            ):
                raise ValueError("Conflicting archive file and directory paths")
            if len(path.parts) == 1 and not member.isdir():
                raise ValueError("Archive files require a containing root directory")
            size += member.size
            if size > limit:
                raise ValueError("Bundle exceeds aggregate extraction limit")
            seen.add(canonical)
            directories.update(canonical.parents)
            (directories if member.isdir() else files).add(canonical)
            members.append((member, path))
        metadata = next(
            (m for m, p in members if p.parts == (root, "package.json") and m.isfile()),
            None,
        )
        if metadata is None or metadata.size > 1024 * 1024:
            raise ValueError("Missing or oversized dependency manifest")
        manifest = json.load(archive.extractfile(metadata))
        if not isinstance(manifest, dict) or (
            manifest.get("name"),
            manifest.get("version"),
        ) != (name, version):
            raise ValueError("Dependency manifest name/version differs from lock")
        destination.mkdir(parents=True, exist_ok=False)
        hashes, licenses = {}, []
        for member, path in members:
            relative = PurePosixPath(*path.parts[1:])
            target = destination.joinpath(*relative.parts)
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            target.parent.mkdir(parents=True, exist_ok=True)
            payload = archive.extractfile(member).read(member.size + 1)
            if len(payload) != member.size:
                raise ValueError("Truncated dependency member")
            target.write_bytes(payload)
            target.chmod(member.mode & 0o755)
            label = str(relative)
            is_license = any(
                term in label.lower()
                for term in ("license", "licence", "notice", "copying", "copyright")
            )
            # npm pack omits these non-runtime source files in the locked tree.
            if relative.name not in ("CHANGELOG.md", "yarn.lock") or is_license:
                hashes[label] = hashlib.sha256(payload).hexdigest()
            if is_license:
                licenses.append(label)
    return {
        "name": name,
        "version": version,
        "integrity": integrity,
        "archive_sha256": hashlib.sha256(blob).hexdigest(),
        "license": manifest.get("license"),
        "files": dict(sorted(hashes.items())),
        "license_files": sorted(licenses),
        "unpacked_bytes": size,
        "member_count": len(members),
        "path_bytes": path_bytes,
    }
