#!/usr/bin/env python3
"""Exercise npm upgrades and legacy-command migration without uninstalling.

The old version comes from the owned registry. Candidate archives are served by
the existing isolated test registry; all new native bytes must match provenance.
Only disposable prefixes and harness state are used.
"""

import argparse
import hashlib
import json
import os
import re
import subprocess
import threading
from http.server import ThreadingHTTPServer
from pathlib import Path

from airs_npm_registry import install_environment, registry_handler


def run(arguments, environment, log):
    result = subprocess.run(
        arguments,
        env=environment,
        capture_output=True,
        text=True,
        timeout=300,
        check=False,
    )
    log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f"Upgrade command failed; inspect {log}")
    return result.stdout.strip()


def native_info(prefix):
    manifest = prefix / "lib/node_modules/airs-harness/package.json"
    path = subprocess.check_output(
        [
            "node",
            "-e",
            (
                "const {createRequire}=require('module');const r=createRequire(process.argv[1]);"
                "console.log(r.resolve('airs-harness-'+process.platform+'-'+process.arch+'/package.json'));"
            ),
            str(manifest),
        ],
        text=True,
    ).strip()
    package = Path(path).parent
    native = package / "bin/airs-harness"
    return native, json.loads((package / "BUILD-INFO.json").read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--previous", default="0.1.0-alpha.12")
    args = parser.parse_args()
    if not re.fullmatch(
        r"\d+\.\d+\.\d+-alpha\.\d+(?:\.onboarding\.\d+)?", args.previous
    ):
        parser.error("Expected an explicit immutable alpha version")
    args.output.mkdir(parents=True, exist_ok=False)
    packages = args.packages.resolve(strict=True)
    records = json.loads((packages / "NPM-PACKAGES.json").read_text())["publish_order"]
    launcher = next(record for record in records if record["name"] == "airs-harness")
    assert launcher["version"] != args.previous
    prefix = args.output / "npm managed prefix"
    prefix.mkdir()
    old_env = install_environment(prefix, "https://npm.cdot.io", False)
    old_env["AIRS_HARNESS_HOME"] = str(args.output / "preserved-harness-state")
    run(
        [
            "npm",
            "install",
            "-g",
            "--prefix",
            str(prefix),
            "--ignore-scripts",
            "--no-audit",
            "--no-fund",
            "--registry=https://npm.cdot.io",
            "airs-harness@" + args.previous,
        ],
        old_env,
        args.output / "install-previous.log",
    )
    command = prefix / "bin/airs-harness"
    previous_alpha = int(args.previous.rsplit(".", 1)[1])
    previous_command_name = "airs" if previous_alpha >= 22 else "airs-harness"
    assert (
        run([str(command), "--version"], old_env, args.output / "previous-version.log")
        == previous_command_name + " " + args.previous
    )
    old_native, old_info = native_info(prefix)
    old_hash = hashlib.sha256(old_native.read_bytes()).hexdigest()
    assert old_hash == old_info["binary_sha256"]

    # Preserve a real pre-upgrade harness configuration, not a fabricated marker.
    previous_setup = ["env", "create", "work"] if previous_alpha >= 21 else ["setup"]
    run(
        [
            str(command),
            *previous_setup,
            "--gateway-url",
            "https://gateway.example.com/v1",
        ],
        old_env,
        args.output / "previous-setup.log",
    )
    state = Path(old_env["AIRS_HARNESS_HOME"])
    before = {
        str(p.relative_to(state)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in state.rglob("*")
        # arg0 contains per-process command shims and locks, not persisted
        # settings. Starting a later executable legitimately replaces them.
        if p.is_file() and p.relative_to(state).parts[:2] != ("tmp", "arg0")
    }
    (args.output / "state-before.json").write_text(json.dumps(before, indent=2))
    old_link = os.readlink(command)

    # The legacy path intentionally points outside npm's package directory.
    legacy = args.output / "legacy prefix"
    (legacy / "bin").mkdir(parents=True)
    preserved_binary = args.output / "legacy executable"
    preserved_binary.write_bytes(old_native.read_bytes())
    preserved_binary.chmod(0o755)
    (legacy / "bin/airs-harness").symlink_to(preserved_binary)
    # This fixture knows the manual link's owner. Preserve that link explicitly
    # before installation, as the runbook requires; never ask npm to force it.
    (legacy / "bin/airs-harness").rename(legacy / "bin/airs-harness.pre-alpha22")

    # Some hosts put ~/.local/bin before a separate npm global prefix. Hand
    # that known legacy command to npm once, without changing the npm prefix.
    front = args.output / "front-of-PATH"
    front.mkdir()
    visible_command = front / "airs-harness"
    visible_command.symlink_to(preserved_binary)
    replacement = front / ".airs-harness-npm-handover"
    replacement.symlink_to(command)
    os.replace(replacement, visible_command)

    metadata, archives, requests, unexpected, redirects = {}, {}, [], [], []
    server = ThreadingHTTPServer(
        ("127.0.0.1", 0),
        registry_handler(metadata, archives, requests, unexpected, redirects, True),
    )
    registry = f"http://127.0.0.1:{server.server_port}"
    for record in records:
        manifest = json.loads((packages / record["name"] / "package.json").read_text())
        path = f"/{record['name']}/-/{record['filename']}"
        archive = packages / "tarballs" / record["filename"]
        assert hashlib.sha256(archive.read_bytes()).hexdigest() == record["sha256"]
        manifest["dist"] = {
            "tarball": registry + path,
            "integrity": record["integrity"],
        }
        metadata["/" + record["name"]] = {
            "name": record["name"],
            "dist-tags": {"latest": record["version"]},
            "versions": {record["version"]: manifest},
        }
        archives[path] = archive
    threading.Thread(target=server.serve_forever, daemon=True).start()
    results = []
    try:
        for destination, flags, case in [
            (prefix, [], "npm-managed-in-place"),
            (legacy, [], "legacy-one-time-migration"),
        ]:
            env = install_environment(destination, registry, True)
            env["AIRS_HARNESS_HOME"] = str(state)
            run(
                [
                    "npm",
                    "install",
                    "-g",
                    "--prefix",
                    str(destination),
                    "--ignore-scripts",
                    "--no-audit",
                    "--no-fund",
                    "--registry",
                    registry,
                    "airs-harness@latest",
                    *flags,
                ],
                env,
                args.output / (case + ".log"),
            )
            actual = run(
                [str(destination / "bin/airs-harness"), "--version"],
                env,
                args.output / (case + "-version.log"),
            )
            assert actual == "airs " + launcher["version"]
            binary, info = native_info(destination)
            digest = hashlib.sha256(binary.read_bytes()).hexdigest()
            assert digest == info["binary_sha256"] and digest != old_hash
            results.append(
                {
                    "case": case,
                    "passed": True,
                    "binary_sha256": digest,
                    "uninstall_used": False,
                    "manual_command_removal": False,
                    "force_used": bool(flags),
                    "known_manual_link_preserved": destination == legacy,
                }
            )
        forwarded_version = run(
            [str(visible_command), "--version"],
            old_env,
            args.output / "front-of-path-version.log",
        )
        assert forwarded_version == "airs " + launcher["version"]
        results.append(
            {
                "case": "legacy-command-before-separate-npm-prefix",
                "passed": True,
                "binary_sha256": native_info(prefix)[1]["binary_sha256"],
                "uninstall_used": False,
                "manual_command_removal": False,
                "force_used": False,
            }
        )
        assert os.readlink(command) == old_link
        assert hashlib.sha256(preserved_binary.read_bytes()).hexdigest() == old_hash
        after = {
            str(p.relative_to(state)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in state.rglob("*")
            if p.is_file() and p.relative_to(state).parts[:2] != ("tmp", "arg0")
        }
        (args.output / "state-after.json").write_text(json.dumps(after, indent=2))
        assert before == after, str(
            {
                "changed_state_files": sorted(
                    k
                    for k in before.keys() | after.keys()
                    if before.get(k) != after.get(k)
                )
            }
        )
        assert not unexpected and not redirects
        receipt = {
            "passed": True,
            "previous": args.previous,
            "version": launcher["version"],
            "platform": os.uname().sysname,
            "configuration_preserved": True,
            "legacy_target_preserved": True,
            "cases": results,
            "registry_requests": requests,
        }
        (args.output / "UPGRADE.json").write_text(json.dumps(receipt, indent=2) + "\n")
        print(json.dumps(receipt))
    finally:
        server.shutdown()


if __name__ == "__main__":
    main()
