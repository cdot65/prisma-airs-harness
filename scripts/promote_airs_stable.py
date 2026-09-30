#!/usr/bin/env python3
"""Promote verified stable candidate packages to the default npm install."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

from airs_release_contract import verify_acceptance_set
from airs_release_receipts import atomic_json, safe_destination
from airs_test_release_publish import Registry
from airs_test_release_spec import (
    PACKAGE_ORDER,
    STABLE_SCOPE,
    canonical_digest,
    load_json,
    load_spec,
    require,
)
from airs_test_release_stage import verify_staged


class StableRegistry(Registry):
    def promote(self, name, version):
        require(name in PACKAGE_ORDER, "Unexpected package")
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.upper().startswith(("NPM", "NODE_AUTH_TOKEN"))
            and key != "NODE_OPTIONS"
        }
        with tempfile.TemporaryDirectory(
            prefix=".npm-promote-", dir=self.output
        ) as directory:
            root = Path(directory)
            (root / "global.npmrc").write_text("")
            environment.update(
                NPM_CONFIG_USERCONFIG=str(self.userconfig),
                NPM_CONFIG_GLOBALCONFIG=str(root / "global.npmrc"),
                NPM_CONFIG_CACHE=str(root / "cache"),
                NPM_CONFIG_REGISTRY=self.registry,
                NPM_CONFIG_LOGLEVEL="error",
                NPM_CONFIG_LOGS_MAX="0",
                NPM_CONFIG_FETCH_RETRIES="0",
                NPM_CONFIG_FETCH_TIMEOUT="30000",
                NPM_CONFIG_UPDATE_NOTIFIER="false",
            )
            result = subprocess.run(
                [
                    "npm",
                    "dist-tag",
                    "add",
                    f"{name}@{version}",
                    "latest",
                    "--registry",
                    self.registry,
                ],
                cwd=root,
                env=environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=120,
            )
            require(
                result.returncode == 0,
                "Stable tag update failed; credentials were not logged",
            )


# These inherited 0.154 cases are the only exceptions eligible for the 0.1.1
# release review. The original full-workspace failure result remains in evidence.
WORKSPACE_BASELINE_011 = {
    "codex-exec-server::exec_process shell_snapshot_v2_capture_failure_falls_back_and_retries::remote_pipe_recovery": "disabled-upstream-service",
    "codex-exec-server::exec_process shell_snapshot_v2_capture_failure_falls_back_and_retries::remote_tty_recovery": "disabled-upstream-service",
    "codex-exec-server::exec_process shell_snapshot_v2_filters_profile_exports_and_stays_in_memory::remote_sandbox": "disabled-upstream-service",
    "codex-protocol permission_profile_intersection::tests::effective_workspace_intersection_preserves_network_metadata_and_temp": "corrected-test-fixture",
    "codex-skills-extension host_roots::tests::repo_ancestry_without_project_marker_does_not_walk_parents": "corrected-test-fixture",
    "codex-voice-host::bin/codex-voice-host devices::playout::tests::real_decoder_renders_current_rtp_and_rejects_pre_epoch_arrivals": "configured-test-runtime",
}


# Run 283 reproduced only these unchanged upstream service failures. This review
# is confined to the exact 0.1.2 native source; it cannot waive later releases or
# carry forward 0.1.1's corrected-fixture exceptions.
WORKSPACE_SOURCE_012 = "aee13a272b5e79fe0ea6fe62485ca7c164927028"
WORKSPACE_BASELINE_012 = {
    "codex-exec-server::exec_process shell_snapshot_v2_capture_failure_falls_back_and_retries::remote_pipe_recovery": "disabled-upstream-service",
    "codex-exec-server::exec_process shell_snapshot_v2_capture_failure_falls_back_and_retries::remote_tty_recovery": "disabled-upstream-service",
    "codex-exec-server::exec_process shell_snapshot_v2_filters_profile_exports_and_stays_in_memory::remote_sandbox": "disabled-upstream-service",
}


# Release 0.1.3 retains the complete six-failure run. Three schema comparisons
# are corrected in tests only; the remaining cases are disabled upstream services.
WORKSPACE_SOURCE_013 = "900792f8707c8f2a47d9814575469ecc4817be75"
WORKSPACE_BASELINE_013 = {
    **WORKSPACE_BASELINE_012,
    "codex-app-server-protocol schema_fixtures_tests::json_schema_fixtures_match_generated": "corrected-test-fixture",
    "codex-app-server-protocol schema_fixtures_tests::typescript_schema_fixtures_match_generated": "corrected-test-fixture",
    "codex-app-server-protocol schema_fixtures_tests::stable_precomputed_exports_match_schema_fixtures": "corrected-test-fixture",
}
# Release 0.1.4 stamps merged main: the accepted 0.1.4-alpha.2 preview (renamed
# launcher, Developer ID signing), in-place environment auth replacement and
# the fresh-conversation mark. The schema fixture correction is merged. Run 4226
# reproduced the disabled upstream service cases plus two stale `codex` binary
# help snapshots that the `airs-harness` binary's identical tests already pass;
# the snapshot files are corrected in tests only.
WORKSPACE_SOURCE_014 = "946dc6e34c9bd7f439c5fa00a9c05a11bee53673"
WORKSPACE_BASELINE_014 = {
    **WORKSPACE_BASELINE_012,
    "codex-cli::bin/codex airs_help::tests::standalone_environment_help_describes_the_complete_lifecycle": "corrected-test-fixture",
    "codex-cli::bin/codex airs_help::tests::standalone_login_help_describes_hidden_input": "corrected-test-fixture",
}
# Owner-authorized stable publications. A release whose runtime equals the last
# preview plus its version stamp records that; one that merges reviewed changes
# after the preview must list every merge so the diff from the accepted preview
# is explicit rather than implied.
RELEASE_AUTHORIZATIONS = {
    "0.1.3": {
        "source_commit": WORKSPACE_SOURCE_013,
        "previous_alpha": "0.1.3-alpha.7.mcp.1",
        "runtime_unchanged_from_requested_alpha": True,
        "reviewed_changes": None,
    },
    "0.1.4": {
        "source_commit": WORKSPACE_SOURCE_014,
        "previous_alpha": "0.1.4-alpha.2.mcp.1",
        "runtime_unchanged_from_requested_alpha": False,
        "reviewed_changes": {
            56: "c9b18e59cf4586abc57c2b0a02ac5797518263b3",
            59: "05423d46578ad2d96e7f9cc52955cd88b59a6fde",
            60: "a8046d50bfb83ddf64f3f343ad082a69bfe5ad40",
        },
    },
}


def validate_workspace(spec, workspace):
    require(
        workspace.get("scope") == "full-workspace"
        and type(workspace.get("passed")) is int
        and workspace["passed"] > 0
        and type(workspace.get("failed")) is int
        and workspace["failed"] >= 0
        and workspace.get("source_commit") == spec["source_commit"],
        "A complete full workspace run on the release source is required",
    )
    if workspace["failed"] == 0:
        require(not workspace.get("failures"), "Failure count disagrees with cases")
        return
    baseline = {}
    if spec["version"] == "0.1.1":
        baseline = WORKSPACE_BASELINE_011
    elif spec["version"] == "0.1.2" and spec["source_commit"] == WORKSPACE_SOURCE_012:
        baseline = WORKSPACE_BASELINE_012
    elif spec["version"] == "0.1.3" and spec["source_commit"] == WORKSPACE_SOURCE_013:
        baseline = WORKSPACE_BASELINE_013
    elif spec["version"] == "0.1.4" and spec["source_commit"] == WORKSPACE_SOURCE_014:
        baseline = WORKSPACE_BASELINE_014
    failures = workspace.get("failures", [])
    review = workspace.get("baseline_review", {})
    require(
        bool(baseline)
        and len(failures) == workspace["failed"]
        and len(set(failures)) == len(failures)
        and set(failures) <= baseline.keys()
        and review.get("source_commit") == spec["source_commit"]
        and review.get("upstream_revision") == "rust-v0.154.0"
        and review.get("upstream_implementations_unchanged") is True
        and review.get("unresolved_release_blockers") == [],
        "Unclassified workspace failures prevent promotion",
    )
    cases = review.get("cases", {})
    require(set(cases) == set(failures), "Every workspace failure needs a disposition")
    for name in failures:
        case = cases[name]
        require(
            case.get("disposition") == baseline[name]
            and case.get("evidence_verified") is True
            and isinstance(case.get("evidence_sha256"), str)
            and len(case["evidence_sha256"]) == 64
            and all(c in "0123456789abcdef" for c in case["evidence_sha256"]),
            "Workspace baseline evidence is incomplete",
        )
        if case["disposition"] == "disabled-upstream-service":
            require(
                case.get("installed_command_rejected") is True,
                "Disabled service requires an installed command check",
            )
        else:
            require(
                case.get("focused_check_passed") is True
                and case.get("runtime_source_unchanged") is True,
                "Fixture disposition requires a passing check on unchanged runtime code",
            )


def validate_readiness(spec, readiness):
    require(spec["scope"] == STABLE_SCOPE, "Stable scope required")
    require(
        readiness.get("source_commit") == spec["source_commit"]
        and readiness.get("version") == spec["version"],
        "Readiness source/version mismatch",
    )
    validate_workspace(spec, readiness.get("workspace", {}))
    # The owner explicitly requested stable publication of the current alpha.
    # Record that authorization separately from attended production SSO evidence.
    authorization = readiness.get("release_authorization", {})
    if authorization:
        known = RELEASE_AUTHORIZATIONS.get(spec["version"], {})
        require(
            known.get("source_commit") == spec["source_commit"]
            and authorization.get("version") == spec["version"]
            and authorization.get("source_commit") == spec["source_commit"]
            and authorization.get("explicit_stable_publication") is True
            and authorization.get("attended_acceptance_claimed") is False
            and authorization.get("runtime_unchanged_from_requested_alpha")
            is known["runtime_unchanged_from_requested_alpha"]
            and authorization.get("previous_alpha") == known["previous_alpha"]
            and isinstance(authorization.get("evidence_sha256"), str)
            and len(authorization["evidence_sha256"]) == 64
            and all(c in "0123456789abcdef" for c in authorization["evidence_sha256"]),
            "Explicit release authorization is incomplete or outside its scope",
        )
        reviewed = known["reviewed_changes"]
        changes = authorization.get("reviewed_changes")
        if reviewed is None:
            require(changes is None, "Unchanged runtime cannot list reviewed changes")
        else:
            require(
                isinstance(changes, list)
                and [c.get("pull_request") for c in changes] == list(reviewed)
                and all(
                    isinstance(c, dict)
                    and c.get("merge_commit") == reviewed[c["pull_request"]]
                    and isinstance(c.get("accepted_as"), str)
                    and 0 < len(c["accepted_as"]) <= 256
                    for c in changes
                ),
                "Every merged change since the accepted preview must be reviewed",
            )
        return
    owner = readiness.get("owner_acceptance", {})
    require(
        owner.get("version") in (spec["version"], spec["previous_version"])
        and all(
            owner.get(key) is True
            for key in (
                "inference_signin",
                "mcp_signin",
                "servicenow_read",
                "restart_reuse",
            )
        ),
        "Attended account acceptance is incomplete",
    )
    if owner["version"] != spec["version"]:
        require(
            readiness.get("runtime_behavior_unchanged_since_owner_acceptance") is True,
            "Earlier owner acceptance requires a reviewed runtime diff",
        )


def promote(spec, plan, verification, readiness, output, registry):
    validate_readiness(spec, readiness)
    require(
        verification.get("spec_sha256") == canonical_digest(spec)
        and verification.get("installation") == "registry",
        "Registry acceptance identity mismatch",
    )
    records = plan["publish_order"]
    require(
        [row["name"] for row in records] == PACKAGE_ORDER, "Native-first order required"
    )
    identity = canonical_digest(
        {
            "spec": spec,
            "plan": plan,
            "verification": verification,
            "readiness": readiness,
        }
    )
    output = safe_destination(output)
    output.mkdir(parents=True, exist_ok=True)
    path = output / "STABLE-PROMOTION.json"
    receipt = load_json(path) if path.exists() else None
    if receipt is None:
        receipt = {
            "schema_version": 1,
            "identity_sha256": identity,
            "version": spec["version"],
            "source_commit": spec["source_commit"],
            "workspace": readiness["workspace"],
            "release_authorization": readiness.get("release_authorization"),
            "original_tags": {
                name: registry.metadata(name)["dist-tags"] for name in PACKAGE_ORDER
            },
            "promoted": [],
            "complete": False,
        }
        atomic_json(path, receipt)
    require(
        receipt.get("identity_sha256") == identity,
        "Promotion checkpoint identity changed",
    )

    def check():
        for row in records:
            name = row["name"]
            document = registry.metadata(name)
            version = document["versions"].get(spec["version"], {})
            require(
                version.get("name") == name
                and version.get("version") == spec["version"]
                and version.get("dist", {}).get("integrity") == row["integrity"],
                "Published immutable package identity changed",
            )
            original = receipt["original_tags"][name]
            tags = document["dist-tags"]
            require(
                {k: v for k, v in tags.items() if k != "latest"}
                == {k: v for k, v in original.items() if k != "latest"}
                and tags.get(spec["tag"]) == spec["version"]
                and tags.get("latest") in (original.get("latest"), spec["version"]),
                "Registry tags changed concurrently; promotion stopped",
            )

    for row in records:
        check()
        name = row["name"]
        if registry.metadata(name)["dist-tags"].get("latest") != spec["version"]:
            registry.promote(name, spec["version"])
        check()
        require(
            registry.metadata(name)["dist-tags"].get("latest") == spec["version"],
            "Latest was not updated",
        )
        if name not in receipt["promoted"]:
            receipt["promoted"].append(name)
        atomic_json(path, receipt)
    check()
    require(
        all(
            registry.metadata(name)["dist-tags"].get("latest") == spec["version"]
            for name in PACKAGE_ORDER
        ),
        "Incomplete stable promotion",
    )
    receipt["complete"] = True
    atomic_json(path, receipt)
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in (
        "spec",
        "packages",
        "candidate-acceptance",
        "registry-acceptance",
        "readiness",
        "output",
        "userconfig",
    ):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    spec = load_spec(args.spec)
    readiness = load_json(args.readiness)
    validate_readiness(spec, readiness)
    plan = verify_staged(spec, args.packages, args.candidate_acceptance)
    verification = {
        **verify_acceptance_set(
            spec, args.registry_acceptance, installation="registry"
        ),
        "installation": "registry",
    }
    output = safe_destination(args.output)
    output.mkdir(parents=True, exist_ok=True)
    registry = StableRegistry(spec["registry"], args.userconfig, output)
    print(
        json.dumps(
            promote(spec, plan, verification, readiness, output, registry), indent=2
        )
    )


if __name__ == "__main__":
    main()
