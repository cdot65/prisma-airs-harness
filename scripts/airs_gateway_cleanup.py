"""Coordinate issuer revocation across tests sharing a browser SSO session.

OS credential-store isolation does not isolate the Keycloak client session.
The controller releases every peer only after all workflows have finished.
No access token or refresh token belongs in these coordination files.
"""

import json
from pathlib import Path
import secrets
import time


def release_finished_peers(peers, read_ready, write_release):
    """Do not revoke any shared grant until all named workflows have finished."""
    ready = [(peer, read_ready(peer)) for peer in peers]
    if not ready or any(value is None for _, value in ready):
        return False
    for _, value in ready:
        if (
            not isinstance(value, dict)
            or not isinstance(value.get("workflow_passed"), bool)
            or not isinstance(value.get("nonce"), str)
            or len(value["nonce"]) != 48
            or any(c not in "0123456789abcdef" for c in value["nonce"])
        ):
            raise ValueError("Invalid peer cleanup readiness")
    for peer, value in ready:
        write_release(peer, {"nonce": value["nonce"]})
    return True


def wait_for_cleanup_release(
    directory: Path,
    *,
    workflow_passed: bool,
    timeout_seconds: int = 10800,
    monotonic=time.monotonic,
    sleep=time.sleep,
):
    directory.mkdir(mode=0o700, parents=True, exist_ok=False)
    nonce = secrets.token_hex(24)
    ready = {"nonce": nonce, "workflow_passed": workflow_passed}
    pending = directory / "ready.tmp"
    pending.write_text(json.dumps(ready) + "\n")
    pending.replace(directory / "ready.json")
    deadline = monotonic() + timeout_seconds
    while monotonic() < deadline:
        try:
            release = json.loads((directory / "release.json").read_text())
            if release == {"nonce": nonce}:
                return
        except (FileNotFoundError, ValueError):
            pass
        sleep(min(5, max(0, deadline - monotonic())))
    raise TimeoutError("Peer cleanup was not released; acceptance failed, cleaning up")
