"""Verify complete npm Windows command wrappers against retained upstream output."""

import hashlib
import json
from pathlib import Path, PurePosixPath
import posixpath
import re

from airs_bundle_archive import MAX_EXTRACTED

TEMPLATES = Path(__file__).parent / "fixtures/npm-command-shims/node.json"
EXTENSIONS = ("", ".cmd", ".ps1")


def verify_shims(root, commands, read_file):
    """Accept only whole, byte-exact node-only triplets for declared command targets."""
    reference = json.loads(read_file(TEMPLATES, 64 * 1024))
    receipts = []
    for command, target in commands.items():
        payload = read_file(root / target, MAX_EXTRACTED)
        if not payload.startswith(b"#!/usr/bin/env node\n"):
            raise ValueError("Unsupported bundled command shebang for Windows wrappers")
        relative = posixpath.relpath(target, str(PurePosixPath(command).parent))
        # Templates quote paths, but shell expansion still applies inside quotes.
        # Restrict interpolated bytes rather than attempting cross-shell escaping.
        if not re.fullmatch(r"[A-Za-z0-9@_./ -]+", relative):
            raise ValueError("Unsupported bundled command path for Windows wrappers")
        try:
            actual = {
                suffix: read_file(root / (command + suffix), 16 * 1024)
                for suffix in EXTENSIONS
            }
        except FileNotFoundError as error:
            raise ValueError(
                "Incomplete Windows bundled command wrapper set"
            ) from error
        for variant in reference["variants"]:
            expected = {
                suffix: template.replace(
                    "{{TARGET}}",
                    relative.replace("/", "\\") if suffix == ".cmd" else relative,
                ).encode()
                for suffix, template in variant["templates"].items()
            }
            if actual == expected:
                receipts.append(
                    {
                        "command": command,
                        "target": target,
                        "template_cmd_shim": variant["cmd_shim"],
                        "template_source_sha256": variant["source_sha256"],
                        "files": {
                            command + suffix: hashlib.sha256(data).hexdigest()
                            for suffix, data in actual.items()
                        },
                    }
                )
                break
        else:
            raise ValueError(
                "Windows bundled command wrappers differ from declared target"
            )
    return sorted(receipts, key=lambda item: item["command"])
