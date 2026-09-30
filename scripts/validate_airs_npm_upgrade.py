#!/usr/bin/env python3
"""Exercise npm upgrades and legacy-command migration without forcing npm.

A previous release published under the old launcher name is replaced by name:
the old package is uninstalled and the renamed launcher installed into the same
prefix. Same-name upgrades never uninstall.

The old version comes from the owned registry. Candidate archives are served by
the existing isolated test registry; all new native bytes must match provenance.
Only disposable prefixes and harness state are used.
"""

import argparse
import hashlib
import json
import os
from urllib.parse import urlsplit
import subprocess
import threading
import sys
import signal
from http.server import ThreadingHTTPServer
from pathlib import Path

from airs_npm_registry import install_environment, registry_handler
from airs_release_receipts import evidence_path
from airs_test_release_archive import inspect_archive
from airs_test_release_spec import COMMIT, LAUNCHER, SHA256, require
from airs_npm_versions import released_version
from airs_native_test_store import native_test_command
from airs_npm_roundtrip import link_package, observe_roundtrip


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


def installed_launcher(prefix, names):
    """Exactly one of the launcher package names must be installed in the prefix."""
    present = [name for name in names if (prefix / "lib/node_modules" / name).is_dir()]
    require(len(present) == 1, "Expected exactly one installed launcher package")
    return present[0]


def uninstall(prefix, name, registry, log):
    """Remove a launcher package by name without --force; report whether it existed."""
    if not (prefix / "lib/node_modules" / name).is_dir():
        return False
    run(
        [
            "npm",
            "uninstall",
            "-g",
            "--prefix",
            str(prefix),
            "--ignore-scripts",
            "--no-audit",
            "--no-fund",
            "--registry=" + registry,
            name,
        ],
        install_environment(prefix, registry, False),
        log,
    )
    require(
        not (prefix / "lib/node_modules" / name).exists(),
        "npm uninstall left the previous launcher package behind",
    )
    return True


def native_info(prefix, package):
    manifest = prefix / "lib/node_modules" / package / "package.json"
    path = subprocess.check_output(
        [
            "node",
            "-e",
            (
                "const {createRequire}=require('module');const fs=require('fs');"
                "const r=createRequire(process.argv[1]);"
                "const m=JSON.parse(fs.readFileSync(process.argv[1],'utf8'));"
                "const legacy='airs-harness-'+process.platform+'-'+process.arch;"
                "const deps=m.optionalDependencies||{};"
                "const name=[legacy,'@cdot65/prisma-'+legacy].find(n=>Object.hasOwn(deps,n))||legacy;"
                "console.log(r.resolve(name+'/package.json'));"
            ),
            str(manifest),
        ],
        text=True,
    ).strip()
    package = Path(path).parent
    native = package / "bin/airs-harness"
    return native, json.loads((package / "BUILD-INFO.json").read_text())


def archive_manifest(packages, record):
    archive = evidence_path(packages, "tarballs/" + record["filename"])
    inventory = inspect_archive(archive)
    require(
        inventory["sha256"] == record["sha256"]
        and inventory["integrity"] == record["integrity"],
        "Upgrade archive integrity mismatch",
    )
    manifest = inventory["json"].get("package/package.json")
    require(
        isinstance(manifest, dict)
        and manifest.get("name") == record["name"]
        and manifest.get("version") == record["version"],
        "Upgrade package identity mismatch",
    )
    return manifest, archive


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--previous", default="0.1.0-alpha.12")
    parser.add_argument(
        "--previous-package",
        help="Launcher package name the previous version was published under",
    )
    parser.add_argument("--registry", default="https://npm.cdot.io")
    parser.add_argument("--previous-native-sha256")
    parser.add_argument("--previous-source-commit")
    parser.add_argument("--native-worker", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    registry = urlsplit(args.registry)
    if (
        registry.scheme != "https"
        or not registry.hostname
        or registry.username
        or registry.password
        or registry.query
        or registry.fragment
    ):
        parser.error(
            "Expected an HTTPS registry URL without credentials, query or fragment"
        )
    args.registry = args.registry.rstrip("/")
    try:
        previous_version = released_version(args.previous)
    except ValueError as error:
        parser.error(str(error))
    previous_package = args.previous_package or previous_version.package
    if previous_package not in ("airs-harness", "prisma-airs-harness", LAUNCHER):
        parser.error("Unknown previous launcher package")
    if previous_version.stable and (
        not SHA256.fullmatch(args.previous_native_sha256 or "")
        or not COMMIT.fullmatch(args.previous_source_commit or "")
    ):
        parser.error(
            "Stable upgrades require pinned previous native SHA256 and source commit"
        )
    if not args.native_worker:
        worker = subprocess.Popen(
            native_test_command(__file__, [*sys.argv[1:], "--native-worker"]),
            start_new_session=True,
        )
        try:
            status = worker.wait(timeout=720)
        except subprocess.TimeoutExpired:
            os.killpg(worker.pid, signal.SIGTERM)
            try:
                worker.wait(timeout=60)
            except subprocess.TimeoutExpired:
                os.killpg(worker.pid, signal.SIGKILL)
                worker.wait(timeout=10)
            raise RuntimeError("Isolated upgrade observation exceeded deadline")
        raise SystemExit(status)

    def interrupted(_signal, _frame):
        raise RuntimeError("Isolated upgrade observation interrupted")

    signal.signal(signal.SIGTERM, interrupted)
    args.output.mkdir(parents=True, exist_ok=False)
    packages = args.packages.resolve(strict=True)
    records = json.loads((packages / "NPM-PACKAGES.json").read_text())["publish_order"]
    launcher = next(record for record in records if record["name"] == LAUNCHER)
    assert launcher["version"] != args.previous
    migration = previous_package != launcher["name"]
    prefix = args.output / "npm managed prefix"
    prefix.mkdir()
    old_env = install_environment(prefix, args.registry, False)
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
            "--registry=" + args.registry,
            previous_package + "@" + args.previous,
        ],
        old_env,
        args.output / "install-previous.log",
    )
    command = prefix / "bin/airs-harness"
    previous_command_name = previous_version.command
    assert (
        run([str(command), "--version"], old_env, args.output / "previous-version.log")
        == previous_command_name + " " + args.previous
    )
    old_native, old_info = native_info(prefix, previous_package)
    old_hash = hashlib.sha256(old_native.read_bytes()).hexdigest()
    assert old_hash == old_info["binary_sha256"]
    if previous_version.stable:
        require(
            old_hash == args.previous_native_sha256
            and old_info["source_commit"] == args.previous_source_commit,
            "Previous package differs from pinned baseline",
        )

    # Preserve a real pre-upgrade harness configuration, not a fabricated marker.
    previous_setup = previous_version.setup
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
        manifest, archive = archive_manifest(packages, record)
        path = f"/{record['name']}/-/{record['filename']}"
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
            # npm refuses to let a differently named package take over the same
            # commands without --force, so a renamed launcher is replaced by name.
            uninstalled = destination == prefix and uninstall(
                destination,
                previous_package,
                registry,
                args.output / (case + "-uninstall.log"),
            )
            assert uninstalled == (migration and destination == prefix)
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
                    launcher["name"] + "@" + launcher["version"],
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
            binary, info = native_info(destination, launcher["name"])
            digest = hashlib.sha256(binary.read_bytes()).hexdigest()
            assert digest == info["binary_sha256"] and digest != old_hash
            results.append(
                {
                    "case": case,
                    "passed": True,
                    "binary_sha256": digest,
                    "uninstall_used": uninstalled,
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
                "binary_sha256": native_info(prefix, launcher["name"])[1][
                    "binary_sha256"
                ],
                "uninstall_used": False,
                "manual_command_removal": False,
                "force_used": False,
            }
        )
        if migration:
            assert link_package(command) == launcher["name"]
        else:
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
        roundtrip = None
        if previous_version.stable:

            def install_phase(phase):
                url = args.registry if phase == "previous" else registry
                version = args.previous if phase == "previous" else launcher["version"]
                name = previous_package if phase == "previous" else launcher["name"]
                other = launcher["name"] if phase == "previous" else previous_package
                removed = migration and uninstall(
                    prefix,
                    other,
                    url,
                    args.output / ("roundtrip-" + phase + "-uninstall.log"),
                )
                environment = install_environment(prefix, url, phase == "candidate")
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
                        "--registry=" + url,
                        name + "@" + version,
                    ],
                    environment,
                    args.output / ("roundtrip-" + phase + "-install.log"),
                )
                for name in ("airs", "airs-harness"):
                    require(
                        run(
                            [str(prefix / "bin" / name), "--version"],
                            environment,
                            args.output / ("roundtrip-" + phase + "-" + name + ".log"),
                        )
                        == "airs " + version,
                        "Roundtrip command reports wrong version",
                    )
                return removed

            names = (previous_package, launcher["name"])
            roundtrip = observe_roundtrip(
                prefix,
                args.previous,
                launcher["version"],
                install_phase,
                lambda root: native_info(root, installed_launcher(root, names)),
                args.previous_native_sha256,
                args.previous_source_commit,
                link_packages=(
                    {"previous": previous_package, "candidate": launcher["name"]}
                    if migration
                    else None
                ),
            )
        receipt = {
            "schema_version": 2,
            "roundtrip": roundtrip,
            "passed": True,
            "previous": args.previous,
            "previous_package": previous_package,
            "version": launcher["version"],
            "package": launcher["name"],
            "launcher_migration": migration,
            "platform": os.uname().sysname,
            "configuration_preserved": True,
            "previous_registry": args.registry,
            "legacy_target_preserved": True,
            "cases": results,
            "registry_requests": requests,
        }
        (args.output / "UPGRADE.json").write_text(json.dumps(receipt, indent=2) + "\n")
        print(json.dumps(receipt))
    finally:
        server.shutdown()


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        # Native PTY exceptions may contain synthetic callback URLs. Keep only
        # bounded phase/type diagnostics; never persist their raw message.
        print(
            json.dumps(
                {
                    "passed": False,
                    "error_type": type(error).__name__,
                    "phase": getattr(observe_roundtrip, "phase", "upgrade"),
                }
            ),
            file=sys.stderr,
        )
        raise SystemExit(1)
