"""Bind the owner's alpha.22 publication direction to tested native bytes."""

import argparse, hashlib, json, shutil
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument("--native", type=Path, required=True)
p.add_argument("--output", type=Path, required=True)
p.add_argument("--environments", type=Path, required=True)
p.add_argument("--fixtures", type=Path, required=True)
a = p.parse_args()


def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


info = json.loads((a.native / "BUILD-INFO.json").read_text())
assert info["version"] == "0.1.0-alpha.22"
assert info["source_commit"] == "2a787c03d1673b28fe85803c4082dd1a5e85a36e"
assert digest(a.native / "airs-harness") == info["binary_sha256"]
env = json.loads(a.environments.read_text())
assert (
    env["passed"]
    and env["version"] == info["version"]
    and env["binary_sha256"] == info["binary_sha256"]
)
log = a.fixtures.read_text()
assert "\nOK" in log and "\nFAILED" not in log and "\nRan " in log
if info["target"] == "aarch64-unknown-linux-musl":
    assert env["platform"] == "Linux" and env["architecture"] in ["aarch64", "arm64"]
if info["target"] == "aarch64-apple-darwin":
    sign = json.loads((a.native / "SIGNING.json").read_text())
    assert sign["binary_sha256"] == info["binary_sha256"]
    assert all(
        sign[k]
        for k in ["codesign_verified", "hardened_runtime", "notarization_verified"]
    )
shutil.copytree(a.native, a.output)
evidence = a.output / "validation-evidence"
evidence.mkdir(exist_ok=True)
records = []
for role, path in [
    ("environment-lifecycle", a.environments),
    ("native-fixture-suite", a.fixtures),
    ("original-candidate-validation", a.native / "VALIDATION.json"),
]:
    name = role + path.suffix
    shutil.copy2(path, evidence / name)
    records.append({"role": role, "path": name, "sha256": digest(evidence / name)})
validation = {
    "schema_version": 1,
    "scope": "owner-authorized-alpha22-command-migration",
    "publication_authorized": True,
    "authorization": "okay, begin. let me know when both the CLI and harness are published and ready for me to test end-to-end on another remote machine",
    "authorization_date": "2026-09-17",
    "product_version": info["version"],
    "source_commit": info["source_commit"],
    "target": info["target"],
    "binary_sha256": info["binary_sha256"],
    "native_environment_lifecycle_passed": True,
    "native_fixture_suite_passed": True,
    "passed": False,
    "release_ready": False,
    "full_gateway_lifecycle_passed": False,
    "production_sso_servicenow_acceptance": False,
    "limitations": [
        "Fresh production SSO and ServiceNow tool-call acceptance are not claimed.",
        "Production frontend expiry and hourly renewal acceptance remain incomplete.",
        "Full workspace validation and independent release review are not claimed.",
    ],
    "evidence": records,
}
(a.output / "VALIDATION.json").write_text(json.dumps(validation, indent=2) + "\n")
info.pop("publishable", None)
info.pop("release_status", None)
info["release_scope"] = validation["scope"]
info["validation_receipt_sha256"] = digest(a.output / "VALIDATION.json")
(a.output / "BUILD-INFO.json").write_text(json.dumps(info, indent=2) + "\n")
(a.output / "SHA256SUMS").write_text(
    "".join(
        f"{digest(f)}  {f.relative_to(a.output).as_posix()}\n"
        for f in sorted(a.output.rglob("*"))
        if f.is_file() and f.name != "SHA256SUMS"
    )
)
print(
    json.dumps(
        {
            "target": info["target"],
            "binary_sha256": info["binary_sha256"],
            "publication_authorized": True,
            "production_sso_servicenow_acceptance": False,
        }
    )
)
