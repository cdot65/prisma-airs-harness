"""Disposable npm registry and rejecting proxy for private package validation."""

from http.server import BaseHTTPRequestHandler
import json
import os
import shutil
from urllib.parse import quote, unquote, urlsplit


PROXY_VARIABLES = {"HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY"}


def install_environment(prefix, registry_url, bundled):
    env = {
        key: value
        for key, value in os.environ.items()
        if not key.upper().startswith(("NPM_", "NODE_AUTH_TOKEN"))
        and (not bundled or key.upper() not in PROXY_VARIABLES)
    }
    for filename in ("empty.npmrc", "empty-global.npmrc"):
        (prefix / filename).write_text("")
    env.update(
        NPM_CONFIG_USERCONFIG=str(prefix / "empty.npmrc"),
        NPM_CONFIG_GLOBALCONFIG=str(prefix / "empty-global.npmrc"),
        NPM_CONFIG_CACHE=str(prefix / "npm-cache"),
        NPM_CONFIG_UPDATE_NOTIFIER="false",
        NPM_CONFIG_FETCH_RETRIES="0",
        NPM_CONFIG_FETCH_TIMEOUT="15000",
    )
    if bundled:
        # npm obeys these proxy settings for explicit URL dependencies too.
        # Even registry traffic uses this proxy, which serves only its own
        # staged paths without forwarding. This is not an OS network sandbox.
        env.update(
            HTTP_PROXY=registry_url,
            HTTPS_PROXY=registry_url,
            ALL_PROXY=registry_url,
            NO_PROXY="",
            NPM_CONFIG_PROXY=registry_url,
            NPM_CONFIG_HTTPS_PROXY=registry_url,
            NPM_CONFIG_NOPROXY="",
        )
    return env


def registry_handler(metadata, archives, requests, unexpected, redirects, bundled):
    class Registry(BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass

        def send_error(self, code, message=None, explain=None):
            if code == 501:
                # Unsupported methods must not disappear as optional npm errors.
                requests.append(self.command + " [unsupported method]")
                unexpected.append(self.command + " [unsupported method]")
            super().send_error(code, message, explain)

        def do_CONNECT(self):
            # Never forward or retain proxy authority/credentials in evidence.
            requests.append("CONNECT [external target]")
            unexpected.append("CONNECT [external target]")
            self.send_error(403)

        def do_GET(self):
            parsed = urlsplit(self.path)
            path = unquote(parsed.path)
            local_origin = f"127.0.0.1:{self.server.server_port}"
            if bundled and (
                parsed.query
                or (
                    parsed.netloc
                    and (parsed.scheme != "http" or parsed.netloc != local_origin)
                )
            ):
                requests.append("GET [external or queried target]")
                unexpected.append("GET [external or queried target]")
                self.send_error(403)
                return
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
                if bundled or path.startswith(
                    (
                        "/airs-harness",
                        "/prisma-airs-harness",
                        "/@cdot65/prisma-airs-harness",
                    )
                ):
                    unexpected.append(path)
                    self.send_error(404)
                else:
                    redirects.append(path)
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

    return Registry
