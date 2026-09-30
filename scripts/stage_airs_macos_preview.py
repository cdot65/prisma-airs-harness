#!/usr/bin/env python3
"""Stage an accepted, ad-hoc signed Mac candidate for npm.cdot.io/mac-preview.

This is not the notarized release path. Preserve candidate provenance and every
runtime byte, and bind the metadata-only promotion to the native CI receipts.
"""

import argparse
import json
import re
from pathlib import Path

from airs_release_receipts import evidence_path
from airs_test_release_archive import inspect_archive, rewrite_archive
from airs_test_release_spec import digest_file, load_json, require

REGISTRY = "https://npm.cdot.io"
NATIVE = "airs-harness-darwin-arm64"
MANIFEST = "package/package.json"
BUILD = "package/BUILD-INFO.json"


def encoded(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def stage(packages, evidence, output, source_commit, tag):
    require(tag == "mac-preview", "Only the explicit mac-preview tag is allowed")
    require(re.fullmatch(r"[0-9a-f]{40}", source_commit), "Full source SHA required")
    metadata = load_json(evidence_path(packages, "NPM-PACKAGES.json"))
    require(metadata.get("source_commit") == source_commit, "Candidate source mismatch")
    require(metadata.get("registry") == REGISTRY, "Only npm.cdot.io is allowed")
    records = metadata.get("publish_order", [])
    require(
        [r["name"] for r in records] == [NATIVE, "airs-harness"],
        "Expected Mac native and launcher only",
    )
    version = records[0]["version"]
    require(
        re.fullmatch(r"\d+\.\d+\.\d+-alpha\.\d+\.mcp\.\d+", version),
        "Expected a versioned test build",
    )
    selection = load_json(evidence_path(evidence, "artifact-selection.json"))
    validation = load_json(evidence_path(evidence, "VALIDATION.json"))
    binary = selection["binary_sha256"]
    require(
        selection.get("runtime_source") == source_commit
        and selection.get("validation_tooling_source")
        == metadata.get("packaging_commit")
        and selection.get("signing")
        == {"ad_hoc": True, "developer_id": False, "notarized": False}
        and validation.get("source_commit") == source_commit
        and validation.get("validation_tooling_commit")
        == metadata.get("packaging_commit")
        and validation.get("product_version") == version
        and validation.get("target") == "aarch64-apple-darwin"
        and validation.get("binary_sha256") == binary
        and validation.get("validation_run") == selection.get("acceptance_run"),
        "Native acceptance identity mismatch",
    )
    evidence_hashes = {}
    for name in [
        "artifact-selection.json",
        "VALIDATION.json",
        "native-integrity.json",
        "native-cli-keychain.json",
        "cli-keychain.json",
        "keychain.json",
        "npm-install.json",
        "prisma-cli.json",
    ]:
        path = evidence_path(evidence, name)
        receipt = load_json(path)
        if name not in ("artifact-selection.json", "VALIDATION.json"):
            require(receipt.get("passed") is True, f"Missing successful {name}")
        if name in ("native-integrity.json", "npm-install.json"):
            require(
                receipt.get("binary_sha256") == binary
                and receipt.get("source_commit") == source_commit,
                f"Installed identity mismatch in {name}",
            )
        if name in (
            "native-cli-keychain.json",
            "cli-keychain.json",
            "npm-install.json",
        ):
            require(
                receipt.get("version") == f"airs {version}",
                f"Version mismatch in {name}",
            )
        evidence_hashes[name] = digest_file(path)
    for name in ("native-tests.log", "npm-tests.log"):
        path = evidence_path(evidence, name)
        text = path.read_text()
        require(
            re.search(
                r"(?m)^Ran [1-9][0-9]* tests? in .+\n\nOK(?: \(skipped=\d+\))?$", text
            )
            and not re.search(r"(?m)^FAILED(?: |$)", text),
            f"Missing successful {name}",
        )
        evidence_hashes[name] = digest_file(path)

    # Inspect the complete set before creating any publishable output.
    candidates = []
    for record in records:
        require(
            record["version"] == version
            and record["filename"] == f"{record['name']}-{version}.tgz",
            "Candidate package identity mismatch",
        )
        archive = evidence_path(packages, "tarballs/" + record["filename"])
        inventory = inspect_archive(archive)
        require(
            all(inventory[k] == record[k] for k in ("sha256", "integrity")),
            "Candidate archive hash mismatch",
        )
        manifest = dict(inventory["json"][MANIFEST])
        require(
            manifest.get("private") is True
            and manifest.get("name") == record["name"]
            and manifest.get("version") == version
            and manifest.get("publishConfig") == {"registry": REGISTRY},
            "Expected a private registry-bound candidate",
        )
        manifest.pop("private")
        manifest.update(os=["darwin"], cpu=["arm64"])
        changes = {MANIFEST: encoded(manifest)}
        if record["name"] == NATIVE:
            original = inventory["json"][BUILD]
            require(
                original.get("source_commit") == source_commit
                and original.get("version") == version
                and original.get("target") == "aarch64-apple-darwin"
                and original.get("binary_sha256") == binary
                and inventory["members"]["package/bin/airs-harness"]["sha256"]
                == binary,
                "Native payload differs from accepted binary",
            )
            info = dict(
                original, release_status="accepted-mac-preview", publishable=True
            )
            receipt = dict(
                validation,
                scope="owner-authorized-ad-hoc-mac-preview",
                registry=REGISTRY,
                tag=tag,
                release_ready=False,
                notarized=False,
                live_gateway_e2e=False,
                acceptance_evidence_sha256=evidence_hashes,
            )
            changes.update(
                {
                    BUILD: encoded(info),
                    "package/VALIDATION.json": encoded(receipt),
                    "package/validation-evidence/original-build-candidate.json": encoded(
                        original
                    ),
                }
            )
        else:
            require(
                manifest.get("optionalDependencies") == {NATIVE: version},
                "Launcher native dependency mismatch",
            )
        candidates.append((record, archive, changes))

    output = Path(output)
    (output / "tarballs").mkdir(parents=True, exist_ok=False)
    staged = []
    for record, archive, changes in candidates:
        inventory = rewrite_archive(
            archive, output / "tarballs" / record["filename"], changes
        )
        staged.append(
            dict(
                record,
                sha256=inventory["sha256"],
                integrity=inventory["integrity"],
                candidate_sha256=record["sha256"],
            )
        )
    result = dict(
        metadata,
        publish_order=staged,
        release_status="accepted-mac-preview",
        publishable=True,
        published=False,
        tag=tag,
        release_ready=False,
        notarized=False,
        acceptance_evidence_sha256=evidence_hashes,
        staging_script_sha256=digest_file(Path(__file__)),
        candidate_metadata_sha256=digest_file(
            evidence_path(packages, "NPM-PACKAGES.json")
        ),
    )
    (output / "NPM-PACKAGES.json").write_bytes(encoded(result))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--tag", required=True, choices=["mac-preview"])
    args = parser.parse_args()
    print(
        json.dumps(
            stage(
                args.packages, args.evidence, args.output, args.source_commit, args.tag
            ),
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
