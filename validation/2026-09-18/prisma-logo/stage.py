"""Bind the requested npm publication to already accepted native artifacts."""

import hashlib, json, shutil, subprocess, tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
cache = root.parent
repo = Path("/home/cdot/development/cdot65/prisma-airs-harness")
source = cache / "candidate-npm"
output = root / "npm-release"
proof = cache / "acceptance"
acceptance = json.loads((proof / "ACCEPTANCE.json").read_text())
metadata = json.loads((source / "NPM-PACKAGES.json").read_text())
assert metadata["source_commit"] == "71f2ea33144cc0535772e8408a3037502a82b02e"
assert json.loads((proof / "INSTALLED-MAC-SIGNATURE.json").read_text())["passed"]
output.mkdir(exist_ok=False)
(output / "tarballs").mkdir()


def digest(path):
    with path.open("rb") as f:
        return hashlib.file_digest(f, "sha256").hexdigest()


records = []
for record in metadata["publish_order"]:
    name = record["name"]
    version = record["version"]
    assert version == "0.1.0-alpha.22.onboarding.2"
    assert digest(source / "tarballs" / record["filename"]) == record["sha256"]
    package = output / name
    shutil.copytree(source / name, package, symlinks=True)
    manifest = json.loads((package / "package.json").read_text())
    manifest.pop("private", None)
    (package / "package.json").write_text(json.dumps(manifest, indent=2) + "\n")
    allowed = {"package/package.json"}
    if name != "airs-harness":
        info = json.loads((package / "BUILD-INFO.json").read_text())
        platform = next(
            p for p in acceptance["platforms"] if p["target"] == info["target"]
        )
        assert (
            digest(package / "bin/airs-harness")
            == platform["binary_sha256"]
            == info["binary_sha256"]
        )
        assert (
            platform["onboarding_checks"] >= 12
            and platform["installed_regressions"]["passed"] == 46
        )
        original = package / "validation-evidence/original-build-candidate.json"
        original.parent.mkdir(exist_ok=True)
        shutil.copy2(package / "BUILD-INFO.json", original)
        receipt = {
            "schema_version": 1,
            "scope": "owner-requested-onboarding-npm-publication",
            "publication_authorized": True,
            "authorization": "Owner requested replacing the welcome mark with Prisma AIRS branding; existing authorization covers publishing the updated harness for remote testing.",
            "authorization_date": "2026-09-18",
            "product_version": version,
            "source_commit": info["source_commit"],
            "target": info["target"],
            "binary_sha256": info["binary_sha256"],
            "native_installed_acceptance": True,
            "acceptance": platform,
            "passed": False,
            "release_ready": False,
            "production_sso_servicenow_acceptance": False,
            "full_gateway_lifecycle_passed": False,
            "independent_review_claimed": False,
            "limitations": [
                "Attended production SSO and ServiceNow acceptance remains with the owner.",
                "Six baseline HTTP fallback/classification test failures are documented and reproduced on prior source.",
            ],
            "original_candidate_sha256": digest(original),
        }
        (package / "VALIDATION.json").write_text(json.dumps(receipt, indent=2) + "\n")
        info.pop("publishable", None)
        info.pop("release_status", None)
        info["release_scope"] = receipt["scope"]
        info["validation_receipt_sha256"] = digest(package / "VALIDATION.json")
        (package / "BUILD-INFO.json").write_text(json.dumps(info, indent=2) + "\n")
        allowed |= {
            "package/BUILD-INFO.json",
            "package/VALIDATION.json",
            "package/validation-evidence/original-build-candidate.json",
        }
    packed = json.loads(
        subprocess.check_output(
            [
                "npm",
                "pack",
                "--ignore-scripts",
                "--json",
                "--pack-destination",
                str(output / "tarballs"),
            ],
            cwd=package,
            text=True,
            stderr=subprocess.DEVNULL,
        )
    )
    packed = next(iter(packed.values())) if isinstance(packed, dict) else packed[0]
    archive = output / "tarballs" / packed["filename"]

    def contents(path):
        with tarfile.open(path) as t:
            return {
                m.name: (
                    m.mode,
                    m.linkname,
                    hashlib.sha256(t.extractfile(m).read()).hexdigest()
                    if m.isfile()
                    else None,
                )
                for m in t.getmembers()
                if not m.isdir()
            }

    before = contents(source / "tarballs" / record["filename"])
    after = contents(archive)
    differences = {
        n for n in before.keys() | after.keys() if before.get(n) != after.get(n)
    }
    assert differences <= allowed, (name, differences - allowed)
    records.append(
        {
            "name": name,
            "version": version,
            "filename": packed["filename"],
            "integrity": packed["integrity"],
            "sha256": digest(archive),
            "original_candidate_sha256": record["sha256"],
            "changed_archive_entries": sorted(differences),
            "runtime_payload_unchanged": True,
        }
    )
    print(
        name + " staged; executable, launcher and bundled payload unchanged", flush=True
    )
metadata.pop("publishable", None)
metadata.pop("release_status", None)
metadata["publish_order"] = records
metadata["release_scope"] = "owner-requested-onboarding-npm-publication"
metadata["publication_preparation_source"] = subprocess.check_output(
    ["git", "rev-parse", "HEAD"], cwd=repo, text=True
).strip()
(output / "NPM-PACKAGES.json").write_text(json.dumps(metadata, indent=2) + "\n")
