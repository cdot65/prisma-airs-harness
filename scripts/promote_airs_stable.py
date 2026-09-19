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


def validate_readiness(spec, readiness):
    require(spec["scope"] == STABLE_SCOPE, "Stable scope required")
    require(
        readiness.get("source_commit") == spec["source_commit"]
        and readiness.get("version") == spec["version"],
        "Readiness source/version mismatch",
    )
    workspace = readiness.get("workspace", {})
    require(
        workspace.get("failed") == 0
        and type(workspace.get("passed")) is int
        and workspace["passed"] > 0
        and workspace.get("source_commit") == spec["source_commit"],
        "A passing full workspace run on the release source is required",
    )
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
