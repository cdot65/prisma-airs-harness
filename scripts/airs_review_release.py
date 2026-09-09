"""Fail-closed evidence contract for a signed review prerelease, not production.

This module validates supplied attestations and their retained evidence bytes.
It does not manufacture evidence, assign a score, verify Apple's service, or
publish packages. Callers must collect the stated checks on the exact binaries.
"""

import hashlib
import math
import os
from pathlib import Path
import re
import stat
import uuid

from airs_bundle_archive import safe_path

SCOPE = "signed-prerelease-distribution"
TARGETS = {"x86_64-unknown-linux-musl", "aarch64-apple-darwin"}
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
COMMIT = re.compile(r"[0-9a-f]{40}\Z")
PRERELEASE = re.compile(r"[0-9]+\.[0-9]+\.[0-9]+-(?:alpha|beta|rc)\.[0-9]+\Z")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def evidence_digest(root, relative):
    """Hash bounded regular evidence without following links or Windows reparse points."""
    require(isinstance(relative, str), "Evidence path must be a string")
    parts = safe_path(relative).parts
    path = Path(root)
    for part in (None, *parts):
        if part is not None:
            path /= part
        metadata = path.lstat()
        require(
            not stat.S_ISLNK(metadata.st_mode)
            and not getattr(metadata, "st_file_attributes", 0) & 0x400,
            "Evidence cannot use linked or reparse paths",
        )
        if path != Path(root).joinpath(*parts):
            require(
                stat.S_ISDIR(metadata.st_mode), "Evidence parent must be a directory"
            )
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0)
    with os.fdopen(os.open(path, flags), "rb") as stream:
        metadata = os.fstat(stream.fileno())
        require(
            stat.S_ISREG(metadata.st_mode)
            and not getattr(metadata, "st_file_attributes", 0) & 0x400
            and metadata.st_size <= 16 * 1024 * 1024,
            "Evidence must be a bounded regular file",
        )
        # Reconcile the opened file with the checked path; never trust pre-open size alone.
        current = path.lstat()
        require(
            (current.st_dev, current.st_ino) == (metadata.st_dev, metadata.st_ino)
            and not getattr(current, "st_file_attributes", 0) & 0x400,
            "Evidence changed while opening",
        )
        digest = hashlib.sha256()
        remaining = 16 * 1024 * 1024 + 1
        while chunk := stream.read(min(65536, remaining)):
            remaining -= len(chunk)
            require(remaining > 0, "Evidence exceeded its read limit")
            digest.update(chunk)
        return digest.hexdigest()


def validate_release(
    document, *, binary_sha256, target, version, source_commit, evidence_root
):
    """Validate exact binary/source bindings and evidence for the review-only channel."""
    require(isinstance(document, dict), "Expected review-release object")
    require(
        target in TARGETS and PRERELEASE.fullmatch(version) is not None,
        "Only Linux x64 and Apple Silicon prereleases are supported",
    )
    require(
        SHA256.fullmatch(binary_sha256) is not None
        and COMMIT.fullmatch(source_commit) is not None,
        "Invalid release identity",
    )
    require(
        document.get("schema_version") == 1
        and type(document.get("schema_version")) is int,
        "Unsupported review-release schema",
    )
    require(
        document.get("scope") == SCOPE
        and document.get("passed") is True
        and document.get("release_ready") is True
        and document.get("full_authentication_release_ready") is False,
        "A passing review-only release attestation is required",
    )
    require(
        not any(
            key in document
            for key in (
                "release_status",
                "private",
                "private_candidate",
                "unsigned_candidate",
            )
        ),
        "Candidate markers cannot be relabeled as review-release evidence",
    )
    expected = {
        "binary_sha256": binary_sha256,
        "target": target,
        "product_version": version,
        "source_commit": source_commit,
    }
    require(
        all(document.get(key) == value for key, value in expected.items()),
        "Review-release evidence does not match binary, target, version or source",
    )
    review = document.get("independent_review")
    require(isinstance(review, dict), "Independent review is required")
    score = review.get("score")
    require(
        type(score) in (int, float)
        and math.isfinite(score)
        and 9 <= score <= 10
        and review.get("scope") == SCOPE
        and review.get("verdict") == "pass"
        and isinstance(review.get("reviewer"), str)
        and 0 < len(review["reviewer"]) <= 128
        and not any(ord(c) < 32 or ord(c) == 127 for c in review["reviewer"]),
        "Independent scoped review must meet the 9/10 threshold",
    )
    evidence = document.get("evidence")
    require(
        isinstance(evidence, list) and 3 <= len(evidence) <= 16,
        "Bounded release evidence references are required",
    )
    roles = set()
    for record in evidence:
        require(
            isinstance(record, dict) and set(record) == {"role", "path", "sha256"},
            "Invalid release evidence reference",
        )
        role = record["role"]
        require(
            isinstance(role, str)
            and re.fullmatch(r"[a-z][a-z-]{0,63}", role) is not None
            and role not in roles
            and isinstance(record["sha256"], str)
            and SHA256.fullmatch(record["sha256"]) is not None,
            "Invalid or duplicate evidence role",
        )
        require(
            evidence_digest(evidence_root, record["path"]) == record["sha256"],
            "Release evidence checksum mismatch",
        )
        roles.add(role)
    required = {
        "installed-runtime",
        "package-integrity",
        "independent-review",
        "native-provenance",
    }
    signing = document.get("signing")
    require(
        isinstance(signing, dict), "Explicit platform signing disposition is required"
    )
    if target == "aarch64-apple-darwin":
        required |= {"developer-id-signature", "apple-notarization"}
        require(
            signing.get("kind") == "developer-id-application"
            and signing.get("verified") is True
            and signing.get("hardened_runtime") is True
            and signing.get("signed_binary_sha256") == binary_sha256
            and isinstance(signing.get("team_identifier"), str)
            and re.fullmatch(r"[A-Z0-9]{10}", signing["team_identifier"]) is not None,
            "Verified hardened Developer ID signature must bind the final executable",
        )
        notarization = signing.get("notarization")
        require(
            isinstance(notarization, dict)
            and notarization.get("status") == "Accepted"
            and isinstance(notarization.get("archive_sha256"), str)
            and SHA256.fullmatch(notarization["archive_sha256"]) is not None,
            "Accepted notarization of the retained signed archive is required",
        )
        try:
            submission = uuid.UUID(notarization["submission_id"])
        except (KeyError, ValueError, TypeError, AttributeError):
            raise ValueError("Invalid notarization submission identity") from None
        require(
            str(submission) == notarization["submission_id"],
            "Noncanonical notarization identity",
        )
    else:
        require(
            signing == {"kind": "linux-provenance-checksums"},
            "Linux release must explicitly use provenance and checksum verification",
        )
    require(required <= roles, "Required release evidence roles are missing")
    return {
        "scope": SCOPE,
        "passed": True,
        **expected,
        "evidence_roles_verified": sorted(roles),
        "full_authentication_release_ready": False,
    }
