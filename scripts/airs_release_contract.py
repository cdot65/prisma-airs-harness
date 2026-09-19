"""Portable acceptance contracts and source-bound receipt collection."""

from pathlib import Path

from airs_release_receipts import evidence_path, verify_stage
from airs_test_release_spec import (
    COMMIT,
    PACKAGE_ORDER,
    TARGETS,
    canonical_digest,
    digest_file,
    load_json,
    require,
    validate_spec,
)


PATTERNS = {
    "installed-regressions": "test_airs_harness*.py",
    "mcp-manager": "test_airs_mcp_manager.py",
    "doctor": "test_airs_doctor.py",
}


RESULTS = {
    "install": "INSTALL-VERIFICATION.json",
    "onboarding": "ONBOARDING-ACCEPTANCE.json",
    "terminals": "TERMINAL-ACCEPTANCE.json",
    **{key: "TEST-RESULTS.json" for key in PATTERNS},
    "managed-cli": "MANAGED-CLI.json",
    "upgrade": "UPGRADE.json",
    "command-output": "COMMAND-OUTPUT.json",
    "mac-signature": "INSTALLED-MAC-SIGNATURE.json",
}


def stages(target):
    return (
        list(RESULTS)
        if target == "aarch64-apple-darwin"
        else [name for name in RESULTS if name != "mac-signature"]
    )


def package_summary(spec, manifest):
    require(
        manifest.get("source_commit") == spec["source_commit"]
        and manifest.get("packaging_commit") == spec["packaging_commit"],
        "Candidate source identity mismatch",
    )
    require(
        manifest.get("registry", "").rstrip("/") == spec["registry"],
        "Candidate registry mismatch",
    )
    records = manifest.get("publish_order", [])
    require(
        [row.get("name") for row in records] == PACKAGE_ORDER,
        "Incomplete or reordered candidate package set",
    )
    summary = {}
    for row in records:
        require(row.get("version") == spec["version"], "Candidate version mismatch")
        require(
            isinstance(row.get("sha256"), str)
            and len(row["sha256"]) == 64
            and isinstance(row.get("integrity"), str)
            and row["integrity"].startswith("sha512-"),
            "Missing candidate archive digest",
        )
        summary[row["name"]] = {
            key: row[key] for key in ("sha256", "integrity", "version", "filename")
        }
    return summary


def contract(name, target):
    return {
        "stage": name,
        "target": target,
        "selection": PATTERNS.get(name, RESULTS[name]),
        "normal_python": True,
    }


def paths_for(name, target):
    paths = [f"logs/{name}.log", f"files/{name}/{RESULTS[name]}"]
    if name == "install":
        paths += ["files/install/INSTALL-NETWORK.json"]
    if name == "onboarding" and target == "aarch64-apple-darwin":
        paths += ["files/onboarding/CREDENTIAL-CLEANUP.json"]
    return paths


def validate_result(name, value, spec, target):
    require(
        isinstance(value, dict) and value.get("passed") is True,
        f"{name} acceptance did not pass",
    )
    expected = next(
        row["binary_sha256"] for row in spec["platforms"] if row["target"] == target
    )
    hash_key = {
        "install": "binary_sha256",
        "onboarding": "binary_sha256",
        "terminals": "native_binary_sha256",
        "command-output": "native_sha256",
        "mac-signature": "binary_sha256",
    }.get(name)
    if hash_key:
        require(value.get(hash_key) == expected, f"{name} native identity mismatch")
    if name == "install":
        require(
            value.get("source_commit") == spec["source_commit"]
            and value.get("version") == "airs " + spec["version"],
            "Installed source/version mismatch",
        )
        require(
            value.get("native_package") == TARGETS[target],
            "Installed native target mismatch",
        )
    if name == "onboarding":
        require(
            value.get("local_https_oidc") is True
            and value.get("native_os_store") in (True, "Keychain"),
            "Native credential fixture not established",
        )
        require(
            value.get("production_sso") is False
            and value.get("production_servicenow") is False,
            "Fixture cannot claim production acceptance",
        )
    if name in ("onboarding", "terminals", "managed-cli", "command-output"):
        require(
            isinstance(value.get("checks"), list) and value["checks"],
            "Missing behavioral acceptance checks",
        )
    if name in PATTERNS:
        require(value.get("schema_version") == 1, "Unknown unittest result schema")
        require(
            value.get("pattern") == PATTERNS[name]
            and not value.get("failures")
            and not value.get("errors"),
            "Fixture selection or results mismatch",
        )
        ids, skipped, count = (
            value.get("test_ids"),
            value.get("skipped"),
            value.get("tests_run"),
        )
        require(
            isinstance(ids, list)
            and isinstance(skipped, list)
            and type(count) is int
            and count == len(ids)
            and len(set(ids)) == count
            and count > len(skipped)
            and all(
                isinstance(row, dict) and row.get("test") in ids for row in skipped
            ),
            "Empty or entirely skipped fixture suite",
        )
    if name == "upgrade":
        require(
            value.get("previous") == spec["previous_version"]
            and value.get("previous_registry") == spec["registry"]
            and value.get("version") == spec["version"]
            and value.get("configuration_preserved") is True
            and value.get("legacy_target_preserved") is True,
            "Upgrade identity or preservation mismatch",
        )
    if name == "managed-cli":
        require(
            value.get("live_api_operations") is False,
            "Managed CLI fixture cannot claim live API coverage",
        )
    if name == "mac-signature":
        require(
            value.get("developer_id_team") == spec["developer_id_team"]
            and value.get("notarization_verified") is True
            and value.get("installed_bytes") is True,
            "Installed Mac signing acceptance missing",
        )


def verification_commit(spec, installation, revision=None):
    if revision is None:
        return spec["tooling_commit"]
    require(
        installation == "registry",
        "Tooling revisions apply only to registry verification",
    )
    require(
        isinstance(revision, str) and COMMIT.fullmatch(revision) is not None,
        "Verification tooling requires a full source commit",
    )
    return revision


def verify_one(
    spec, root, target, installation="candidate", verification_tooling_commit=None
):
    expected_tooling = verification_commit(
        spec, installation, verification_tooling_commit
    )
    result = load_json(evidence_path(root, "ACCEPTANCE.json"))
    require(result.get("schema_version") == 1, "Unknown acceptance schema")
    identity = result.get("identity")
    require(isinstance(identity, dict), "Missing acceptance identity")
    for key in ("source_commit", "tooling_commit", "packaging_commit", "version"):
        require(identity.get(key) == spec[key], "Acceptance source/version mismatch")
    require(
        identity.get("installation") == installation,
        "Acceptance installation mode mismatch",
    )
    require(
        identity.get("verification_tooling_commit") == verification_tooling_commit,
        "Registry verification tooling revision mismatch",
    )
    expected = next(
        row["binary_sha256"] for row in spec["platforms"] if row["target"] == target
    )
    require(
        identity.get("spec_sha256") == canonical_digest(spec)
        and identity.get("target") == target
        and identity.get("binary_sha256") == expected,
        "Acceptance specification/target mismatch",
    )
    require(
        identity.get("observed_target") == target,
        "Acceptance was not performed on the declared native target",
    )
    require(
        result.get("passed") is True
        and result.get("production_sso") is False
        and result.get("production_servicenow") is False,
        "Invalid acceptance scope",
    )
    require(
        result.get("stages") == stages(target),
        "Incomplete or unexpected acceptance stage set",
    )
    require(
        digest_file(evidence_path(root, "CANDIDATE-NPM-PACKAGES.json"))
        == identity.get("candidate_metadata_sha256"),
        "Candidate evidence changed",
    )
    manifest = load_json(evidence_path(root, "CANDIDATE-NPM-PACKAGES.json"))
    require(
        canonical_digest(package_summary(spec, manifest))
        == identity.get("candidate_packages_sha256"),
        "Candidate archive-set evidence changed",
    )
    tooling = load_json(evidence_path(root, "TOOLING.json"))
    require(
        tooling.get("tooling_commit") == expected_tooling
        and canonical_digest(tooling) == identity.get("tooling_sha256"),
        "Acceptance tooling evidence changed",
    )
    hashes = {}
    for name in stages(target):
        inputs = {**identity, "contract": contract(name, target)}
        row = verify_stage(root, name, inputs, dict(hashes))
        require(
            set(row["outputs"]) == set(paths_for(name, target)),
            "Unexpected or missing retained stage outputs",
        )
        validate_result(
            name,
            load_json(evidence_path(root, f"files/{name}/{RESULTS[name]}")),
            spec,
            target,
        )
        if name == "onboarding" and target == "aarch64-apple-darwin":
            cleanup = load_json(
                evidence_path(root, "files/onboarding/CREDENTIAL-CLEANUP.json")
            )
            require(
                cleanup.get("passed") is True and not cleanup.get("failures"),
                "Mac fixture credential cleanup did not pass",
            )
        hashes[name] = digest_file(evidence_path(root, f"receipts/{name}.json"))
    require(result.get("stage_receipts") == hashes, "Acceptance receipt set changed")
    return {
        "target": target,
        "binary_sha256": expected,
        "acceptance_sha256": digest_file(root / "ACCEPTANCE.json"),
        "evidence_sha256": canonical_digest(hashes),
        "stages": stages(target),
    }, identity


def verify_acceptance_set(
    spec, evidence_root, installation="candidate", verification_tooling_commit=None
):
    require(installation in ("candidate", "registry"), "Unknown installation mode")
    spec, evidence_root = validate_spec(spec), Path(evidence_root)
    verification_commit(spec, installation, verification_tooling_commit)
    actual = {path.parent.name for path in evidence_root.glob("*/ACCEPTANCE.json")}
    require(
        actual == set(TARGETS), "Exactly three native acceptance roots are required"
    )
    platforms, identities = [], []
    for target in TARGETS:
        result, identity = verify_one(
            spec,
            evidence_root / target,
            target,
            installation,
            verification_tooling_commit,
        )
        platforms.append(result)
        identities.append(identity)
    candidate = {
        key: identities[0][key]
        for key in ("candidate_metadata_sha256", "candidate_packages_sha256")
    }
    require(
        all(
            all(identity.get(key) == value for key, value in candidate.items())
            for identity in identities
        ),
        "Native acceptances used different package sets",
    )
    return {
        "spec_sha256": canonical_digest(spec),
        "source_commit": spec["source_commit"],
        "tooling_commit": spec["tooling_commit"],
        "packaging_commit": spec["packaging_commit"],
        "version": spec["version"],
        **(
            {"verification_tooling_commit": verification_tooling_commit}
            if verification_tooling_commit is not None
            else {}
        ),
        **candidate,
        "platforms": platforms,
        "evidence_sha256": canonical_digest(platforms),
    }
