"""Run installed native acceptance with portable, resumable evidence."""

import argparse
from pathlib import Path
import shutil
import subprocess
import uuid

from airs_release_contract import (
    PATTERNS as PATTERNS,
    RESULTS,
    contract,
    package_summary as package_summary,
    paths_for,
    stages,
    validate_result,
    verify_one,
    verify_acceptance_set as verify_acceptance_set,
)
from airs_release_execution import (
    candidate_identity,
    clean_environment,
    installed_identity,
    invocation,
    observed_target,
)
from airs_release_receipts import (
    atomic_json,
    evidence_path,
    safe_destination,
    verify_stage,
    verify_tooling,
    write_stage,
)
from airs_test_release_spec import (
    canonical_digest,
    digest_file,
    load_json,
    load_spec,
    require,
    validate_spec,
)


def run_acceptance(
    spec, packages, scripts, output, resume=False, installation="candidate"
):
    require(installation in ("candidate", "registry"), "Unknown installation mode")
    spec = validate_spec(spec)
    packages, scripts, output = (
        Path(packages).resolve(strict=True),
        Path(scripts).resolve(strict=True),
        safe_destination(output),
    )
    require(
        not output.is_relative_to(packages) and not output.is_relative_to(scripts),
        "Evidence must not be nested in release inputs",
    )
    require(
        not output.exists() or resume,
        "Evidence exists; use resume or a fresh output directory",
    )
    target = observed_target()
    environment = clean_environment()
    tooling = load_json(scripts.parent / "ACCEPTANCE-TOOLING.json")
    tooling_hash = verify_tooling(scripts.parent, tooling, spec["tooling_commit"])
    if installation == "registry":
        require(
            "scripts/validate_airs_test_registry_install.py" in tooling["files"],
            "Registry installer is absent from committed tooling",
        )
    require(
        all(
            f"scripts/{name}" in tooling["files"]
            for name in (
                "airs_release_acceptance.py",
                "airs_release_contract.py",
                "airs_release_execution.py",
                "airs_release_receipts.py",
                "airs_release_unittest.py",
                "validate_airs_npm.py",
                "validate_airs_command_output.py",
                "validate_airs_npm_upgrade.py",
            )
        ),
        "Incomplete validation tooling manifest",
    )
    identity = {
        "installation": installation,
        "spec_sha256": canonical_digest(spec),
        **{
            key: spec[key]
            for key in (
                "source_commit",
                "tooling_commit",
                "packaging_commit",
                "version",
            )
        },
        "target": target,
        "observed_target": target,
        "binary_sha256": next(
            row["binary_sha256"] for row in spec["platforms"] if row["target"] == target
        ),
        **candidate_identity(spec, packages),
        "tooling_sha256": tooling_hash,
    }
    output.mkdir(parents=True, exist_ok=True)
    work_root = safe_destination(output.with_name(output.name + ".work"))
    if work_root.exists():
        require(
            load_json(work_root / "OWNER.json") == identity,
            "Local acceptance work belongs to different inputs",
        )
    else:
        work_root.mkdir()
        atomic_json(work_root / "OWNER.json", identity)
    for filename, document in [
        ("TOOLING.json", tooling),
        ("CANDIDATE-NPM-PACKAGES.json", load_json(packages / "NPM-PACKAGES.json")),
    ]:
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
            require(
                set(row["outputs"]) == set(paths_for(name, target)),
                "Stage output set changed",
            )
            validate_result(
                name, load_json(output / f"files/{name}/{RESULTS[name]}"), spec, target
            )
            if name == "install":
                state = load_json(local_state)
                require(
                    state.get("identity") == identity,
                    "Local installation identity changed",
                )
                prefix = Path(state["prefix"])
                require(
                    prefix.resolve().is_relative_to(work_root.resolve()),
                    "Local prefix escapes owned work",
                )
                installed = installed_identity(
                    spec, packages, prefix, target, environment
                )
        else:
            work = work_root / f"{name}-{uuid.uuid4().hex}"
            work.mkdir()
            if name == "install":
                prefix = work / "prefix"
            command, result_root = invocation(
                name,
                spec,
                target,
                scripts,
                packages,
                work,
                prefix,
                installed,
                installation,
            )
            log = safe_destination(output / "logs" / f"{name}.log")
            log.parent.mkdir(parents=True, exist_ok=True)
            require(not log.is_symlink(), "Linked acceptance log is prohibited")
            stage_env = dict(environment)
            if installed:
                stage_env.update(
                    AIRS_HARNESS_BIN=str(installed[0]), AIRS_MANAGED_CLI_ACCEPTANCE="1"
                )
            with log.open("w") as stream:
                if name == "mac-signature":
                    native = installed[1]
                    for signature_command in command:
                        subprocess.run(
                            signature_command,
                            env=stage_env,
                            stdout=stream,
                            stderr=subprocess.STDOUT,
                            check=True,
                            timeout=120,
                        )
                    atomic_json(
                        work / RESULTS[name],
                        {
                            "passed": True,
                            "binary_sha256": digest_file(native),
                            "developer_id_team": spec["developer_id_team"],
                            "notarization_verified": True,
                            "installed_bytes": True,
                        },
                    )
                else:
                    subprocess.run(
                        command,
                        env=stage_env,
                        cwd=scripts.parent,
                        stdout=stream,
                        stderr=subprocess.STDOUT,
                        check=True,
                        timeout=900,
                    )
            validate_result(name, load_json(result_root / RESULTS[name]), spec, target)
            for relative in paths_for(name, target)[1:]:
                destination = safe_destination(output / relative)
                destination.parent.mkdir(parents=True, exist_ok=True)
                source = evidence_path(result_root, destination.name)
                require(
                    not destination.is_symlink(), "Linked retained output is prohibited"
                )
                shutil.copyfile(source, destination)
            if name == "install":
                installed = installed_identity(
                    spec, packages, prefix, target, environment
                )
                atomic_json(local_state, {"identity": identity, "prefix": str(prefix)})
            write_stage(
                output, name, inputs, dict(hashes), command, paths_for(name, target)
            )
        hashes[name] = digest_file(receipt_path)
    # Rehash the real installed native and managed bundle at final acceptance,
    # including on resume. Collector verification uses portable retained evidence.
    installed_identity(spec, packages, prefix, target, environment)
    require(
        verify_tooling(scripts.parent, tooling, spec["tooling_commit"]) == tooling_hash,
        "Validation tooling changed during acceptance",
    )
    require(
        candidate_identity(spec, packages)
        == {
            key: identity[key]
            for key in ("candidate_metadata_sha256", "candidate_packages_sha256")
        },
        "Candidate packages changed during acceptance",
    )
    atomic_json(
        output / "ACCEPTANCE.json",
        {
            "schema_version": 1,
            "passed": True,
            "identity": identity,
            "stages": stages(target),
            "stage_receipts": hashes,
            "production_sso": False,
            "production_servicenow": False,
        },
    )
    verify_one(spec, output, target, installation)
    return output / "ACCEPTANCE.json"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, required=True)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument("--scripts", type=Path, default=Path(__file__).parent)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--resume", action="store_true")
    parser.add_argument(
        "--installation", choices=("candidate", "registry"), default="candidate"
    )
    args = parser.parse_args()
    print(
        run_acceptance(
            load_spec(args.spec),
            args.packages,
            args.scripts,
            args.output,
            args.resume,
            args.installation,
        )
    )


if __name__ == "__main__":
    main()
