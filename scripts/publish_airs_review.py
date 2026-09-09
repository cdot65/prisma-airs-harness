#!/usr/bin/env python3
"""Publish only exact approved review archives; never promote latest or production.

Uses the caller's npm/GitHub credential configuration without reading tokens.
Preserves a sanitized partial receipt if any operation fails. No lifecycle scripts
or native executables run, and the approved archive bytes are never repacked.
"""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
from urllib.parse import urlsplit

from airs_review_release import require
from plan_airs_review_publication import REGISTRY, plan_publication, regular_digest


class Registry:
    def __init__(self, cache):
        self.environment = dict(
            os.environ, NPM_CONFIG_CACHE=str(cache), NPM_CONFIG_UPDATE_NOTIFIER="false"
        )

    def command(
        self, arguments, *, cwd=None, allow_missing=False, allow_package_missing=False
    ):
        result = subprocess.run(
            arguments,
            cwd=cwd,
            env=self.environment,
            capture_output=True,
            text=True,
            timeout=180,
        )
        if result.returncode:
            # npm's structured E404 is the only accepted absent-version response.
            if allow_missing:
                try:
                    error = json.loads(result.stdout).get("error", {})
                except (ValueError, AttributeError):
                    error = {}
                if error.get("code") == "E404":
                    return None
            if allow_package_missing:
                try:
                    error = json.loads(result.stdout)
                except ValueError:
                    error = {}
                if isinstance(error, dict) and error.get("status") == "404":
                    return None
            raise RuntimeError(
                "Registry command failed; no raw authentication output was retained"
            )
        return result.stdout.strip()

    def view(self, spec):
        result = self.command(
            ["npm", "view", spec, "dist", "--json", "--registry", REGISTRY],
            allow_missing=True,
        )
        return None if result is None else json.loads(result)

    def download(self, spec, directory):
        value = json.loads(
            self.command(
                [
                    "npm",
                    "pack",
                    spec,
                    "--ignore-scripts",
                    "--json",
                    "--registry",
                    REGISTRY,
                    "--pack-destination",
                    str(directory),
                ]
            )
        )
        record = next(iter(value.values())) if isinstance(value, dict) else value[0]
        filename = record["filename"]
        require(
            isinstance(filename, str)
            and Path(filename).name == filename
            and "/" not in filename
            and "\\" not in filename,
            "Invalid registry download filename",
        )
        return directory / filename

    def publish(self, archive, tag):
        self.command(
            [
                "npm",
                "publish",
                str(archive),
                "--ignore-scripts",
                "--access",
                "restricted",
                "--tag",
                tag,
                "--registry",
                REGISTRY,
            ]
        )

    def tag(self, spec, tag):
        self.command(["npm", "dist-tag", "add", spec, tag, "--registry", REGISTRY])

    def tags(self, name):
        return json.loads(
            self.command(
                ["npm", "view", name, "dist-tags", "--json", "--registry", REGISTRY]
            )
        )

    def association(self, name):
        result = self.command(
            ["gh", "api", "users/cdot65/packages/npm/" + name.split("/")[1]],
            allow_package_missing=True,
        )
        if result is None:
            return None
        data = json.loads(result)
        return {
            "repository": (data.get("repository") or {}).get("full_name"),
            "visibility": data.get("visibility"),
        }


def save(path, value):
    with tempfile.NamedTemporaryFile(mode="w", dir=path.parent, delete=False) as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())
        temporary = Path(stream.name)
    temporary.replace(path)


def publish_review(
    packages, approved_plan, expected_plan_sha256, output, *, registry=None
):
    packages, approved_plan, output = Path(packages), Path(approved_plan), Path(output)
    require(
        re.fullmatch(r"[0-9a-f]{64}", expected_plan_sha256) is not None,
        "An explicit approved plan SHA256 is required",
    )
    require(
        regular_digest(approved_plan, limit=4 * 1024 * 1024)[0] == expected_plan_sha256,
        "Approved publication plan checksum mismatch",
    )
    plan = json.loads(approved_plan.read_text())
    require(
        plan == plan_publication(packages, plan.get("dist_tag")),
        "Staged publication no longer matches the approved plan",
    )
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    receipt_path = output / "PUBLICATION.json"
    receipt = {
        "schema_version": 1,
        "scope": plan["scope"],
        "registry": REGISTRY,
        "dist_tag": plan["dist_tag"],
        "source_commit": plan["source_commit"],
        "version": plan["version"],
        "approved_plan_sha256": expected_plan_sha256,
        "published": False,
        "complete": False,
        "phase": "preflight",
        "mutation_attempted": False,
        "packages": [],
        "full_authentication_release_ready": False,
        "teammate_read_only_install_verified": False,
    }
    save(receipt_path, receipt)
    with tempfile.TemporaryDirectory(
        prefix="private-registry-", dir=output
    ) as temporary:
        work = Path(temporary)
        registry = registry or Registry(work / "cache")
        snapshots = {}
        for record in plan["packages"]:
            snapshot = work / record["filename"]
            shutil.copyfile(packages / "tarballs" / record["filename"], snapshot)
            require(
                regular_digest(snapshot) == (record["sha256"], record["integrity"]),
                "Publication archive changed while being snapshotted",
            )
            snapshots[record["name"]] = snapshot

        def verified_download(record, phase):
            spec = record["name"] + "@" + record["version"]
            dist = registry.view(spec)
            require(
                isinstance(dist, dict) and dist.get("integrity") == record["integrity"],
                "Registry version differs from the approved immutable archive",
            )
            url = urlsplit(dist.get("tarball", ""))
            require(
                url.scheme == "https"
                and url.netloc == "npm.pkg.github.com"
                and not url.username
                and not url.password
                and not url.query
                and not url.fragment,
                "Registry tarball is outside the approved GitHub origin",
            )
            directory = work / (phase + "-" + record["name"].split("/")[1])
            directory.mkdir()
            fetched = registry.download(spec, directory)
            require(
                regular_digest(fetched) == (record["sha256"], record["integrity"]),
                "Downloaded registry bytes differ from the approved archive",
            )
            association = registry.association(record["name"])
            require(
                association
                == {"repository": "cdot65/airs-harness", "visibility": "private"},
                "GitHub package must be private and associated with the intended repository",
            )
            return association

        try:
            existing = set()
            for record in plan["packages"]:
                association = registry.association(record["name"])
                require(
                    association is None
                    or association
                    == {"repository": "cdot65/airs-harness", "visibility": "private"},
                    "Existing GitHub package must be private and linked to cdot65/airs-harness before publication; configure its package repository connection",
                )
            # Detect every immutable-version collision before the first registry mutation.
            for record in plan["packages"]:
                spec = record["name"] + "@" + record["version"]
                if registry.view(spec) is not None:
                    verified_download(record, "preflight")
                    existing.add(record["name"])
            require(
                plan == plan_publication(packages, plan["dist_tag"]),
                "Staged publication changed during registry preflight",
            )
            receipt["phase"] = "publish-and-download"
            save(receipt_path, receipt)
            for record in plan["packages"]:
                if record["name"] not in existing:
                    receipt["mutation_attempted"] = True
                    save(receipt_path, receipt)
                    registry.publish(snapshots[record["name"]], plan["dist_tag"])
                association = verified_download(record, "verified")
                receipt["packages"].append(
                    {
                        **record,
                        **association,
                        "download_verified": True,
                        "already_existed": record["name"] in existing,
                    }
                )
                save(receipt_path, receipt)
            # Only tag old immutable versions after every archive has passed download checks.
            receipt["phase"] = "verify-review-tag"
            save(receipt_path, receipt)
            for record in plan["packages"]:
                if record["name"] in existing:
                    receipt["mutation_attempted"] = True
                    save(receipt_path, receipt)
                    registry.tag(
                        record["name"] + "@" + record["version"], plan["dist_tag"]
                    )
                require(
                    registry.tags(record["name"]).get(plan["dist_tag"])
                    == record["version"],
                    "Review tag does not select the approved version",
                )
            receipt.update(published=True, complete=True, phase="complete")
            save(receipt_path, receipt)
        except Exception:
            receipt["failure"] = (
                "Publication stopped; retain completed package records and retry only identical approved bytes"
            )
            save(receipt_path, receipt)
            raise
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument("--approved-plan", type=Path, required=True)
    parser.add_argument("--approved-plan-sha256", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    publish_review(
        args.packages, args.approved_plan, args.approved_plan_sha256, args.output
    )


if __name__ == "__main__":
    main()
