"""Installed native acceptance with portable, source-bound resumable evidence."""

import argparse
import base64
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import uuid

from airs_release_receipts import atomic_json, evidence_path, safe_destination, verify_stage, verify_tooling, write_stage
from airs_test_release_spec import PACKAGE_ORDER, TARGETS, canonical_digest, digest_file, load_json, load_spec, require, validate_spec

PATTERNS = {
    "installed-regressions": "test_airs_harness*.py",
    "mcp-manager": "test_airs_mcp_manager.py",
    "doctor": "test_airs_doctor.py",
}
RESULTS = {
    "install": "INSTALL-VERIFICATION.json", "onboarding": "ONBOARDING-ACCEPTANCE.json",
    "terminals": "TERMINAL-ACCEPTANCE.json", **{key: "TEST-RESULTS.json" for key in PATTERNS},
    "managed-cli": "MANAGED-CLI.json", "upgrade": "UPGRADE.json",
    "command-output": "COMMAND-OUTPUT.json", "mac-signature": "INSTALLED-MAC-SIGNATURE.json",
}


def stages(target):
    return list(RESULTS) if target == "aarch64-apple-darwin" else [name for name in RESULTS if name != "mac-signature"]


def observed_target():
    host = (platform.system(), platform.machine())
    known = {("Linux", "x86_64"): "x86_64-unknown-linux-musl", ("Linux", "aarch64"): "aarch64-unknown-linux-musl", ("Linux", "arm64"): "aarch64-unknown-linux-musl", ("Darwin", "arm64"): "aarch64-apple-darwin"}
    require(host in known, "Acceptance requires native Linux x64/ARM64 or Apple Silicon")
    return known[host]


def package_summary(spec, manifest):
    require(manifest.get("source_commit") == spec["source_commit"] and manifest.get("packaging_commit") == spec["packaging_commit"], "Candidate source identity mismatch")
    require(manifest.get("registry", "").rstrip("/") == spec["registry"], "Candidate registry mismatch")
    records = manifest.get("publish_order", [])
    require([row.get("name") for row in records] == PACKAGE_ORDER, "Incomplete or reordered candidate package set")
    summary = {}
    for row in records:
        require(row.get("version") == spec["version"], "Candidate version mismatch")
        require(isinstance(row.get("sha256"), str) and len(row["sha256"]) == 64 and isinstance(row.get("integrity"), str) and row["integrity"].startswith("sha512-"), "Missing candidate archive digest")
        summary[row["name"]] = {key: row[key] for key in ("sha256", "integrity", "version", "filename")}
    return summary


def candidate_identity(spec, packages):
    manifest = load_json(packages / "NPM-PACKAGES.json")
    summary = package_summary(spec, manifest)
    for row in manifest["publish_order"]:
        require(row.get("version") == spec["version"], "Candidate version mismatch")
        archive = evidence_path(packages, "tarballs/" + row["filename"])
        digest = digest_file(archive)
        integrity = "sha512-" + base64.b64encode(bytes.fromhex(digest_file(archive, "sha512"))).decode()
        require(digest == row.get("sha256") and integrity == row.get("integrity"), "Candidate archive integrity mismatch")
    return {"candidate_metadata_sha256": digest_file(packages / "NPM-PACKAGES.json"), "candidate_packages_sha256": canonical_digest(summary)}


def contract(name, target):
    return {"stage": name, "target": target, "selection": PATTERNS.get(name, RESULTS[name]), "normal_python": True}


def paths_for(name, target):
    paths = [f"logs/{name}.log", f"files/{name}/{RESULTS[name]}"]
    if name == "install":
        paths += ["files/install/INSTALL-NETWORK.json"]
    if name == "onboarding" and target == "aarch64-apple-darwin":
        paths += ["files/onboarding/CREDENTIAL-CLEANUP.json"]
    return paths


def validate_result(name, value, spec, target):
    require(isinstance(value, dict) and value.get("passed") is True, f"{name} acceptance did not pass")
    expected = next(row["binary_sha256"] for row in spec["platforms"] if row["target"] == target)
    hash_key = {"install": "binary_sha256", "onboarding": "binary_sha256", "terminals": "native_binary_sha256", "command-output": "native_sha256", "mac-signature": "binary_sha256"}.get(name)
    if hash_key:
        require(value.get(hash_key) == expected, f"{name} native identity mismatch")
    if name == "install":
        require(value.get("source_commit") == spec["source_commit"] and value.get("version") == "airs " + spec["version"], "Installed source/version mismatch")
        require(value.get("native_package") == TARGETS[target], "Installed native target mismatch")
    if name == "onboarding":
        require(value.get("local_https_oidc") is True and value.get("native_os_store") in (True, "Keychain"), "Native credential fixture not established")
        require(value.get("production_sso") is False and value.get("production_servicenow") is False, "Fixture cannot claim production acceptance")
    if name in ("onboarding", "terminals", "managed-cli", "command-output"):
        require(isinstance(value.get("checks"), list) and value["checks"], "Missing behavioral acceptance checks")
    if name in PATTERNS:
        require(value.get("schema_version") == 1, "Unknown unittest result schema")
        require(value.get("pattern") == PATTERNS[name] and not value.get("failures") and not value.get("errors"), "Fixture selection or results mismatch")
        ids, skipped, count = value.get("test_ids"), value.get("skipped"), value.get("tests_run")
        require(isinstance(ids, list) and isinstance(skipped, list) and type(count) is int and count == len(ids) and len(set(ids)) == count and count > len(skipped) and all(isinstance(row, dict) and row.get("test") in ids for row in skipped), "Empty or entirely skipped fixture suite")
    if name == "upgrade":
        require(value.get("previous") == spec["previous_version"] and value.get("previous_registry") == spec["registry"] and value.get("version") == spec["version"] and value.get("configuration_preserved") is True and value.get("legacy_target_preserved") is True, "Upgrade identity or preservation mismatch")
    if name == "managed-cli":
        require(value.get("live_api_operations") is False, "Managed CLI fixture cannot claim live API coverage")
    if name == "mac-signature":
        require(value.get("developer_id_team") == spec["developer_id_team"] and value.get("notarization_verified") is True and value.get("installed_bytes") is True, "Installed Mac signing acceptance missing")


def verify_one(spec, root, target, installation="candidate"):
    result = load_json(evidence_path(root, "ACCEPTANCE.json"))
    require(result.get("schema_version") == 1, "Unknown acceptance schema")
    identity = result.get("identity")
    require(isinstance(identity, dict), "Missing acceptance identity")
    for key in ("source_commit", "tooling_commit", "packaging_commit", "version"):
        require(identity.get(key) == spec[key], "Acceptance source/version mismatch")
    require(identity.get("installation") == installation, "Acceptance installation mode mismatch")
    expected = next(row["binary_sha256"] for row in spec["platforms"] if row["target"] == target)
    require(identity.get("spec_sha256") == canonical_digest(spec) and identity.get("target") == target and identity.get("binary_sha256") == expected, "Acceptance specification/target mismatch")
    require(identity.get("observed_target") == target, "Acceptance was not performed on the declared native target")
    require(result.get("passed") is True and result.get("production_sso") is False and result.get("production_servicenow") is False, "Invalid acceptance scope")
    require(result.get("stages") == stages(target), "Incomplete or unexpected acceptance stage set")
    require(digest_file(evidence_path(root, "CANDIDATE-NPM-PACKAGES.json")) == identity.get("candidate_metadata_sha256"), "Candidate evidence changed")
    manifest = load_json(evidence_path(root, "CANDIDATE-NPM-PACKAGES.json"))
    require(canonical_digest(package_summary(spec, manifest)) == identity.get("candidate_packages_sha256"), "Candidate archive-set evidence changed")
    tooling = load_json(evidence_path(root, "TOOLING.json"))
    require(tooling.get("tooling_commit") == spec["tooling_commit"] and canonical_digest(tooling) == identity.get("tooling_sha256"), "Acceptance tooling evidence changed")
    hashes = {}
    for name in stages(target):
        inputs = {**identity, "contract": contract(name, target)}
        row = verify_stage(root, name, inputs, dict(hashes))
        require(set(row["outputs"]) == set(paths_for(name, target)), "Unexpected or missing retained stage outputs")
        validate_result(name, load_json(evidence_path(root, f"files/{name}/{RESULTS[name]}")), spec, target)
        if name == "onboarding" and target == "aarch64-apple-darwin":
            cleanup = load_json(evidence_path(root, "files/onboarding/CREDENTIAL-CLEANUP.json"))
            require(cleanup.get("passed") is True and not cleanup.get("failures"), "Mac fixture credential cleanup did not pass")
        hashes[name] = digest_file(evidence_path(root, f"receipts/{name}.json"))
    require(result.get("stage_receipts") == hashes, "Acceptance receipt set changed")
    return {"target": target, "binary_sha256": expected, "acceptance_sha256": digest_file(root / "ACCEPTANCE.json"), "evidence_sha256": canonical_digest(hashes), "stages": stages(target)}, identity


def verify_acceptance_set(spec, evidence_root, installation="candidate"):
    require(installation in ("candidate", "registry"), "Unknown installation mode")
    spec, evidence_root = validate_spec(spec), Path(evidence_root)
    actual = {path.parent.name for path in evidence_root.glob("*/ACCEPTANCE.json")}
    require(actual == set(TARGETS), "Exactly three native acceptance roots are required")
    platforms, identities = [], []
    for target in TARGETS:
        result, identity = verify_one(spec, evidence_root / target, target, installation)
        platforms.append(result)
        identities.append(identity)
    candidate = {key: identities[0][key] for key in ("candidate_metadata_sha256", "candidate_packages_sha256")}
    require(all(all(identity.get(key) == value for key, value in candidate.items()) for identity in identities), "Native acceptances used different package sets")
    return {"spec_sha256": canonical_digest(spec), "source_commit": spec["source_commit"], "tooling_commit": spec["tooling_commit"], "packaging_commit": spec["packaging_commit"], "version": spec["version"], **candidate, "platforms": platforms, "evidence_sha256": canonical_digest(platforms)}


def clean_environment():
    allowed = {"PATH", "HOME", "USER", "USERPROFILE", "LANG", "LC_ALL", "LC_CTYPE", "TMPDIR", "TMP", "TEMP", "SHELL", "DISPLAY", "WAYLAND_DISPLAY", "DBUS_SESSION_BUS_ADDRESS", "XDG_RUNTIME_DIR", "GNOME_KEYRING_CONTROL"}
    environment = {key: value for key, value in os.environ.items() if key in allowed}
    environment.update(TERM="xterm-256color", PYTHONOPTIMIZE="0")
    return environment


def installed_identity(spec, packages, prefix, target, environment):
    from airs_bundle import verify_bundle

    launcher = prefix / "lib/node_modules/airs-harness"
    manifest = load_json(launcher / "package.json")
    require(manifest.get("name") == "airs-harness" and manifest.get("version") == spec["version"], "Installed launcher identity mismatch")
    require(manifest.get("optionalDependencies") == {name: spec["version"] for name in TARGETS.values()}, "Installed launcher native dependencies mismatch")
    native_manifest = subprocess.check_output(["node", "-e", "const r=require('module').createRequire(process.argv[1]);console.log(r.resolve(process.argv[2]+'/package.json'));", str(launcher / "package.json"), TARGETS[target]], env=environment, text=True, timeout=20).strip()
    native_package = Path(native_manifest).resolve().parent
    require(native_package.is_relative_to(prefix.resolve()), "Installed native escapes its prefix")
    native = native_package / "bin/airs-harness"
    info = load_json(native_package / "BUILD-INFO.json")
    native_info = load_json(native_package / "package.json")
    expected = next(row["binary_sha256"] for row in spec["platforms"] if row["target"] == target)
    require(native_info.get("name") == TARGETS[target] and native_info.get("version") == spec["version"], "Installed native package identity mismatch")
    require(info.get("source_commit") == spec["source_commit"] and info.get("target") == target and info.get("version") == spec["version"] and info.get("binary_sha256") == expected, "Installed native provenance mismatch")
    require(digest_file(native) == expected, "Actual installed native bytes changed")
    candidate = load_json(packages / "NPM-PACKAGES.json")
    tooling = load_json(launcher / "PACKAGE-TOOLING.json")
    require(tooling == candidate.get("package_tooling") and tooling.get("packaging_commit") == spec["packaging_commit"], "Installed packaging tooling mismatch")
    installed_files = {"npm/airs-harness/" + path.relative_to(launcher).as_posix() for directory in ("lib", "bin", "managed-cli") for path in (launcher / directory).iterdir() if path.is_file()}
    expected_files = {relative for relative in tooling.get("files", {}) if relative.startswith(("npm/airs-harness/lib/", "npm/airs-harness/bin/", "npm/airs-harness/managed-cli/"))}
    require(installed_files and installed_files == expected_files, "Installed launcher tooling inventory changed")
    for relative, expected_hash in tooling.get("files", {}).items():
        if relative.startswith(("npm/airs-harness/lib/", "npm/airs-harness/bin/", "npm/airs-harness/managed-cli/")):
            require(digest_file(evidence_path(launcher, relative.removeprefix("npm/airs-harness/"))) == expected_hash, "Installed launcher tooling bytes changed")
    inventory = launcher / "BUNDLE-INVENTORY.json"
    require(digest_file(inventory) == candidate.get("cli_bundle", {}).get("inventory_sha256"), "Installed managed CLI inventory mismatch")
    verify_bundle(launcher, load_json(inventory), require_windows_wrappers=False)
    command = prefix / "bin/airs"
    require(command.resolve(strict=True).is_relative_to(launcher.resolve()), "Installed launcher command escapes package")
    version = subprocess.check_output([str(command), "--version"], env=environment, text=True, timeout=20).strip()
    require(version == "airs " + spec["version"], "Actual installed executable version mismatch")
    return command, native, launcher


def invocation(name, spec, target, scripts, packages, work, prefix, installed, installation="candidate"):
    python = sys.executable  # No -O; clean_environment forces assertions on in legacy validators.
    if name == "install":
        if installation == "registry":
            atomic_json(work / "SPEC.json", spec)
            return [python, str(scripts / "validate_airs_test_registry_install.py"), "--spec", str(work / "SPEC.json"), "--packages", str(packages), "--prefix", str(prefix)], prefix
        return [python, str(scripts / "validate_airs_npm.py"), "--packages", str(packages), "--prefix", str(prefix)], prefix
    command, native, launcher = installed
    if name in PATTERNS:
        return [python, str(scripts / "airs_release_unittest.py"), "--scripts", str(scripts), "--pattern", PATTERNS[name], "--receipt", str(work / RESULTS[name])], work
    if name == "onboarding":
        validator = "validate_airs_onboarding_macos.py" if target == "aarch64-apple-darwin" else "validate_airs_onboarding.py"
        command_args = [python, str(scripts / validator), "--binary", str(command), "--native-binary", str(native), "--output", str(work)]
        return (command_args if target == "aarch64-apple-darwin" else ["dbus-run-session", "--", *command_args]), work
    if name == "terminals":
        args = [python, str(scripts / "validate_airs_onboarding_terminals.py"), "--binary", str(command), "--native-binary", str(native), "--output", str(work)]
        return args + (["--shells", "bash", "zsh"] if target == "aarch64-apple-darwin" else []), work
    if name == "managed-cli":
        return [python, str(scripts / "validate_prisma_cli.py"), "--launcher", str(launcher / "bin/airs.js"), "--receipt", str(work / RESULTS[name])], work
    if name == "upgrade":
        return [python, str(scripts / "validate_airs_npm_upgrade.py"), "--packages", str(packages), "--previous", spec["previous_version"], "--registry", spec["registry"], "--output", str(work / "upgrade")], work / "upgrade"
    if name == "command-output":
        return [python, str(scripts / "validate_airs_command_output.py"), "--binary", str(command), "--native", str(native), "--scripts", str(scripts), "--receipt", str(work / RESULTS[name])], work
    require(name == "mac-signature", "Unknown acceptance stage")
    requirement = '=anchor apple generic and certificate leaf[subject.OU] = "' + spec["developer_id_team"] + '" and certificate 1[field.1.2.840.113635.100.6.2.6] exists and certificate leaf[field.1.2.840.113635.100.6.1.13] exists'
    return [["/usr/bin/codesign", "--verify", "--strict", "--verbose=2", "-R", requirement, str(native)], ["/usr/bin/codesign", "--verify", "--strict", "--verbose=4", "--check-notarization", "-R", "=notarized", str(native)]], work


def run_acceptance(spec, packages, scripts, output, resume=False, installation="candidate"):
    require(installation in ("candidate", "registry"), "Unknown installation mode")
    spec = validate_spec(spec)
    packages, scripts, output = Path(packages).resolve(strict=True), Path(scripts).resolve(strict=True), safe_destination(output)
    require(not output.is_relative_to(packages) and not output.is_relative_to(scripts), "Evidence must not be nested in release inputs")
    require(not output.exists() or resume, "Evidence exists; use resume or a fresh output directory")
    target = observed_target()
    environment = clean_environment()
    tooling = load_json(scripts.parent / "ACCEPTANCE-TOOLING.json")
    tooling_hash = verify_tooling(scripts.parent, tooling, spec["tooling_commit"])
    if installation == "registry":
        require("scripts/validate_airs_test_registry_install.py" in tooling["files"], "Registry installer is absent from committed tooling")
    require(all(f"scripts/{name}" in tooling["files"] for name in ("airs_release_acceptance.py", "airs_release_receipts.py", "airs_release_unittest.py", "validate_airs_npm.py", "validate_airs_command_output.py", "validate_airs_npm_upgrade.py")), "Incomplete validation tooling manifest")
    identity = {"installation": installation, "spec_sha256": canonical_digest(spec), **{key: spec[key] for key in ("source_commit", "tooling_commit", "packaging_commit", "version")}, "target": target, "observed_target": target, "binary_sha256": next(row["binary_sha256"] for row in spec["platforms"] if row["target"] == target), **candidate_identity(spec, packages), "tooling_sha256": tooling_hash}
    output.mkdir(parents=True, exist_ok=True)
    work_root = safe_destination(output.with_name(output.name + ".work"))
    if work_root.exists():
        require(load_json(work_root / "OWNER.json") == identity, "Local acceptance work belongs to different inputs")
    else:
        work_root.mkdir()
        atomic_json(work_root / "OWNER.json", identity)
    for filename, document in [("TOOLING.json", tooling), ("CANDIDATE-NPM-PACKAGES.json", load_json(packages / "NPM-PACKAGES.json"))]:
        path = output / filename
        if path.exists():
            require(load_json(path) == document, "Retained acceptance inputs changed")
        else:
            # Preserve exact candidate bytes: its receipt hash is not canonical JSON.
            if filename == "CANDIDATE-NPM-PACKAGES.json":
                shutil.copyfile(packages / "NPM-PACKAGES.json", path)
            else:
                atomic_json(path, document)
    hashes, installed = {}, None
    local_state = work_root / "LOCAL.json"
    prefix = None
    for name in stages(target):
        inputs = {**identity, "contract": contract(name, target)}
        receipt_path = output / "receipts" / f"{name}.json"
        if receipt_path.exists():
            require(resume, "Stage already exists")
            row = verify_stage(output, name, inputs, dict(hashes))
            require(set(row["outputs"]) == set(paths_for(name, target)), "Stage output set changed")
            validate_result(name, load_json(output / f"files/{name}/{RESULTS[name]}"), spec, target)
            if name == "install":
                state = load_json(local_state)
                require(state.get("identity") == identity, "Local installation identity changed")
                prefix = Path(state["prefix"])
                require(prefix.resolve().is_relative_to(work_root.resolve()), "Local prefix escapes owned work")
                installed = installed_identity(spec, packages, prefix, target, environment)
        else:
            work = work_root / f"{name}-{uuid.uuid4().hex}"
            work.mkdir()
            if name == "install":
                prefix = work / "prefix"
            command, result_root = invocation(name, spec, target, scripts, packages, work, prefix, installed, installation)
            log = safe_destination(output / "logs" / f"{name}.log")
            log.parent.mkdir(parents=True, exist_ok=True)
            require(not log.is_symlink(), "Linked acceptance log is prohibited")
            stage_env = dict(environment)
            if installed:
                stage_env.update(AIRS_HARNESS_BIN=str(installed[0]), AIRS_MANAGED_CLI_ACCEPTANCE="1")
            with log.open("w") as stream:
                if name == "mac-signature":
                    native = installed[1]
                    for signature_command in command:
                        subprocess.run(signature_command, env=stage_env, stdout=stream, stderr=subprocess.STDOUT, check=True, timeout=120)
                    atomic_json(work / RESULTS[name], {"passed": True, "binary_sha256": digest_file(native), "developer_id_team": spec["developer_id_team"], "notarization_verified": True, "installed_bytes": True})
                else:
                    subprocess.run(command, env=stage_env, cwd=scripts.parent, stdout=stream, stderr=subprocess.STDOUT, check=True, timeout=900)
            validate_result(name, load_json(result_root / RESULTS[name]), spec, target)
            for relative in paths_for(name, target)[1:]:
                destination = safe_destination(output / relative)
                destination.parent.mkdir(parents=True, exist_ok=True)
                source = evidence_path(result_root, destination.name)
                require(not destination.is_symlink(), "Linked retained output is prohibited")
                shutil.copyfile(source, destination)
            if name == "install":
                installed = installed_identity(spec, packages, prefix, target, environment)
                atomic_json(local_state, {"identity": identity, "prefix": str(prefix)})
            write_stage(output, name, inputs, dict(hashes), command, paths_for(name, target))
        hashes[name] = digest_file(receipt_path)
    # Rehash the real installed native and managed bundle at final acceptance,
    # including on resume. Collector verification uses portable retained evidence.
    installed_identity(spec, packages, prefix, target, environment)
    require(verify_tooling(scripts.parent, tooling, spec["tooling_commit"]) == tooling_hash, "Validation tooling changed during acceptance")
    require(candidate_identity(spec, packages) == {key: identity[key] for key in ("candidate_metadata_sha256", "candidate_packages_sha256")}, "Candidate packages changed during acceptance")
    atomic_json(output / "ACCEPTANCE.json", {"schema_version": 1, "passed": True, "identity": identity, "stages": stages(target), "stage_receipts": hashes, "production_sso": False, "production_servicenow": False})
    verify_one(spec, output, target, installation)
    return output / "ACCEPTANCE.json"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, required=True)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument("--scripts", type=Path, default=Path(__file__).parent)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--resume", action="store_true")
    parser.add_argument("--installation", choices=("candidate", "registry"), default="candidate")
    args = parser.parse_args()
    print(run_acceptance(load_spec(args.spec), args.packages, args.scripts, args.output, args.resume, args.installation))


if __name__ == "__main__":
    main()
