"""Identity and bounded I/O for owner-authorized npm releases and explicit Mac-first previews."""

import hashlib
import json
import os
from pathlib import Path
import re
import stat
from urllib.parse import urlsplit

from airs_npm_versions import released_version

SCOPE = "owner-authorized-test"
MAC_SCOPE = "owner-authorized-mac-preview"
MAC_TAG = "mac-preview"
MAC_TARGET = "aarch64-apple-darwin"
STABLE_SCOPE = "owner-authorized-stable"
STABLE_TAG = "stable-candidate"
STABLE_VERSION = re.compile(r"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\Z")
TARGETS = {
    "x86_64-unknown-linux-musl": "airs-harness-linux-x64",
    "aarch64-unknown-linux-musl": "airs-harness-linux-arm64",
    "aarch64-apple-darwin": "airs-harness-darwin-arm64",
}
PACKAGE_ORDER = [*TARGETS.values(), "airs-harness"]
MAX_ARCHIVE = 512 * 1024 * 1024
MAX_JSON = 1024 * 1024
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
COMMIT = re.compile(r"[0-9a-f]{40}\Z")
VERSION = re.compile(
    r"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)-alpha\.(?:0|[1-9]\d*)\.mcp\.(?:0|[1-9]\d*)\Z"
)


def release_targets(spec):
    """Mac-first previews cannot relax the ordinary three-platform release gate."""
    return (MAC_TARGET,) if spec.get("scope") == MAC_SCOPE else tuple(TARGETS)


def package_order(spec):
    return [*(TARGETS[target] for target in release_targets(spec)), "airs-harness"]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def canonical_digest(document):
    return hashlib.sha256(
        json.dumps(
            document, sort_keys=True, separators=(",", ":"), allow_nan=False
        ).encode()
    ).hexdigest()


def regular_file(path, limit=MAX_ARCHIVE):
    """Open a bounded regular file without following its final symlink."""
    path = Path(path)
    before = path.lstat()
    require(
        stat.S_ISREG(before.st_mode)
        and not getattr(before, "st_file_attributes", 0) & 0x400,
        "Expected a regular non-linked file",
    )
    require(before.st_size <= limit, "File exceeds size limit")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0)
    stream = os.fdopen(os.open(path, flags), "rb")
    current = os.fstat(stream.fileno())
    if not stat.S_ISREG(current.st_mode) or (
        before.st_dev,
        before.st_ino,
        before.st_size,
    ) != (current.st_dev, current.st_ino, current.st_size):
        stream.close()
        raise ValueError("File changed while opening")
    return stream


def digest_file(path, algorithm="sha256", limit=MAX_ARCHIVE):
    require(algorithm in ("sha256", "sha512"), "Unsupported digest algorithm")
    digest = hashlib.new(algorithm)
    size = 0
    with regular_file(path, limit) as stream:
        while chunk := stream.read(65536):
            size += len(chunk)
            require(size <= limit, "File exceeded read limit")
            digest.update(chunk)
    return digest.hexdigest()


def json_bytes(payload):
    require(len(payload) <= MAX_JSON, "JSON exceeds size limit")

    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "Duplicate JSON field")
            result[key] = value
        return result

    return json.loads(
        payload,
        object_pairs_hook=unique,
        parse_constant=lambda _: (_ for _ in ()).throw(
            ValueError("Nonfinite JSON value")
        ),
    )


def load_json(path):
    with regular_file(path, MAX_JSON) as stream:
        payload = stream.read(MAX_JSON + 1)
    return json_bytes(payload)


def validate_spec(document):
    require(isinstance(document, dict), "Expected release specification")
    previous = released_version(document.get("previous_version"))
    fields = {
        "schema_version",
        "scope",
        "version",
        "tag",
        "registry",
        "source_commit",
        "tooling_commit",
        "packaging_commit",
        "previous_version",
        "developer_id_team",
        "platforms",
    }
    if previous.stable:
        fields.add("previous_release")
    if "previous_registry" in document:
        fields.add("previous_registry")
    require(
        set(document) == fields, "Unexpected or missing release specification fields"
    )
    require(
        type(document["schema_version"]) is int and document["schema_version"] == 1,
        "Unsupported release specification schema",
    )
    require(
        (document["scope"], document["tag"])
        in ((SCOPE, "mcp"), (STABLE_SCOPE, STABLE_TAG), (MAC_SCOPE, MAC_TAG)),
        "Expected an explicitly authorized test or stable candidate scope",
    )
    for key in fields - {"schema_version", "platforms", "previous_release"}:
        value = document[key]
        require(
            isinstance(value, str)
            and 0 < len(value) <= 2048
            and all(32 <= ord(c) < 127 for c in value),
            "Invalid release specification string",
        )
    require(
        (STABLE_VERSION if document["scope"] == STABLE_SCOPE else VERSION).fullmatch(
            document["version"]
        )
        is not None
        and len(document["version"]) <= 128,
        "Version does not match the release scope",
    )
    require(
        document["previous_version"] != document["version"], "Invalid previous version"
    )
    if previous.stable:
        baseline = document["previous_release"]
        require(
            isinstance(baseline, dict)
            and set(baseline) == {"source_commit", "platforms"},
            "Invalid previous release declaration",
        )
        require(
            isinstance(baseline["source_commit"], str)
            and COMMIT.fullmatch(baseline["source_commit"]) is not None,
            "Invalid previous source commit",
        )
        rows = baseline["platforms"]
        require(
            isinstance(rows, list) and len(rows) == 3,
            "Previous release requires three native targets",
        )
        for row in rows:
            require(
                isinstance(row, dict)
                and set(row) == {"target", "binary_sha256"}
                and isinstance(row["target"], str)
                and row["target"] in TARGETS
                and isinstance(row["binary_sha256"], str)
                and SHA256.fullmatch(row["binary_sha256"]) is not None,
                "Invalid previous native target",
            )
        require(
            {row["target"] for row in rows} == set(TARGETS),
            "Duplicate previous native target",
        )
    for key in ("source_commit", "tooling_commit", "packaging_commit"):
        require(
            COMMIT.fullmatch(document[key]) is not None, "Expected a full source commit"
        )
    require(
        re.fullmatch(r"[A-Z0-9]{10}", document["developer_id_team"]) is not None,
        "Invalid Developer ID team",
    )
    for registry in {
        document["registry"],
        document.get("previous_registry", document["registry"]),
    }:
        parsed = urlsplit(registry)
        require(
            parsed.scheme == "https"
            and parsed.hostname
            and parsed.username is None
            and parsed.password is None
            and not any(c in registry for c in "\\?# ")
            and parsed.port != 0,
            "Registry requires HTTPS without credentials, query, fragment or whitespace",
        )
        require(
            not parsed.path
            or all(part not in (".", "..") for part in parsed.path.split("/")),
            "Invalid registry path",
        )
    platforms = document["platforms"]
    require(
        isinstance(platforms, list)
        and len(platforms) == len(release_targets(document)),
        "Native targets must exactly match the authorized release scope",
    )
    seen = set()
    for platform in platforms:
        require(
            isinstance(platform, dict) and set(platform) == {"target", "binary_sha256"},
            "Invalid platform declaration",
        )
        target, digest = platform["target"], platform["binary_sha256"]
        require(
            isinstance(target, str)
            and target in release_targets(document)
            and target not in seen,
            "Unsupported or duplicate native target",
        )
        require(
            isinstance(digest, str) and SHA256.fullmatch(digest) is not None,
            "Invalid native digest",
        )
        seen.add(target)
    # Copy and canonicalize ordering without mutating the caller's declaration.
    result = dict(document)
    result["registry"] = document["registry"].rstrip("/")
    if "previous_registry" in document:
        result["previous_registry"] = document["previous_registry"].rstrip("/")
    result["platforms"] = [
        dict(next(p for p in platforms if p["target"] == target))
        for target in release_targets(document)
    ]
    if previous.stable:
        result["previous_release"] = {
            "source_commit": baseline["source_commit"],
            "platforms": [
                dict(
                    next(
                        row for row in baseline["platforms"] if row["target"] == target
                    )
                )
                for target in TARGETS
            ],
        }
    return result


def load_spec(path):
    return validate_spec(load_json(path))
