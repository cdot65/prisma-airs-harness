#!/usr/bin/env python3
"""Exercise secure login, inference and logout through the actual macOS CLI."""

import argparse
import json
import os
from pathlib import Path
import secrets
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    if sys.platform != "darwin":
        raise RuntimeError("This acceptance check requires macOS Keychain")
    os.environ["AIRS_HARNESS_BIN"] = str(args.binary.resolve(strict=True))
    from test_airs_harness import TerminalIntegration

    fixture = TerminalIntegration()
    fixture.setUp()
    key = secrets.token_urlsafe(32)
    try:
        fixture.configure()
        fixture.env.pop("AIRS_TEST_CREDENTIAL", None)
        # Exercise the actual OIDC chunked-store preflight before discovery.
        # The closed loopback endpoint cannot authenticate or issue any tokens.
        preflight = fixture.run_cli(
            "login",
            "--issuer-url",
            "https://127.0.0.1:9",
            "--oidc-client-id",
            "keychain-acceptance",
            "--audience",
            "keychain-acceptance",
        )
        if (
            preflight.returncode == 0
            or "issuer discovery unavailable" not in preflight.stderr
        ):
            raise RuntimeError(
                "OIDC native-store preflight failed: " + preflight.stderr[-4000:]
            )
        login = subprocess.run(
            [str(args.binary.resolve()), "login", "--with-api-key"],
            input=key + "\n",
            env=fixture.env,
            cwd=fixture.work,
            text=True,
            capture_output=True,
            timeout=60,
        )
        if login.returncode:
            raise RuntimeError(
                "Native CLI Keychain login failed: "
                + login.stderr.replace(key, "[REDACTED]")[-4000:]
            )
        doctor = fixture.run_cli("doctor", "--json")
        if doctor.returncode or not json.loads(doctor.stdout)["passed"]:
            raise RuntimeError(
                "A new CLI process could not use its Keychain credential"
            )
        result = fixture.execute()
        if result.returncode or len(fixture.requests) != 2:
            raise RuntimeError("Keychain-authenticated local tool loop failed")
        if (fixture.work / "result.txt").read_text() != "local tool worked\n":
            raise RuntimeError("Local shell tool did not create the expected file")
        outputs = [
            item
            for item in fixture.requests[1][2]["input"]
            if item.get("type") == "function_call_output"
        ]
        if (
            len(outputs) != 1
            or "local tool worked" not in outputs[0]["output"]
            or "Process exited with code 0" not in outputs[0]["output"]
        ):
            raise RuntimeError("Gateway did not receive successful local tool output")
        for _, headers, _ in fixture.requests:
            headers = {name.lower(): value for name, value in headers.items()}
            if headers.get("authorization") != "Bearer " + key:
                raise RuntimeError("Inference did not receive the Keychain credential")
        for path in fixture.home.rglob("*"):
            if path.is_file() and not path.is_symlink():
                if key.encode() in path.read_bytes():
                    raise RuntimeError(
                        "Credential appeared in plaintext application state"
                    )
        logout = fixture.run_cli("logout")
        if logout.returncode:
            raise RuntimeError("Native CLI logout failed")
        after_logout = fixture.execute()
        if not after_logout.returncode or len(fixture.requests) != 2:
            raise RuntimeError("Logout did not disable subsequent inference")
        args.receipt.write_text(
            json.dumps(
                {
                    "passed": True,
                    "platform": sys.platform,
                    "cli_secure_login": True,
                    "oidc_native_store_preflight": True,
                    "new_process_credential_access": True,
                    "authenticated_local_tool_loop": True,
                    "plaintext_state_absent": True,
                    "logout_blocks_inference": True,
                    "gateway": "deterministic loopback Responses server",
                    "version": subprocess.check_output(
                        [str(args.binary), "--version"], text=True
                    ).strip(),
                },
                indent=2,
            )
            + "\n"
        )
    finally:
        fixture.run_cli("logout")
        fixture.doCleanups()


if __name__ == "__main__":
    main()
