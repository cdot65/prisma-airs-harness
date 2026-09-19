#!/usr/bin/env python3
"""Prepare pinned GNU voice build inputs for the complete upstream workspace.

These are test/build dependencies only; AIRS realtime remains disabled.
"""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import urllib.request

root = Path(__file__).resolve().parents[1]
manifest = root / "third_party/voice/sources.json"
identity = hashlib.sha256(manifest.read_bytes()).hexdigest()
cache = Path("/airs-native")
archives = cache / "archives"
archives.mkdir(parents=True, exist_ok=True)
output = cache / identity
if not (output / "built.json").is_file():
    if output.exists():
        output.rename(cache / f"{identity}-failed-{time.time_ns()}")
    for item in json.loads(manifest.read_text())["sources"]:
        archive = archives / item["archive"]
        if (
            not archive.exists()
            or hashlib.sha256(archive.read_bytes()).hexdigest() != item["sha256"]
        ):
            with urllib.request.urlopen(item["url"], timeout=120) as response:
                data = response.read(256 * 1024 * 1024)
            if hashlib.sha256(data).hexdigest() != item["sha256"]:
                raise ValueError(f"Checksum mismatch: {item['name']}")
            archive.write_bytes(data)
    subprocess.run(
        [
            sys.executable,
            str(root / "third_party/voice/build_native.py"),
            "--archives",
            str(archives),
            "--output",
            str(output),
            "--target",
            "x86_64-unknown-linux-gnu",
            "--cc",
            "/usr/bin/clang",
            "--cxx",
            "/usr/bin/clang++",
            "--cmake",
            "/usr/bin/cmake",
            "--make",
            "/usr/bin/make",
            "--pkg-config",
            "/usr/bin/pkg-config",
            "--shell",
            "/bin/bash",
            "--jobs",
            "2",
        ],
        check=True,
    )
prefix = output / "prefix"
# Keep system ALSA discoverable alongside the pinned GStreamer/GLib prefix.
with Path(os.environ["RUNNER_TEMP"], "airs-native.env").open("w") as stream:
    import shlex

    stream.write(
        f"export PKG_CONFIG_PATH={shlex.quote(str(prefix / 'lib/pkgconfig'))}\n"
    )
    stream.write(f"export LD_LIBRARY_PATH={shlex.quote(str(prefix / 'lib'))}\n")

    stream.write(f"export CODEX_TEST_VOICE_RUNTIME={shlex.quote(str(prefix))}\n")
