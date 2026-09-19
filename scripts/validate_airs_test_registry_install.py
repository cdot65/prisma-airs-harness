#!/usr/bin/env python3
"""Fresh anonymous npm installation of exact, previously staged test packages."""

import argparse
import json
from pathlib import Path
import platform
import stat
import subprocess
from urllib.parse import urlsplit

from airs_bundle import verify_bundle
from airs_npm_registry import install_environment
from airs_release_receipts import atomic_json, evidence_path, safe_destination
from airs_test_release_archive import inspect_archive
from airs_test_release_publish import Registry
from airs_test_release_spec import (
    PACKAGE_ORDER,
    TARGETS,
    digest_file,
    load_json,
    load_spec,
    require,
)


def host_target():
    machine = platform.machine().lower()
    target = {
        ("Linux", "x86_64"): "x86_64-unknown-linux-musl",
        ("Linux", "aarch64"): "aarch64-unknown-linux-musl",
        ("Linux", "arm64"): "aarch64-unknown-linux-musl",
        ("Darwin", "arm64"): "aarch64-apple-darwin",
    }.get((platform.system(), machine))
    require(target in TARGETS, "Registry acceptance requires a supported native host")
    return target


def verify_installed_files(package, inventory):
    """Compare every shipped file, including launcher and bundled CLI content."""
    for relative, expected in inventory["members"].items():
        if expected["type"] == "dir":
            continue
        path = evidence_path(package, relative.removeprefix("package/"))
        require(
            digest_file(path) == expected["sha256"],
            "Installed package file differs from staged bytes",
        )
        require(
            stat.S_IMODE(path.stat().st_mode) & 0o111 == expected["mode"] & 0o111,
            "Installed executable mode differs from staged package",
        )


def verify_registry_metadata(spec, records, registry):
    expected_origin = urlsplit(spec["registry"])
    for record in records:
        document = registry.metadata(record["name"])
        require(
            document.get("dist-tags", {}).get(spec["tag"]) == spec["version"],
            "Published mcp channel does not select the accepted version",
        )
        published = document["versions"].get(spec["version"], {})
        require(
            published.get("name") == record["name"]
            and published.get("version") == spec["version"],
            "Published package identity mismatch",
        )
        distribution = published.get("dist", {})
        require(
            distribution.get("integrity") == record["integrity"],
            "Published immutable integrity mismatch",
        )
        location = urlsplit(distribution.get("tarball", ""))
        require(
            location.scheme == "https"
            and location.netloc == expected_origin.netloc
            and location.username is None
            and location.password is None
            and not location.query
            and not location.fragment
            and location.path.startswith(expected_origin.path.rstrip("/") + "/"),
            "Published archive must stay on the selected HTTPS registry",
        )


def install(spec, packages, prefix):
    target = host_target()
    expected_native = next(
        row["binary_sha256"] for row in spec["platforms"] if row["target"] == target
    )
    packages, prefix = Path(packages).resolve(), safe_destination(prefix)
    require(
        not prefix.resolve().is_relative_to(packages)
        and not packages.is_relative_to(prefix.resolve()),
        "Installation output must not overlap staged inputs",
    )
    plan = load_json(evidence_path(packages, "NPM-PACKAGES.json"))
    records = plan.get("publish_order")
    require(
        isinstance(records, list) and [r.get("name") for r in records] == PACKAGE_ORDER,
        "Registry install requires exactly three natives and one launcher",
    )
    inventories = {}
    selected = ["airs-harness", TARGETS[target]]
    for record in records:
        require(
            record["version"] == spec["version"]
            and record["filename"] == f"{record['name']}-{spec['version']}.tgz",
            "Staged registry package identity mismatch",
        )
        if record["name"] in selected:
            inventory = inspect_archive(
                evidence_path(packages, "tarballs/" + record["filename"])
            )
            require(
                inventory["sha256"] == record["sha256"]
                and inventory["integrity"] == record["integrity"],
                "Staged registry archive integrity mismatch",
            )
            inventories[record["name"]] = inventory
    prefix.mkdir(parents=True, exist_ok=False)
    environment = install_environment(prefix, spec["registry"], False)
    environment = {
        key: value
        for key, value in environment.items()
        if not key.startswith(("AIRS_", "OPENAI_"))
        and key
        not in ("CODEX_HOME", "CODEX_SQLITE_HOME", "NODE_OPTIONS", "PYTHONOPTIMIZE")
    }
    environment.update(NPM_CONFIG_LOGS_MAX="0", NPM_CONFIG_LOGLEVEL="error")
    # The empty userconfig is used for anonymous metadata reads too. It never
    # receives the publication credential or the caller's npm configuration.
    userconfig = prefix / "empty.npmrc"
    userconfig.chmod(0o600)
    registry = Registry(spec["registry"], userconfig, prefix)
    verify_registry_metadata(spec, records, registry)
    result = subprocess.run(
        [
            "npm",
            "install",
            "--global",
            "--prefix",
            str(prefix),
            "--ignore-scripts",
            "--no-audit",
            "--no-fund",
            "--registry",
            spec["registry"],
            "airs-harness@" + spec["version"],
        ],
        cwd=prefix,
        env=environment,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        text=True,
        timeout=300,
    )
    network = {
        "anonymous_fresh_install": True,
        "isolated_npm_configuration": True,
        "fresh_npm_cache": True,
        "npm_exit_code": result.returncode,
        "registry": spec["registry"],
        "published_archive_integrities_verified": True,
        "scope": "npm configuration isolation; network packet capture is not claimed",
    }
    atomic_json(prefix / "INSTALL-NETWORK.json", network)
    require(
        result.returncode == 0,
        "Anonymous npm installation failed; check registry availability and prerequisites",
    )
    launcher = prefix / "lib/node_modules/airs-harness"
    native_package = launcher / "node_modules" / TARGETS[target]
    verify_installed_files(launcher, inventories["airs-harness"])
    verify_installed_files(native_package, inventories[TARGETS[target]])
    native = native_package / "bin/airs-harness"
    native_hash = digest_file(native)
    info = load_json(native_package / "BUILD-INFO.json")
    require(
        native_hash == expected_native == info.get("binary_sha256")
        and info.get("source_commit") == spec["source_commit"]
        and info.get("version") == spec["version"]
        and info.get("target") == target,
        "Installed native provenance mismatch",
    )
    inventory_path = launcher / "BUNDLE-INVENTORY.json"
    require(
        digest_file(inventory_path)
        == plan.get("cli_bundle", {}).get("inventory_sha256"),
        "Installed managed CLI inventory mismatch",
    )
    bundle = verify_bundle(
        launcher, load_json(inventory_path), require_windows_wrappers=False
    )
    manifest = load_json(launcher / "package.json")
    require(
        manifest.get("optionalDependencies")
        == {name: spec["version"] for name in TARGETS.values()},
        "Installed native dependency set changed",
    )
    command = prefix / "bin/airs"
    version = subprocess.check_output(
        [str(command), "--version"], env=environment, text=True, timeout=30
    ).strip()
    require(version == "airs " + spec["version"], "Installed AIRS version mismatch")
    cli_version = subprocess.check_output(
        [str(command), "cli", "--version"], env=environment, text=True, timeout=30
    ).strip()
    require(
        cli_version == manifest["dependencies"]["@cdot65/prisma-airs-cli"],
        "Installed product CLI version mismatch",
    )
    tooling = load_json(launcher / "PACKAGE-TOOLING.json")
    require(
        tooling == plan.get("package_tooling"), "Installed packaging tooling mismatch"
    )
    verify_registry_metadata(spec, records, registry)
    receipt = {
        **network,
        "passed": True,
        "published": True,
        "version": version,
        "mcp_channel_verified_before_and_after_install": True,
        "source_commit": spec["source_commit"],
        "binary_sha256": native_hash,
        "launcher_package": "airs-harness",
        "native_package": TARGETS[target],
        "package_tooling": tooling,
        "prisma_airs_cli_version": cli_version,
        "bundle_verification": bundle,
        "command": str(command),
        "platform": platform.system(),
        "architecture": platform.machine(),
    }
    atomic_json(prefix / "INSTALL-VERIFICATION.json", receipt)
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, required=True)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument("--prefix", type=Path, required=True)
    args = parser.parse_args()
    print(
        json.dumps(install(load_spec(args.spec), args.packages, args.prefix), indent=2)
    )


if __name__ == "__main__":
    main()
