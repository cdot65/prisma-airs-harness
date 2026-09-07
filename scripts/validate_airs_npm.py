#!/usr/bin/env python3
"""Install prepared npm tarballs through a loopback registry without publishing.

The resulting prefix is retained for executable and live acceptance tests. This
tests registry dependency resolution and npm's command link, not just npm pack.
"""

import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shutil
import subprocess
import threading
from urllib.parse import unquote, urlsplit


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packages", type=Path, required=True)
    parser.add_argument("--prefix", type=Path, required=True)
    args = parser.parse_args()
    packages = args.packages.resolve(strict=True)
    prefix = args.prefix.resolve()
    prefix.mkdir(parents=True, exist_ok=False)
    records = json.loads((packages / "NPM-PACKAGES.json").read_text())["publish_order"]
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
                self.send_error(404)
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
        env = dict(
            os.environ,
            NPM_CONFIG_USERCONFIG=str(config),
            NPM_CONFIG_CACHE=str(prefix / "npm-cache"),
        )
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
                registry_url,
                "airs-harness@" + records[-1]["version"],
            ],
            env=env,
            capture_output=True,
            text=True,
            timeout=180,
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
        expected = "airs-harness " + records[-1]["version"]
        if version != expected:
            raise ValueError(
                "Installed command did not launch the expected native version"
            )
        receipt = {
            "passed": True,
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
