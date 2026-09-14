#!/usr/bin/env python3
"""Release Linux and Mac acceptance cleanup after both workflows finish.

Run this controller alongside both validators, each with --cleanup-barrier.
It reads and writes only nonsecret coordination files, never credentials.
"""

import argparse
import json
from pathlib import Path
import shlex
import subprocess
import time

from airs_gateway_cleanup import release_finished_peers


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--local", type=Path, required=True)
    parser.add_argument("--remote", type=Path, required=True)
    parser.add_argument("--ssh-host", required=True)
    parser.add_argument("--identity-file", type=Path)
    args = parser.parse_args()
    ssh = ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=8"]
    if args.identity_file:
        ssh += ["-o", "IdentityAgent=none", "-o", "IdentitiesOnly=yes", "-i", str(args.identity_file)]
    ssh += [args.ssh_host]

    def remote(operation, value=None):
        code = """import json,sys
from pathlib import Path
d=json.load(sys.stdin); p=Path(d['directory'])
if d['operation']=='read':
 f=p/'ready.json'; print(f.read_text() if f.exists() else 'null')
else:
 f=p/'release.tmp'; f.write_text(json.dumps(d['value'])+'\\n'); f.replace(p/'release.json')
"""
        result = subprocess.run(
            ssh + ["python3 -c " + shlex.quote(code)],
            input=json.dumps({"directory": str(args.remote), "operation": operation, "value": value}),
            text=True, capture_output=True, timeout=15, check=True,
        )
        return json.loads(result.stdout) if operation == "read" else None

    def read_ready(peer):
        if peer == "remote":
            return remote("read")
        path = args.local / "ready.json"
        return json.loads(path.read_text()) if path.exists() else None

    def write_release(peer, value):
        if peer == "remote":
            remote("release", value)
        else:
            pending = args.local / "release.tmp"
            pending.write_text(json.dumps(value) + "\n")
            pending.replace(args.local / "release.json")

    deadline = time.monotonic() + 10800
    while time.monotonic() < deadline:
        try:
            if release_finished_peers(["local", "remote"], read_ready, write_release):
                print("Both workflows finished; issuer cleanup released.", flush=True)
                return
        except (subprocess.SubprocessError, OSError, ValueError) as error:
            print("Cleanup controller retry: " + type(error).__name__, flush=True)
        time.sleep(5)
    raise SystemExit("Cleanup peers did not finish within three hours")


if __name__ == "__main__":
    main()
