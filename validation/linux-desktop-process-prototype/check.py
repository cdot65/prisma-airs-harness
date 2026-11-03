#!/usr/bin/env python3
"""Focused, offline kernel boundary tests; no engine bootstrap or product calls."""

from pathlib import Path
import json
import subprocess
import sys
import tempfile

binary = Path(sys.argv[1]).resolve()
environment = {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"}
outcomes = []
for mode, expected, signal in (
    ("ordinary-control", 0, "Ordinary control fork succeeded"),
    ("assert-desktop-without-guard", 101, "fork escaped Desktop restrictions"),
    ("guard", 0, "Desktop process boundary passed"),
    ("guard-exec", 0, "Desktop process boundary passed"),
):
    result = subprocess.run(
        [str(binary), mode], env=environment, capture_output=True, text=True, timeout=10
    )
    assert result.returncode == expected, (
        mode,
        result.returncode,
        result.stdout,
        result.stderr,
    )
    assert signal in result.stdout + result.stderr, mode
    outcomes.append(
        {
            "mode": mode,
            "exit_code": result.returncode,
            "expected_exit_code": expected,
            "signal_matched": True,
        }
    )
with tempfile.TemporaryDirectory(prefix="desktop-policy-fixture-") as temporary:
    root = Path(temporary)
    account = root / "account"
    resources = root / "installed/harness"
    project = account / "projects/owned"
    secret = root / "secret-service-state"
    redirected = root / "redirected-config"
    for path in (
        account,
        resources,
        project,
        secret,
        redirected,
        account / ".local/share",
    ):
        path.mkdir(parents=True, exist_ok=True)
    (account / ".local/share/keyrings").symlink_to(secret, target_is_directory=True)
    (account / ".config").symlink_to(redirected, target_is_directory=True)
    for name, path, expected in (
        ("owned-project", project, 0),
        ("account-parent", account, 3),
        ("keyring-symlink-target", secret, 3),
        ("missing-leaf-in-redirected-config", redirected, 3),
        ("runtime-resources", resources, 3),
        ("runtime-parent", resources.parent, 3),
    ):
        result = subprocess.run(
            [
                str(binary),
                "project-policy",
                str(path.resolve()),
                str(account),
                str(resources),
            ],
            env=environment,
            capture_output=True,
            text=True,
            timeout=10,
        )
        assert result.returncode == expected, (name, result.returncode)
        outcomes.append(
            {
                "mode": name,
                "exit_code": result.returncode,
                "expected_exit_code": expected,
            }
        )
print(
    json.dumps(
        {
            "status": "pass",
            "outcomes": outcomes,
            "product_calls": False,
            "full_engine_or_filesystem_network_qualification": False,
        },
        indent=2,
    )
)
