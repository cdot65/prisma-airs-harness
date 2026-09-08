"""Nonsecret provenance and private JWT observation for live OIDC acceptance."""

import base64
import hashlib
import json
import subprocess
import time
from pathlib import Path


def jwt_expiry(token, subject, audience):
    """Extract timing from a runtime-verified helper token without exposing it."""
    try:
        if not isinstance(token, str) or len(token) > 16384:
            raise ValueError()
        parts = token.split(".")
        if len(parts) != 3:
            raise ValueError()
        claims = json.loads(base64.urlsafe_b64decode(parts[1] + "=="))
        expiry = claims["exp"]
        if (
            claims["sub"] != subject
            or claims["aud"] != audience
            or type(expiry) is not int
            or not time.time() < expiry <= time.time() + 600
        ):
            raise ValueError()
        return expiry
    except (ValueError, TypeError, KeyError, AttributeError):
        raise AssertionError("Invalid private credential observation") from None


def observe_expiries(run, credential, mcp_credential, subject):
    """Helpers may rotate near-expiry records; no helper output enters receipts."""
    try:
        inference = run(*credential, timeout=30)
        mcp = run(*mcp_credential, timeout=30)
        if inference.returncode or mcp.returncode:
            raise ValueError()
        if len(inference.stdout) > 16385 or len(mcp.stdout) > 32768:
            raise ValueError()
        mcp_token = json.loads(mcp.stdout)["x-portkey-api-key"]
        return {
            "inference": jwt_expiry(
                inference.stdout.strip(), subject, "airs-terminal-inference"
            ),
            "mcp": jwt_expiry(mcp_token, subject, "airs-terminal-security"),
        }
    except (ValueError, TypeError, KeyError, subprocess.TimeoutExpired):
        raise AssertionError("Private credential observation failed") from None


def invocation_provenance(command):
    """Resolve native bytes with the installed launcher's side-effect-free API."""
    command = Path(command).absolute()
    resolved = command.resolve(strict=True)
    with resolved.open("rb") as stream:
        invocation_digest = hashlib.file_digest(stream, "sha256").hexdigest()
    native = resolved
    source = None
    if resolved.suffix == ".js":
        library = resolved.parent.parent / "lib/launcher.js"
        program = """
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
const url = pathToFileURL(process.argv[1]);
const { platformPackage } = await import(url.href);
const require = createRequire(url);
const manifest = require.resolve(platformPackage(process.platform, process.arch) + '/package.json');
console.log(path.join(path.dirname(manifest), 'bin', process.platform === 'win32' ? 'airs-harness.exe' : 'airs-harness'));
"""
        result = subprocess.run(
            ["node", "--input-type=module", "-e", program, str(library)],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if result.returncode or result.stderr:
            raise AssertionError("Installed native resolution failed")
        native = Path(result.stdout.strip()).resolve(strict=True)
        info = json.loads((native.parent.parent / "BUILD-INFO.json").read_text())
        source = info["source_commit"]
    with native.open("rb") as stream:
        native_digest = hashlib.file_digest(stream, "sha256").hexdigest()
    if resolved.suffix == ".js" and native_digest != info["binary_sha256"]:
        raise AssertionError("Installed native differs from its build provenance")
    return {
        "invocation_path": str(command),
        "invocation_resolved_path": str(resolved),
        "invocation_sha256": invocation_digest,
        "native_path": str(native),
        "binary_sha256": native_digest,
        "source_commit": source,
    }
