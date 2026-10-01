#!/usr/bin/env python3
"""Verify retained private-release bytes and native acceptance chains without network access."""

import json
from pathlib import Path
import sys
import tarfile
import tempfile

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
sys.path.insert(0, str(REPO / "scripts"))
from airs_release_contract import verify_acceptance_set
from airs_release_receipts import verify_tooling
from airs_test_release_spec import digest_file, load_spec


def main():
    report = json.loads((HERE / "RELEASE-ACCEPTANCE.json").read_text())
    bundle = HERE / report["evidence_bundle"]["filename"]
    assert bundle.name == "evidence.tar.gz" and bundle.parent == HERE
    assert digest_file(bundle) == report["evidence_bundle"]["sha256"]
    with tempfile.TemporaryDirectory(prefix="airs-016-evidence-") as temporary:
        root = Path(temporary)
        with tarfile.open(bundle) as archive:
            members = archive.getmembers()
            names = [member.name for member in members]
            assert len(names) == len(set(names)) == report["evidence_bundle"]["files"]
            assert sum(member.size for member in members) < 100 * 1024 * 1024
            assert all(
                member.isfile()
                and not Path(member.name).is_absolute()
                and ".." not in Path(member.name).parts
                for member in members
            )
            archive.extractall(root, filter="data")
        inventory_path = root / "EVIDENCE-INVENTORY.json"
        assert (
            digest_file(inventory_path) == report["evidence_bundle"]["inventory_sha256"]
        )
        inventory = json.loads(inventory_path.read_text())["files"]
        assert set(inventory) | {"EVIDENCE-INVENTORY.json"} == set(names)
        for name, expected in inventory.items():
            assert digest_file(root / name) == expected, name
        spec = load_spec(root / "SPEC.json")
        assert spec["version"] == report["version"] == "0.1.6"
        assert spec["source_commit"] == report["source_commit"]
        for installation, folder in [
            ("candidate", "candidate-evidence"),
            ("registry", "registry-evidence"),
        ]:
            actual = verify_acceptance_set(spec, root / folder, installation)
            assert actual == report[installation + "_acceptance"]
        tooling = root / "frozen-tooling"
        verify_tooling(
            tooling,
            json.loads((tooling / "ACCEPTANCE-TOOLING.json").read_text()),
            spec["tooling_commit"],
        )
        publication = json.loads(
            (root / "private-publication/PUBLICATION.json").read_text()
        )
        assert (
            publication["published"]
            and publication["existing_protected_tags_preserved"]
        )
        assert len(publication["packages"]) == 4
        assert all(
            row["registry_integrity_verified"] and row["runtime_payload_unchanged"]
            for row in publication["packages"]
        )
        assert len(report["feature_scores"]) == 4 and all(
            row["score"] >= 9 for row in report["feature_scores"]
        )
        assert report["public_npm_unchanged"] and report["stable_latest"] == "0.1.5"
        assert report["workspace"]["new_failures"] == []
    print(
        json.dumps(
            {
                "passed": True,
                "evidence_files": len(names),
                "native_targets": 3,
                "installation_phases": 2,
                "feature_scores": [row["score"] for row in report["feature_scores"]],
            }
        )
    )


if __name__ == "__main__":
    main()
