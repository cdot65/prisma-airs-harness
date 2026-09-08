#!/usr/bin/env python3
"""Install prepared npm tarballs through a loopback registry without publishing.

The resulting prefix is retained for executable and live acceptance tests. This
tests registry dependency resolution and npm's command link, not just npm pack.
"""

import argparse
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shutil
import subprocess
import threading
from urllib.parse import quote, unquote, urlsplit


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument("--prefix", type=Path, required=True)
    args = parser.parse_args()
    packages = args.packages.resolve(strict=True)
    prefix = args.prefix.resolve()
    prefix.mkdir(parents=True, exist_ok=False)
    package_receipt = json.loads((packages / "NPM-PACKAGES.json").read_text())
    records = package_receipt["publish_order"]
    launchers = [
        r
        for r in records
        if r["name"] in ("airs-harness", "@cdot65/prisma-airs-harness")
    ]
    if len(launchers) != 1:
        raise ValueError("Expected exactly one declared harness launcher")
    launcher_record = launchers[0]
    metadata = {}
    archives = {}
    requests = []

    class Registry(BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass

        def do_GET(self):
            path = unquote(urlsplit(self.path).path)
            requests.append(path)
            if path in archives:
                archive = archives[path]
                self.send_response(200)
                self.send_header("Content-Type", "application/octet-stream")
                self.send_header("Content-Length", str(archive.stat().st_size))
                self.end_headers()
                with archive.open("rb") as source:
                    shutil.copyfileobj(source, self.wfile)
                return
            if path not in metadata:
                # First-party names must resolve only to this candidate. Real
                # CLI/transitive dependencies come from the public npm registry.
                if path.startswith(("/airs-harness", "/@cdot65/prisma-airs-harness")):
                    self.send_error(404)
                else:
                    self.send_response(302)
                    self.send_header(
                        "Location",
                        "https://registry.npmjs.org" + quote(path, safe="/@"),
                    )
                    self.end_headers()
                return
            body = json.dumps(metadata[path]).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    server = ThreadingHTTPServer(("127.0.0.1", 0), Registry)
    registry_url = f"http://127.0.0.1:{server.server_port}"
    for record in records:
        name, version = record["name"], record["version"]
        manifest = json.loads((packages / name / "package.json").read_text())
        path = f"/{name}/-/{record['filename']}"
        manifest["dist"] = {
            "tarball": registry_url + path,
            "integrity": record["integrity"],
        }
        metadata["/" + name] = {
            "name": name,
            "dist-tags": {"latest": version},
            "versions": {version: manifest},
        }
        archives[path] = packages / "tarballs" / record["filename"]
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        config = prefix / "empty.npmrc"
        config.write_text("")
        global_config = prefix / "empty-global.npmrc"
        global_config.write_text("")
        env = {
            key: value
            for key, value in os.environ.items()
            if not key.upper().startswith(("NPM_", "NODE_AUTH_TOKEN"))
        }
        env.update(
            NPM_CONFIG_USERCONFIG=str(config),
            NPM_CONFIG_GLOBALCONFIG=str(global_config),
            NPM_CONFIG_CACHE=str(prefix / "npm-cache"),
        )
        result = subprocess.run(
            [
                shutil.which("npm") or "npm",
                "install",
                "--global",
                "--prefix",
                str(prefix),
                "--ignore-scripts",
                "--no-audit",
                "--no-fund",
                "--registry",
                registry_url,
                launcher_record["name"] + "@" + launcher_record["version"],
            ],
            env=env,
            cwd=prefix,
            capture_output=True,
            text=True,
            timeout=300,
        )
        (prefix / "npm-install.log").write_text(result.stdout + result.stderr)
        if result.returncode:
            raise RuntimeError("npm installation failed; inspect npm-install.log")
        command = prefix / (
            "airs-harness.cmd" if os.name == "nt" else "bin/airs-harness"
        )
        version = subprocess.check_output(
            [str(command), "--version"], text=True
        ).strip()
        expected = "airs-harness " + launcher_record["version"]
        if version != expected:
            raise ValueError(
                "Installed command did not launch the expected native version"
            )
        modules = prefix / ("node_modules" if os.name == "nt" else "lib/node_modules")
        launcher_directory = modules / launcher_record["name"]
        native_manifest = subprocess.check_output(
            [
                shutil.which("node") or "node",
                "--input-type=module",
                "-e",
                """
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
const require = createRequire(process.argv[1]);
const manifest = require(process.argv[1]);
const { platformPackage } = await import(pathToFileURL(path.join(path.dirname(process.argv[1]), 'lib/launcher.js')));
console.log(require.resolve(platformPackage(process.platform, process.arch, manifest) + '/package.json'));
""",
                str(launcher_directory / "package.json"),
            ],
            text=True,
            timeout=10,
        ).strip()
        native_info = Path(native_manifest).parent / "BUILD-INFO.json"
        provenance = json.loads(native_info.read_text())
        native = (
            native_info.parent
            / "bin"
            / ("airs-harness.exe" if os.name == "nt" else "airs-harness")
        )
        with native.open("rb") as stream:
            native_digest = hashlib.file_digest(stream, "sha256").hexdigest()
        if native_digest != provenance["binary_sha256"]:
            raise ValueError("Installed native binary differs from build provenance")
        cli_version = subprocess.check_output(
            [str(command), "airs", "--version"], text=True
        ).strip()
        launcher_manifest = json.loads(
            (launcher_directory / "package.json").read_text()
        )
        if cli_version != launcher_manifest["dependencies"]["@cdot65/prisma-airs-cli"]:
            raise ValueError("Installed Prisma AIRS CLI differs from its exact pin")
        tooling_file = launcher_directory / "PACKAGE-TOOLING.json"
        package_tooling = None
        if "package_tooling" in package_receipt or tooling_file.exists():
            package_tooling = json.loads(tooling_file.read_text())
            if package_tooling != package_receipt.get("package_tooling"):
                raise ValueError(
                    "Installed package tooling differs from its staging receipt"
                )
        receipt = {
            "launcher_package": launcher_record["name"],
            "native_package": json.loads(Path(native_manifest).read_text())["name"],
            "package_tooling": package_tooling,
            "prisma_airs_cli_version": cli_version,
            "passed": True,
            "binary_sha256": native_digest,
            "source_commit": provenance["source_commit"],
            "published": False,
            "version": version,
            "command": str(command),
            "registry_requests": requests,
        }
        (prefix / "INSTALL-VERIFICATION.json").write_text(
            json.dumps(receipt, indent=2) + "\n"
        )
        print(json.dumps(receipt, indent=2))
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


if __name__ == "__main__":
    main()
