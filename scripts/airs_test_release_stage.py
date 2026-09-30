"""Bind metadata-only npm test publication to retained native acceptance evidence."""

import hashlib
import json
from pathlib import Path
import shutil
import tempfile

from airs_release_acceptance import verify_acceptance_set
from airs_release_receipts import evidence_path, safe_destination
from airs_test_release_archive import inspect_archive, rewrite_archive
from airs_ubuntu_helper import HELPER_MEMBER, verify_ubuntu_helper
from airs_test_release_spec import (
    LAUNCHER,
    archive_filename,
    STABLE_SCOPE,
    TARGETS,
    package_order,
    canonical_digest,
    digest_file,
    load_json,
    regular_file,
    require,
    validate_spec,
)

MANIFEST = "package/package.json"
BUILD = "package/BUILD-INFO.json"
VALIDATION = "package/VALIDATION.json"
ORIGINAL = "package/validation-evidence/original-build-candidate.json"
PLATFORM_MANIFEST = {
    "x86_64-unknown-linux-musl": ("linux", "x64"),
    "aarch64-unknown-linux-musl": ("linux", "arm64"),
    "aarch64-apple-darwin": ("darwin", "arm64"),
}
RETAINED_METADATA = ("cli_bundle", "package_tooling", "required_dependencies")


def _bytes(value):
    return (
        json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def _candidate_records(spec, root):
    metadata = load_json(evidence_path(root, "NPM-PACKAGES.json"))
    require(isinstance(metadata, dict), "Invalid candidate metadata")
    require(
        metadata.get("source_commit") == spec["source_commit"]
        and metadata.get("packaging_commit") == spec["packaging_commit"],
        "Candidate source identity mismatch",
    )
    require(
        metadata.get("registry", "").rstrip("/") == spec["registry"],
        "Candidate registry mismatch",
    )
    require(
        all(
            isinstance(metadata.get(key), dict) and metadata[key]
            for key in RETAINED_METADATA
        ),
        "Missing candidate bundle, tooling or dependency identity",
    )
    records = metadata.get("publish_order")
    require(
        isinstance(records, list) and all(isinstance(row, dict) for row in records),
        "Invalid package inventory",
    )
    require(
        [row.get("name") for row in records] == package_order(spec),
        "Expected the scoped native packages followed by launcher",
    )
    for row in records:
        require(row.get("version") == spec["version"], "Candidate version mismatch")
        require(
            row.get("filename") == archive_filename(row["name"], spec["version"]),
            "Unexpected archive filename",
        )
    return metadata, records


def _package_identity(spec, row, inventory):
    manifest = inventory["json"].get(MANIFEST)
    require(
        isinstance(manifest, dict)
        and manifest.get("name") == row["name"]
        and manifest.get("version") == spec["version"],
        "Package manifest identity mismatch",
    )
    require(manifest.get("private") is True, "Expected a private unvalidated candidate")
    if row["name"] == LAUNCHER:
        require(
            manifest.get("optionalDependencies")
            == {name: spec["version"] for name in package_order(spec)[:-1]},
            "Launcher requires the scoped pinned native dependencies",
        )
        if spec["scope"] == "owner-authorized-mac-preview":
            require(
                manifest.get("os") == ["darwin"] and manifest.get("cpu") == ["arm64"],
                "Mac preview launcher must reject other platforms",
            )
        # The shipped Ubuntu helper must install and check this launcher, not a
        # previous name; the check reads the archived copy, never the repository.
        helper = inventory["text"].get(HELPER_MEMBER)
        require(helper is not None, "Launcher archive lacks the Ubuntu helper")
        try:
            verify_ubuntu_helper(helper, LAUNCHER, spec["version"])
        except ValueError as error:
            raise ValueError("Packaged Ubuntu helper names another launcher") from error
        return
    target = next(target for target, name in TARGETS.items() if name == row["name"])
    system, architecture = PLATFORM_MANIFEST[target]
    require(
        manifest.get("os") == [system] and manifest.get("cpu") == [architecture],
        "Native manifest platform mismatch",
    )
    info = inventory["json"].get(BUILD)
    require(isinstance(info, dict), "Missing native build identity")
    require(
        info.get("product") == "Prisma AIRS Harness", "Native build product mismatch"
    )
    require(
        info.get("source_commit") == spec["source_commit"]
        and info.get("version") == spec["version"]
        and info.get("target") == target,
        "Native build identity mismatch",
    )
    native = inventory["members"].get("package/bin/airs-harness", {})
    expected = next(
        p["binary_sha256"] for p in spec["platforms"] if p["target"] == target
    )
    require(
        native.get("type") == "file"
        and native.get("sha256") == info.get("binary_sha256") == expected
        and native.get("mode", 0) & 0o111,
        "Native payload identity mismatch",
    )
    require(
        VALIDATION not in inventory["members"] and ORIGINAL not in inventory["members"],
        "Candidate already contains validation provenance",
    )
    if target == "aarch64-apple-darwin":
        signing = inventory["json"].get("package/SIGNING.json")
        require(isinstance(signing, dict), "Missing Apple signing receipt")
        require(
            signing.get("target") == target
            and signing.get("source_commit") == spec["source_commit"]
            and signing.get("binary_sha256") == expected
            and signing.get("team_id") == spec["developer_id_team"],
            "Apple signing identity mismatch",
        )
        require(
            all(
                signing.get(key) is True
                for key in (
                    "codesign_verified",
                    "hardened_runtime",
                    "notarization_verified",
                )
            ),
            "Apple signing or notarization is unverified",
        )
        require(
            info.get("signing_receipt_sha256")
            == inventory["members"]["package/SIGNING.json"]["sha256"],
            "Apple signing receipt binding mismatch",
        )


def inspect_packages(spec, packages_dir):
    """Inspect the complete candidate package set and all native payload identities."""
    spec, root = validate_spec(spec), Path(packages_dir)
    metadata, records = _candidate_records(spec, root)
    inventories, summary = {}, {}
    for row in records:
        archive = evidence_path(root, "tarballs/" + row["filename"])
        inventory = inspect_archive(archive)
        require(
            inventory["sha256"] == row.get("sha256")
            and inventory["integrity"] == row.get("integrity"),
            "Candidate archive integrity mismatch",
        )
        _package_identity(spec, row, inventory)
        inventories[row["name"]] = inventory
        summary[row["name"]] = {
            key: row[key] for key in ("sha256", "integrity", "version", "filename")
        }
    return {
        "metadata": metadata,
        "publish_order": records,
        "inventories": inventories,
        "candidate_metadata_sha256": digest_file(
            evidence_path(root, "NPM-PACKAGES.json")
        ),
        "candidate_packages_sha256": canonical_digest(summary),
    }


def _bind_acceptance(spec, candidate, acceptance):
    require(
        acceptance.get("spec_sha256") == canonical_digest(spec),
        "Acceptance specification mismatch",
    )
    for key in ("source_commit", "tooling_commit", "packaging_commit", "version"):
        require(acceptance.get(key) == spec[key], "Acceptance source identity mismatch")
    for key in ("candidate_metadata_sha256", "candidate_packages_sha256"):
        require(
            acceptance.get(key) == candidate[key],
            "Acceptance candidate identity mismatch",
        )
    require(
        [
            {key: row[key] for key in ("target", "binary_sha256")}
            for row in acceptance["platforms"]
        ]
        == spec["platforms"],
        "Acceptance native identities mismatch",
    )


def _replacements(spec, row, inventory, acceptance):
    manifest = dict(inventory["json"][MANIFEST])
    manifest.pop("private")
    changes = {MANIFEST: _bytes(manifest)}
    if row["name"] == LAUNCHER:
        return changes
    info = dict(inventory["json"][BUILD])
    platform = next(p for p in acceptance["platforms"] if p["target"] == info["target"])
    receipt = {
        "schema_version": 1,
        "scope": spec["scope"],
        "publication_authorized": True,
        "authorization": "Owner authorized direct versioned npm test handoffs; stable promotion and attended production acceptance remain separate.",
        "product_version": spec["version"],
        "source_commit": spec["source_commit"],
        "target": info["target"],
        "binary_sha256": info["binary_sha256"],
        "spec_sha256": canonical_digest(spec),
        "acceptance": platform,
        "acceptance_sha256": canonical_digest(acceptance),
        "native_installed_acceptance": True,
        "passed": False,
        "release_ready": False,
        "production_sso_servicenow_acceptance": False,
        "full_gateway_lifecycle_passed": False,
        "independent_review_claimed": False,
        "limitations": [
            "Attended production SSO, workspace-key, and ServiceNow OAuth/tool checks are deferred to the owner.",
            "The owner's Ubuntu credential-store incident remains deferred; isolated fixtures do not close it.",
            "Scoped development checks do not claim a clean full Rust workspace; historical failures remain recorded separately.",
        ],
        "original_candidate_sha256": hashlib.sha256(
            inventory["metadata"][BUILD]
        ).hexdigest(),
    }
    if spec["scope"] == STABLE_SCOPE:
        receipt["authorization"] = (
            "Owner authorized the stable release after stabilization. These native "
            "receipts cover automated acceptance; attended acceptance and full-workspace "
            "classification are retained in the stable release record."
        )
        receipt["limitations"] = [
            "Native fixtures do not independently establish production account acceptance.",
            "Default-tag promotion requires fresh registry verification of all platforms.",
        ]
    changes[ORIGINAL] = inventory["metadata"][BUILD]
    changes[VALIDATION] = _bytes(receipt)
    info.pop("publishable", None)
    info.pop("release_status", None)
    info["release_scope"] = spec["scope"]
    info["validation_receipt_sha256"] = hashlib.sha256(changes[VALIDATION]).hexdigest()
    changes[BUILD] = _bytes(info)
    return changes


def _plan(spec, candidate, acceptance, records):
    return {
        "schema_version": 1,
        "published": False,
        "release_scope": spec["scope"],
        "production_release_ready": False,
        "spec_sha256": canonical_digest(spec),
        **{
            key: spec[key]
            for key in (
                "version",
                "tag",
                "registry",
                "source_commit",
                "tooling_commit",
                "packaging_commit",
            )
        },
        **{
            key: candidate[key]
            for key in ("candidate_metadata_sha256", "candidate_packages_sha256")
        },
        **{key: candidate["metadata"][key] for key in RETAINED_METADATA},
        "acceptance_sha256": canonical_digest(acceptance),
        "evidence_sha256": acceptance["evidence_sha256"],
        "publish_order": records,
    }


def _record(row, inventory, original, changes):
    return {
        **{key: row[key] for key in ("name", "version", "filename")},
        "sha256": inventory["sha256"],
        "integrity": inventory["integrity"],
        "original_candidate_sha256": original["sha256"],
        "changed_archive_entries": sorted(changes),
        "runtime_payload_unchanged": True,
    }


def _copy(source, destination):
    with regular_file(source) as incoming, Path(destination).open("xb") as outgoing:
        shutil.copyfileobj(incoming, outgoing, length=65536)


def stage_packages(spec, packages_dir, evidence_root, output_dir):
    """Create an immutable, self-verifiable stage only after all native acceptances required by the release scope."""
    spec = validate_spec(spec)
    source, output, evidence = (
        Path(packages_dir).resolve(),
        safe_destination(output_dir),
        Path(evidence_root).resolve(),
    )
    require(
        not output.exists() and not output.is_symlink(), "Stage output already exists"
    )
    resolved = output.resolve()
    require(
        all(
            resolved != item
            and resolved not in item.parents
            and item not in resolved.parents
            for item in (source, evidence)
        ),
        "Stage output overlaps an input",
    )
    acceptance = verify_acceptance_set(spec, evidence)
    candidate = inspect_packages(spec, source)
    _bind_acceptance(spec, candidate, acceptance)
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix=".airs-release-", dir=output.parent))
    try:
        (temporary / "candidates" / "tarballs").mkdir(parents=True)
        (temporary / "tarballs").mkdir()
        _copy(
            evidence_path(source, "NPM-PACKAGES.json"),
            temporary / "candidates" / "NPM-PACKAGES.json",
        )
        records = []
        for row in candidate["publish_order"]:
            original = candidate["inventories"][row["name"]]
            retained = temporary / "candidates" / "tarballs" / row["filename"]
            _copy(evidence_path(source, "tarballs/" + row["filename"]), retained)
            require(
                digest_file(retained) == original["sha256"],
                "Candidate changed while retaining",
            )
            changes = _replacements(spec, row, original, acceptance)
            staged = rewrite_archive(
                retained, temporary / "tarballs" / row["filename"], changes
            )
            records.append(_record(row, staged, original, changes))
        plan = _plan(spec, candidate, acceptance, records)
        (temporary / "NPM-PACKAGES.json").write_bytes(_bytes(plan))
        verify_staged(spec, temporary, evidence)
        require(
            not output.exists() and not output.is_symlink(),
            "Stage output appeared during staging",
        )
        temporary.rename(safe_destination(output))
        return plan
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)


def verify_staged(spec, packages_dir, acceptance_root):
    """Recompute stage identity and compare every payload with accepted originals."""
    spec, root = validate_spec(spec), Path(packages_dir)
    acceptance = verify_acceptance_set(spec, acceptance_root)
    candidates = root / "candidates"
    require(
        candidates.is_dir() and not candidates.is_symlink(),
        "Retained candidate directory is missing or linked",
    )
    candidate = inspect_packages(spec, candidates)
    _bind_acceptance(spec, candidate, acceptance)
    records = []
    for row in candidate["publish_order"]:
        original = candidate["inventories"][row["name"]]
        staged = inspect_archive(evidence_path(root, "tarballs/" + row["filename"]))
        changes = _replacements(spec, row, original, acceptance)
        require(
            set(staged["members"]) == set(original["members"]) | set(changes),
            "Staged inventory differs from accepted candidate",
        )
        for name, entry in staged["members"].items():
            if name in changes:
                mode = original["members"].get(name, {"mode": 0o644})["mode"]
                require(
                    entry
                    == {
                        "size": len(changes[name]),
                        "mode": mode,
                        "type": "file",
                        "sha256": hashlib.sha256(changes[name]).hexdigest(),
                    },
                    "Staged metadata differs from verified acceptance",
                )
            else:
                require(
                    entry == original["members"][name],
                    "Staged runtime payload differs from accepted candidate",
                )
        records.append(_record(row, staged, original, changes))
    expected = _plan(spec, candidate, acceptance, records)
    require(
        load_json(evidence_path(root, "NPM-PACKAGES.json")) == expected,
        "Staged manifest identity mismatch",
    )
    return expected
