"""Real npm network boundary checks; fixtures do not claim native acceptance."""

import hashlib
from http.server import ThreadingHTTPServer
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import threading
import unittest
from unittest.mock import patch
from urllib.request import urlopen
from urllib.error import HTTPError

from airs_npm_registry import install_environment, registry_handler

ROOT = Path(__file__).resolve().parents[1]


class RegistryContracts(unittest.TestCase):
    def test_bundle_serves_only_staged_paths_and_preserves_legacy_redirect(self):
        for bundled in (True, False):
            with self.subTest(bundled=bundled), tempfile.TemporaryDirectory() as tmp:
                archive = Path(tmp) / "package.tgz"
                archive.write_bytes(b"fixture")
                requests, unexpected, redirects = [], [], []
                server = ThreadingHTTPServer(
                    ("127.0.0.1", 0),
                    registry_handler(
                        {"/fixture": {"name": "fixture"}},
                        {"/fixture.tgz": archive},
                        requests,
                        unexpected,
                        redirects,
                        bundled,
                    ),
                )
                thread = threading.Thread(target=server.serve_forever)
                thread.start()
                try:
                    base = f"http://127.0.0.1:{server.server_port}"
                    with urlopen(base + "/fixture") as response:
                        self.assertEqual(json.load(response), {"name": "fixture"})
                    with urlopen(base + "/fixture.tgz") as response:
                        self.assertEqual(response.read(), b"fixture")
                    if bundled:
                        with self.assertRaises(HTTPError) as error:
                            urlopen(base + "/missing")
                        self.assertEqual(error.exception.code, 404)
                        error.exception.close()
                        self.assertEqual(unexpected, ["/missing"])
                    else:
                        # No external traffic: inspect the redirect without following it.
                        import http.client

                        connection = http.client.HTTPConnection(
                            "127.0.0.1", server.server_port
                        )
                        connection.request("GET", "/missing")
                        response = connection.getresponse()
                        self.assertEqual(response.status, 302)
                        self.assertEqual(
                            response.getheader("Location"),
                            "https://registry.npmjs.org/missing",
                        )
                        response.read()
                        connection.close()
                        self.assertEqual(redirects, ["/missing"])
                finally:
                    server.shutdown()
                    server.server_close()
                    thread.join()

    def test_bundle_environment_replaces_proxy_and_registry_credentials(self):
        with (
            tempfile.TemporaryDirectory() as tmp,
            patch.dict(
                os.environ,
                {
                    "hTtPs_PrOxY": "https://secret:canary@invalid",
                    "no_proxy": "*",
                    "Npm_Config_Registry": "https://invalid",
                    "NODE_AUTH_TOKEN": "canary",
                },
            ),
        ):
            env = install_environment(Path(tmp), "http://127.0.0.1:1234", True)
            self.assertNotIn("canary", json.dumps(env))
            self.assertEqual(env["NO_PROXY"], "")
            self.assertEqual(env["NPM_CONFIG_HTTPS_PROXY"], "http://127.0.0.1:1234")
            self.assertNotIn("no_proxy", env)

    def test_tampered_native_is_rejected_before_any_native_execution(self):
        import validate_airs_npm

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            packages = root / "packages"
            launcher = packages / "airs-harness"
            launcher.mkdir(parents=True)
            manifest = {"name": "airs-harness", "version": "0.0.0-fixture"}
            (launcher / "package.json").write_text(json.dumps(manifest))
            (packages / "NPM-PACKAGES.json").write_text(
                json.dumps(
                    {
                        "publish_order": [
                            {
                                **manifest,
                                "filename": "unused.tgz",
                                "integrity": "fixture",
                            }
                        ]
                    }
                )
            )
            native = root / "installed-native"
            (native / "bin").mkdir(parents=True)
            executable = (
                native
                / "bin"
                / ("airs-harness.exe" if os.name == "nt" else "airs-harness")
            )
            executable.write_bytes(b"tampered bytes must not execute")
            (native / "BUILD-INFO.json").write_text(
                json.dumps({"binary_sha256": "0" * 64})
            )
            (native / "package.json").write_text('{"name":"airs-harness-linux-x64"}')
            calls = []

            def check_output(command, **_kwargs):
                calls.append(command)
                if command[1:] == ["--version"] and Path(command[0]).name.lower() in (
                    "node",
                    "node.exe",
                    "npm",
                    "npm.cmd",
                ):
                    return "fixture-version\n"
                if "--input-type=module" in command:
                    return str(native / "package.json") + "\n"
                raise AssertionError("Native command ran before verification")

            with (
                patch.object(
                    sys,
                    "argv",
                    [
                        "validate",
                        "--packages",
                        str(packages),
                        "--prefix",
                        str(root / "install"),
                    ],
                ),
                patch.object(
                    validate_airs_npm.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess([], 0, "", ""),
                ),
                patch.object(
                    validate_airs_npm.subprocess, "check_output", check_output
                ),
            ):
                with self.assertRaisesRegex(
                    ValueError, "differs from build provenance"
                ):
                    validate_airs_npm.main()
            self.assertEqual(len(calls), 3)
            self.assertFalse((root / "install/INSTALL-VERIFICATION.json").exists())

    @unittest.skipUnless(shutil.which("npm"), "npm is required for install contracts")
    def test_missing_optional_and_direct_url_dependency_fail_without_public_fetch(self):
        for specification in (
            "1.0.0",
            "https://registry.npmjs.org/airs-bundle-missing-fixture/-/fixture.tgz",
            "https://outside.invalid/fixture.tgz",
            "http://127.0.0.1:9/fixture.tgz",
        ):
            with (
                self.subTest(specification=specification),
                tempfile.TemporaryDirectory() as tmp,
            ):
                root = Path(tmp)
                packages = root / "packages"
                launcher = packages / "@cdot65/prisma-airs-harness"
                launcher.mkdir(parents=True)
                from airs_bundle import CLI, SDK, sharp_packages, verify_bundle

                targets = ["aarch64-apple-darwin"]
                versions = {
                    CLI: "5.2.0",
                    SDK: "0.28.0",
                    **{name: "1.0.0" for name in sharp_packages(targets)},
                }
                missing = "airs-bundle-missing-fixture"
                manifest = {
                    "name": "@cdot65/prisma-airs-harness",
                    "version": "0.0.0-fixture",
                    "dependencies": versions,
                    "bundleDependencies": list(versions),
                    "optionalDependencies": {missing: specification},
                }
                (launcher / "package.json").write_text(json.dumps(manifest))
                inventory = {
                    "schema_version": 1,
                    "source_lock_sha256": "a" * 64,
                    "targets": targets,
                    "required_pins": {CLI: "5.2.0", SDK: "0.28.0"},
                    "packages": [],
                }
                for name, version in {**versions, missing: "1.0.0"}.items():
                    directory = launcher / "node_modules" / name
                    directory.mkdir(parents=True)
                    files = {
                        "package.json": json.dumps(
                            {"name": name, "version": version}
                        ).encode(),
                        "LICENSE": b"fixture license",
                    }
                    for relative, body in files.items():
                        (directory / relative).write_bytes(body)
                    inventory["packages"].append(
                        {
                            "path": "node_modules/" + name,
                            "name": name,
                            "version": version,
                            "files": {
                                relative: hashlib.sha256(body).hexdigest()
                                for relative, body in files.items()
                            },
                            "license_files": ["LICENSE"],
                        }
                    )
                baseline = verify_bundle(launcher, inventory)
                self.assertEqual(baseline["packages"], len(versions) + 1)
                # The complete source bundle is valid. Only this optional package
                # is omitted from the tarball; every other byte is preserved.
                inventory_bytes = json.dumps(inventory).encode()
                (launcher / "BUNDLE-INVENTORY.json").write_bytes(inventory_bytes)
                (packages / "tarballs").mkdir()
                archive = packages / "tarballs/launcher.tgz"
                with tarfile.open(archive, "w:gz") as tar:
                    for file in sorted(launcher.rglob("*")):
                        if (
                            not file.is_file()
                            or missing in file.relative_to(launcher).parts
                        ):
                            continue
                        tar.add(
                            file,
                            arcname="package/" + file.relative_to(launcher).as_posix(),
                        )
                import base64

                integrity = (
                    "sha512-"
                    + base64.b64encode(
                        hashlib.sha512(archive.read_bytes()).digest()
                    ).decode()
                )
                (packages / "NPM-PACKAGES.json").write_text(
                    json.dumps(
                        {
                            "cli_bundle": {
                                "inventory_sha256": hashlib.sha256(
                                    inventory_bytes
                                ).hexdigest()
                            },
                            "publish_order": [
                                {
                                    "name": manifest["name"],
                                    "version": manifest["version"],
                                    "filename": archive.name,
                                    "integrity": integrity,
                                }
                            ],
                        }
                    )
                )
                result = subprocess.run(
                    [
                        sys.executable,
                        str(ROOT / "scripts/validate_airs_npm.py"),
                        "--packages",
                        str(packages),
                        "--prefix",
                        str(root / "install"),
                    ],
                    capture_output=True,
                    text=True,
                    timeout=60,
                )
                self.assertNotEqual(result.returncode, 0)
                receipt = json.loads(
                    (root / "install/INSTALL-NETWORK.json").read_text()
                )
                if receipt["unexpected_requests"]:
                    self.assertIn(
                        "attempted an unstaged dependency request", result.stderr
                    )
                    expected_denial = (
                        "/" + missing
                        if specification == "1.0.0"
                        else "/airs-bundle-missing-fixture/-/fixture.tgz"
                        if specification.startswith("https://registry.npmjs.org")
                        else "CONNECT [external target]"
                        if specification.startswith("https")
                        else "GET [external or queried target]"
                    )
                    self.assertEqual(
                        set(receipt["unexpected_requests"]), {expected_denial}
                    )
                else:
                    # npm12 rejects optional remote URLs before fetching and can
                    # still exit0. The required inventory catches the omission.
                    self.assertTrue(specification.startswith("http"))
                    self.assertGreaterEqual(
                        int(receipt["npm_version"].split(".")[0]), 12
                    )
                    self.assertEqual(receipt["npm_exit_code"], 0)
                    self.assertIn("FileNotFoundError", result.stderr)
                    self.assertIn("node_modules", result.stderr)
                    self.assertIn("airs-bundle-missing-fixture", result.stderr)
                    self.assertIn("package.json", result.stderr)
                self.assertEqual(receipt["public_dependency_redirects"], [])
                self.assertFalse((root / "install/INSTALL-VERIFICATION.json").exists())
                if evidence_directory := os.environ.get("AIRS_NPM_FIXTURE_EVIDENCE"):
                    evidence = Path(evidence_directory)
                    evidence.mkdir(parents=True, exist_ok=True)
                    category = (
                        "registry"
                        if specification == "1.0.0"
                        else "npm-url"
                        if "registry.npmjs.org" in specification
                        else "https"
                        if specification.startswith("https")
                        else "http"
                    )
                    (
                        evidence
                        / ("npm" + receipt["npm_version"] + "-" + category + ".json")
                    ).write_text(
                        json.dumps(
                            {
                                "baseline_verified": baseline,
                                "omitted_package": missing,
                                "network": receipt,
                                "failed_as_required": True,
                                "failure_stage": "network"
                                if receipt["unexpected_requests"]
                                else "installed-inventory",
                                "native_executed": False,
                            },
                            indent=2,
                        )
                        + "\n"
                    )


if __name__ == "__main__":
    unittest.main()
