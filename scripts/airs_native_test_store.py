"""Disposable native credential context shared by installed AIRS fixtures.

Linux workers must be launched with native_test_command, which creates a fresh
session bus. Darwin workers use their runner's GUI security session and exact,
randomly namespaced synthetic accounts. No owner records are enumerated.
"""

from contextlib import contextmanager
import json
import os
from pathlib import Path
import secrets
import re
import subprocess
import sys
import time

SERVICE = "Codex MCP Credentials"  # Existing on-disk identity, not product UI.


def native_test_command(script, arguments):
    command = [sys.executable, str(Path(script).resolve()), *arguments]
    return (
        ["dbus-run-session", "--", "env", "AIRS_NATIVE_TEST_PRIVATE_BUS=1", *command]
        if sys.platform.startswith("linux")
        else command
    )


def isolated_environment(root, inherited):
    env = {
        key: value
        for key, value in inherited.items()
        if not key.startswith(("AIRS_", "OPENAI_"))
        and key
        not in {
            "CODEX_HOME",
            "CODEX_SQLITE_HOME",
            "CODEX_CA_CERTIFICATE",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
            "GNOME_KEYRING_CONTROL",
        }
    }
    for variable, name in [
        ("HOME", "home"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_RUNTIME_DIR", "runtime"),
        ("GNOME_KEYRING_CONTROL", "keyring"),
    ]:
        if variable == "HOME" and sys.platform == "darwin":
            # Security.framework resolves the GUI login Keychain through HOME.
            # AIRS_HARNESS_HOME and XDG paths still isolate all fixture state.
            if not inherited.get("HOME") or not Path(inherited["HOME"]).is_absolute():
                raise ValueError("Darwin native fixture requires its GUI session HOME")
            continue
        directory = root / name
        directory.mkdir(mode=0o700, parents=True, exist_ok=True)
        env[variable] = str(directory)
    env.update(NO_PROXY="127.0.0.1,localhost", no_proxy="127.0.0.1,localhost")
    return env


def _has_secret_service(env):
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
        timeout=5,
        text=True,
        check=True,
    )
    if "boolean true" in result.stdout:
        return True
    if "boolean false" in result.stdout:
        return False
    raise RuntimeError("Could not determine private Secret Service ownership")


@contextmanager
def native_store(root, inherited_env):
    root = Path(root)
    env = isolated_environment(root, inherited_env)
    daemon = None
    try:
        if sys.platform.startswith("linux"):
            if (
                inherited_env.get("AIRS_NATIVE_TEST_PRIVATE_BUS") != "1"
                or not env.get("DBUS_SESSION_BUS_ADDRESS")
                or _has_secret_service(env)
            ):
                raise RuntimeError(
                    "Native fixture requires a fresh empty private D-Bus session"
                )
            subprocess.run(
                [
                    "dbus-update-activation-environment",
                    "XDG_DATA_HOME",
                    "XDG_CONFIG_HOME",
                    "XDG_RUNTIME_DIR",
                    "GNOME_KEYRING_CONTROL",
                ],
                env=env,
                capture_output=True,
                timeout=5,
                check=True,
            )
            daemon = subprocess.Popen(
                [
                    "gnome-keyring-daemon",
                    "--foreground",
                    "--unlock",
                    "--components=secrets",
                    "--control-directory",
                    env["GNOME_KEYRING_CONTROL"],
                ],
                env=env,
                stdin=subprocess.PIPE,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            daemon.stdin.write(secrets.token_urlsafe(32).encode())
            daemon.stdin.close()
            deadline = time.monotonic() + 10
            while not _has_secret_service(env):
                if daemon.poll() is not None or time.monotonic() >= deadline:
                    raise RuntimeError(
                        "Disposable native credential service failed to start"
                    )
                time.sleep(0.05)
        elif sys.platform != "darwin":
            raise RuntimeError(
                "Native fixture supports released Linux and Darwin targets"
            )
        yield env
    finally:
        if daemon is not None and daemon.poll() is None:
            daemon.terminate()
            daemon.wait(timeout=10)


def _accounts(identity, env):
    if not re.fullmatch(r"airs-test-[0-9a-f]{24}", identity.name):
        raise ValueError("Native fixture requires a randomly namespaced test identity")
    found = []
    if sys.platform.startswith("linux") and not _has_secret_service(env):
        raise RuntimeError("Private native credential service disappeared")
    for account in identity.accounts():
        if sys.platform == "darwin":
            command = [
                "security",
                "find-generic-password",
                "-s",
                SERVICE,
                "-a",
                account,
            ]
        else:
            command = ["secret-tool", "lookup", "service", SERVICE, "username", account]
        result = subprocess.run(command, env=env, capture_output=True, timeout=10)
        if result.returncode == 0:
            # Darwin's exact metadata query avoids asking another executable for
            # token data. A second AIRS process proves the credential is usable.
            if sys.platform != "darwin":
                record = json.loads(result.stdout)
                identity.verify_record(record)
            found.append(account)
            continue
        missing = (
            result.returncode == 44
            if sys.platform == "darwin"
            else result.returncode == 1 and not result.stderr.strip()
        )
        if not missing:
            raise RuntimeError(
                f"Synthetic native record lookup failed (status {result.returncode})"
            )
    return found


def record_exists(identity, env):
    """Check this run's exact native identity without reading Darwin token data."""
    return bool(_accounts(identity, env))


def delete_record(identity, env):
    # First validate any found record. Commands below name this run's two exact
    # serialization variants; never delete by shared service alone.
    for account in _accounts(identity, env):
        if sys.platform == "darwin":
            command = [
                "security",
                "delete-generic-password",
                "-s",
                SERVICE,
                "-a",
                account,
            ]
        else:
            command = ["secret-tool", "clear", "service", SERVICE, "username", account]
        result = subprocess.run(command, env=env, capture_output=True, timeout=10)
        allowed = {0, 44} if sys.platform == "darwin" else {0, 1}
        if result.returncode not in allowed:
            raise RuntimeError(
                f"Synthetic native record cleanup failed (status {result.returncode})"
            )
    if record_exists(identity, env):
        raise RuntimeError("Synthetic native credential cleanup did not complete")
