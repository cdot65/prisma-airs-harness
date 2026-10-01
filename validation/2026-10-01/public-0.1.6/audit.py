#!/usr/bin/env python3
"""Audit retained public 0.1.6 native acceptance and workflow receipts offline."""

import json
from pathlib import Path
import sys
import subprocess
import tarfile
import tempfile

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(REPO / "scripts"))
from airs_release_contract import verify_acceptance_set
from airs_release_receipts import verify_tooling
from airs_test_release_spec import digest_file, load_spec, PACKAGE_ORDER


def main():
    subprocess.run(
        [sys.executable, str(REPO / "validation/2026-10-01/private-0.1.6/audit.py")],
        check=True,
    )
    report = json.loads((HERE / "RELEASE-ACCEPTANCE.json").read_text())
    bundle = HERE / "evidence.tar.gz"
    assert digest_file(bundle) == report["evidence_bundle"]["sha256"]
    with tempfile.TemporaryDirectory(prefix="airs-public-016-") as temporary:
        root = Path(temporary)
        with tarfile.open(bundle) as t:
            members = t.getmembers()
            names = [m.name for m in members]
            assert len(names) == len(set(names)) == report["evidence_bundle"]["files"]
            assert all(
                m.isfile()
                and not Path(m.name).is_absolute()
                and ".." not in Path(m.name).parts
                for m in members
            )
            assert sum(m.size for m in members) < 100 * 1024 * 1024
            t.extractall(root, filter="data")
        inventory = json.loads((root / "EVIDENCE-INVENTORY.json").read_text())["files"]
        assert set(inventory) | {"EVIDENCE-INVENTORY.json"} == set(names)
        for name, digest in inventory.items():
            assert digest_file(root / name) == digest, name
        spec = load_spec(root / "SPEC.json")
        assert spec["version"] == report["version"] == "0.1.6"
        assert spec["registry"] == "https://registry.npmjs.org"
        for phase in ["candidate", "registry"]:
            actual = verify_acceptance_set(spec, root / (phase + "-evidence"), phase)
            assert actual == report[phase + "_acceptance"]
        tooling = root / "frozen-tooling"
        verify_tooling(
            tooling,
            json.loads((tooling / "ACCEPTANCE-TOOLING.json").read_text()),
            spec["tooling_commit"],
        )
        publication = json.loads(
            (root / "workflow-publish/PUBLICATION.json").read_text()
        )
        assert (
            publication["published"]
            and publication["existing_protected_tags_preserved"]
        )
        assert [r["name"] for r in publication["packages"]] == PACKAGE_ORDER
        assert all(r["registry_integrity_verified"] for r in publication["packages"])
        promotion = json.loads(
            (root / "workflow-promote/STABLE-PROMOTION.json").read_text()
        )
        assert promotion["complete"] and promotion["version"] == "0.1.6"
        defaults = json.loads((root / "DEFAULT-INSTALLS.json").read_text())
        assert defaults["passed"] and len(defaults["platforms"]) == 3
        for row in defaults["platforms"]:
            assert row["passed"] and row["receipt"]["passed"]
        tags = json.loads((root / "RELEASE-TAGS.json").read_text())
        assert [row["name"] for row in tags["public"]] == PACKAGE_ORDER
        assert all(
            row["tags"]["latest"] == row["tags"]["stable-candidate"] == "0.1.6"
            for row in tags["public"]
        )
        assert tags["temporary_npm_credential_removed"] is True
        assert (
            tags["private_promotion_performed"]
            is report["private_promotion_performed"]
            is False
        )
        assert (
            report["public_latest"] == "0.1.6"
            and report["default_installs_passed"] is True
        )
        assert len(report["feature_scores"]) == 4 and all(
            row["score"] >= 9 for row in report["feature_scores"]
        )
        helper = json.loads((root / "PUBLIC-UBUNTU-HELPER.json").read_text())
        assert (
            helper == report["ubuntu_helper"]
            and helper["passed"]
            and helper["checks_passed"] == 6
        )
        for index in report["successful_workflow_runs"]:
            status = json.loads(
                (root / ("WORKFLOW-" + str(index) + "-STATUS.json")).read_text()
            )
            assert status["status"] == "success"
        assert (
            report["workspace"]["failed"] == 3
            and report["workspace"]["new_failures"] == []
        )
        assert (
            report["production_sso_claimed"] is False
            and report["paid_jev_service_claimed"] is False
        )
    print(
        json.dumps(
            {
                "passed": True,
                "evidence_files": len(names),
                "native_targets": 3,
                "public_latest": "0.1.6",
            }
        )
    )


if __name__ == "__main__":
    main()
