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

    @unittest.skipUnless(shutil.which("npm"), "npm is required for install contracts")
    def test_missing_optional_and_direct_url_dependency_fail_without_public_fetch(self):
        for specification in (
            "1.0.0",
            "https://registry.npmjs.org/airs-bundle-missing-fixture/-/fixture.tgz",
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
                manifest = {
                    "name": "@cdot65/prisma-airs-harness",
                    "version": "0.0.0-fixture",
                    "optionalDependencies": {
                        "airs-bundle-missing-fixture": specification
                    },
                }
                (launcher / "package.json").write_text(json.dumps(manifest))
                (packages / "tarballs").mkdir()
                archive = packages / "tarballs/launcher.tgz"
                with tarfile.open(archive, "w:gz") as tar:
                    body = json.dumps(manifest).encode()
                    entry = tarfile.TarInfo("package/package.json")
                    entry.size = len(body)
                    tar.addfile(entry, io.BytesIO(body))
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
                            "cli_bundle": {},
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
                self.assertIn("attempted an unstaged dependency request", result.stderr)
                receipt = json.loads(
                    (root / "install/INSTALL-NETWORK.json").read_text()
                )
                self.assertTrue(receipt["unexpected_requests"])
                self.assertEqual(receipt["public_dependency_redirects"], [])
                self.assertFalse((root / "install/INSTALL-VERIFICATION.json").exists())


if __name__ == "__main__":
    unittest.main()
