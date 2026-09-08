#!/usr/bin/env python3
"""Verify native credential persistence across processes with disposable values.

Linux creates an isolated encrypted Secret Service collection. macOS and Windows
use the runner/user's unlocked native store with a fresh random account name.
No gateway credentials or identity-provider credentials are needed.
"""

import argparse
import json
import os
from pathlib import Path
import secrets
import subprocess
import sys
import tempfile
import time
import uuid


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--inside-dbus", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    binary = args.binary.resolve()
    if sys.platform.startswith("linux") and not args.inside_dbus:
        return subprocess.call(
            [
                "dbus-run-session",
                "--",
                sys.executable,
                str(Path(__file__).resolve()),
                "--binary",
                str(binary),
                "--output",
                str(args.output.resolve()),
                "--inside-dbus",
            ]
        )
    daemon = None
    with tempfile.TemporaryDirectory(prefix="airs-native-credentials-") as directory:
        env = dict(os.environ)
        if sys.platform.startswith("linux"):
            for variable, name in [
                ("XDG_DATA_HOME", "data"),
                ("XDG_CONFIG_HOME", "config"),
                ("XDG_RUNTIME_DIR", "runtime"),
            ]:
                path = Path(directory) / name
                path.mkdir(mode=0o700)
                env[variable] = str(path)
            daemon = subprocess.Popen(
                [
                    "gnome-keyring-daemon",
                    "--foreground",
                    "--unlock",
                    "--components=secrets",
                ],
                env=env,
                stdin=subprocess.PIPE,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                text=True,
            )
            daemon.stdin.write(secrets.token_urlsafe(48))
            daemon.stdin.close()
        try:
            if daemon:
                for _ in range(50):
                    result = subprocess.run(
                        [
                            "dbus-send",
                            "--session",
                            "--print-reply",
                            "--dest=org.freedesktop.DBus",
                            "/org/freedesktop/DBus",
                            "org.freedesktop.DBus.NameHasOwner",
                            "string:org.freedesktop.secrets",
                        ],
                        env=env,
                        capture_output=True,
                        text=True,
                        timeout=5,
                    )
                    if "boolean true" in result.stdout:
                        break
                    time.sleep(0.2)
                else:
                    raise RuntimeError("Isolated Secret Service did not start")
            account = str(uuid.uuid4())
            rows = []
            for phase in [
                "write",
                "read-and-pend",
                "read-and-delete",
                "workspace-legacy-write",
                "workspace-read-and-write-v2",
                "workspace-read-both-delete-legacy",
                "workspace-read-and-delete-v2",
            ]:
                result = subprocess.run(
                    [str(binary), phase, account],
                    env=env,
                    capture_output=True,
                    text=True,
                    timeout=60,
                )
                if result.returncode:
                    raise RuntimeError(
                        f"Native store failed during {phase}; exit {result.returncode}"
                    )
                rows.append(json.loads(result.stdout))
            receipt = {
                "passed": True,
                "platform": sys.platform,
                "separate_processes": len(rows),
                "workspace_v2_max_token_bytes": 16_384,
                "cases": rows,
            }
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(receipt, indent=2) + "\n")
            print(json.dumps(receipt))
        finally:
            if daemon:
                daemon.terminate()
                daemon.wait(timeout=10)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
