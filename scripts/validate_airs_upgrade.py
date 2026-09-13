#!/usr/bin/env python3
"""Linux two-binary upgrade smoke in disposable state and a private Secret Service.

Uses a synthetic workspace key and loopback Responses fixture, never user stores
or live APIs. This is bounded compatibility evidence, not full A18 acceptance.
"""

import argparse
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import secrets
import shutil
import subprocess
import sys
import tempfile
import tomllib
import uuid


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def assert_preserved_state(protected, config_path, helper, rollouts):
    """Permit only the owned executable field to change, never identity/history."""
    expected = tomllib.loads(protected[config_path].decode())
    expected["model_providers"]["airs"]["auth"]["command"] = str(helper)
    if tomllib.loads(config_path.read_text()) != expected:
        raise AssertionError(
            "Unexpected configuration change beyond the owned helper executable"
        )
    if not all(
        path.read_bytes() == original
        for path, original in protected.items()
        if path != config_path
    ):
        raise AssertionError("Credential, environment or session binding changed")
    if not all(
        path.read_bytes().startswith(original) for path, original in rollouts.items()
    ):
        raise AssertionError("Existing session history was rewritten")


def arguments():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("old", "candidate"):
        parser.add_argument(f"--{name}-binary", type=Path, required=True)
        parser.add_argument(f"--{name}-sha256", required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    parser.add_argument(
        "--upgrade-only",
        action="store_true",
        help="Test current native-store upgrades without the alpha.9 missing-helper rollback scenario",
    )
    parser.add_argument("--inside-dbus", action="store_true", help=argparse.SUPPRESS)
    return parser.parse_args()


def exercise(args, receipt):
    from test_airs_harness import TerminalIntegration
    from validate_workspace_login import assert_private, daemon_ready, events

    with ExitStack() as cleanup:
        fixture = TerminalIntegration()
        cleanup.callback(fixture.doCleanups)
        fixture.setUp()
        transcripts = []
        token = ("upgrade-fixture-" + secrets.token_hex(24)).encode()
        with tempfile.TemporaryDirectory(
            prefix=".airs-upgrade-", dir=Path.home()
        ) as tmp:
            root = Path(tmp).resolve()
            state = root / "state"
            work = root / "work"
            work.mkdir(mode=0o700)
            env = {
                key: value
                for key, value in os.environ.items()
                if not key.startswith(("AIRS_", "OPENAI_"))
                and key not in ("CODEX_HOME", "CODEX_SQLITE_HOME")
            }
            env.update(
                HOME=str(root), USERPROFILE=str(root), AIRS_HARNESS_HOME=str(state)
            )
            for name in ("XDG_DATA_HOME", "XDG_CONFIG_HOME", "XDG_RUNTIME_DIR"):
                path = root / name.lower()
                path.mkdir(mode=0o700)
                env[name] = str(path)
            binaries = {}
            for label, original in (
                ("old", args.old_binary),
                ("candidate", args.candidate_binary),
                ("relocated", args.candidate_binary),
            ):
                destination = root / label / "bin" / "airs-harness"
                destination.parent.mkdir(parents=True)
                try:
                    os.link(original, destination)
                except OSError:
                    shutil.copy2(original, destination)
                assert digest(destination) == getattr(
                    args, ("old" if label == "old" else "candidate") + "_sha256"
                )
                binaries[label] = destination
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
            )
            try:
                daemon.stdin.write(secrets.token_urlsafe(48).encode())
                daemon.stdin.close()
                daemon_ready(env, daemon)

                protected = {}
                rollouts = {}

                def run(label, *command, input_bytes=None, success=True):
                    result = subprocess.run(
                        [str(binaries[label]), *command],
                        cwd=work,
                        env=env,
                        input=input_bytes,
                        capture_output=True,
                        timeout=90,
                    )
                    transcripts.extend((result.stdout, result.stderr))
                    if (result.returncode == 0) != success:
                        # Outputs contain only fixture data, but never expose raw
                        # command output as a diagnostic if secret handling regresses.
                        receipt["failed_command"] = {
                            "binary": label,
                            "command": list(command),
                            "exit_code": result.returncode,
                        }
                        diagnostic = (result.stdout + result.stderr)[-8192:].decode(
                            errors="replace"
                        )
                        receipt["failed_command"]["redacted_diagnostic"] = (
                            diagnostic.replace(
                                token.decode(), "[synthetic-key-redacted]"
                            )
                            .replace(str(root), "[fixture-root]")
                            .replace(fixture.url, "[loopback-gateway]")
                        )
                        receipt["failed_command"]["request_count"] = len(
                            fixture.requests
                        )
                        if protected:
                            receipt["failed_command"]["protected_files_unchanged"] = (
                                all(
                                    path.read_bytes() == original
                                    for path, original in protected.items()
                                )
                            )
                            receipt["failed_command"]["history_prefix_preserved"] = all(
                                path.read_bytes().startswith(original)
                                for path, original in rollouts.items()
                            )
                        raise AssertionError("Unexpected upgrade command exit status")
                    return result

                receipt["phase"] = "old-environment"
                run(
                    "old",
                    "setup",
                    "--environment",
                    "upgrade",
                    "--gateway-url",
                    fixture.url,
                    "--allow-http-loopback",
                    "--context-window",
                    "32768",
                )
                registry = json.loads((state / "environments.json").read_text())
                identifier = registry["environments"]["upgrade"]["id"]
                uuid.UUID(identifier)
                home = state / "environments" / identifier
                run("old", "login", "--with-api-key", input_bytes=token + b"\n")
                binding_path = home / "credential-binding.json"
                binding = json.loads(binding_path.read_text())
                assert binding["source"]["kind"] == (
                    "keyring-v2" if args.upgrade_only else "keyring"
                )
                receipt["original_store_kind"] = binding["source"]["kind"]
                binding_id = binding["id"]
                receipt["checks"].append(
                    "old binary created named environment and native workspace binding"
                )

                def private_read(label):
                    result = subprocess.run(
                        [
                            str(binaries[label]),
                            "credential",
                            "--home",
                            str(home),
                            "--binding",
                            binding_id,
                        ],
                        env=env,
                        cwd=work,
                        capture_output=True,
                        timeout=10,
                    )
                    assert (
                        result.returncode == 0
                        and result.stdout == token + b"\n"
                        and not result.stderr
                    )
                    del result

                private_read("old")
                receipt["phase"] = "old-persistent-session"
                first_prompt = "Create result.txt using a local shell tool and remember UPGRADE_CONTEXT_CANARY."
                old_result = run(
                    "old",
                    "exec",
                    "--json",
                    "--skip-git-repo-check",
                    "-s",
                    "workspace-write",
                    first_prompt,
                )
                thread_id = next(
                    event["thread_id"]
                    for event in events(old_result.stdout)
                    if event.get("type") == "thread.started"
                )
                uuid.UUID(thread_id)
                assert (work / "result.txt").read_text() == "local tool worked\n"
                protected = {
                    path: path.read_bytes()
                    for path in [
                        state / "environments.json",
                        home / "config.toml",
                        home / "models.json",
                        binding_path,
                        home / "session-binding.json",
                    ]
                }
                rollouts = {
                    path: path.read_bytes() for path in home.glob("sessions/**/*.jsonl")
                }
                assert rollouts
                receipt["protected_file_sha256"] = {
                    path.relative_to(state).as_posix(): hashlib.sha256(data).hexdigest()
                    for path, data in protected.items()
                }
                receipt["history_prefix_sha256"] = {
                    path.relative_to(state).as_posix(): hashlib.sha256(data).hexdigest()
                    for path, data in rollouts.items()
                }
                receipt["checks"].append(
                    "old binary persistent session completed a real local tool loop"
                )

                config_path = home / "config.toml"
                previous_config_hash = digest(config_path)
                receipt["expected_configuration_migrations"] = []
                absent_old = binaries["old"].with_name("offline-original")
                binaries["old"].rename(absent_old)
                for label in ("candidate", "relocated"):
                    if label == "relocated":
                        binaries["candidate"].rename(
                            binaries["candidate"].with_name("offline-candidate")
                        )
                    receipt["phase"] = label + "-resume"
                    private_read(label)
                    run(label, "status")
                    before = len(fixture.requests)
                    run(
                        label,
                        "exec",
                        "resume",
                        "--json",
                        "--skip-git-repo-check",
                        thread_id,
                        "Confirm the remembered upgrade marker.",
                    )
                    assert len(fixture.requests) > before
                    assert any(
                        "UPGRADE_CONTEXT_CANARY" in json.dumps(body)
                        for _, _, body in fixture.requests[before:]
                    )
                    assert_preserved_state(
                        protected, config_path, binaries[label], rollouts
                    )
                    current_config_hash = digest(config_path)
                    receipt["expected_configuration_migrations"].append(
                        {
                            "stage": label,
                            "only_changed_field": "model_providers.airs.auth.command",
                            "expected_helper_relative_to_fixture_root": binaries[label]
                            .relative_to(root)
                            .as_posix(),
                            "previous_config_sha256": previous_config_hash,
                            "config_sha256": current_config_hash,
                            "all_other_configuration_values_identical": True,
                            "credential_and_session_bindings_byte_identical": True,
                        }
                    )
                    previous_config_hash = current_config_hash
                    receipt["checks"].append(
                        label
                        + " read same native credential and resumed original history with only the expected owned helper path migrated"
                    )
                absent_old.rename(binaries["old"])
                binaries["candidate"].with_name("offline-candidate").rename(
                    binaries["candidate"]
                )
                receipt["checks"].append(
                    "candidate and relocation work with previous fixture executable paths absent"
                )
                assert all("model" not in body for _, _, body in fixture.requests)
                receipt["checks"].append(
                    "gateway default omitted model on every captured request"
                )

                if args.upgrade_only:
                    receipt["phase"] = (
                        "current-client-rollback-with-newer-helpers-absent"
                    )
                    for label in ("candidate", "relocated"):
                        binaries[label].rename(
                            binaries[label].with_name("offline-newer")
                        )
                    count = len(fixture.requests)
                    run(
                        "old",
                        "exec",
                        "resume",
                        "--json",
                        "--skip-git-repo-check",
                        thread_id,
                        "Resume the original session after rolling back the executable.",
                    )
                    assert len(fixture.requests) > count
                    assert_preserved_state(
                        protected, config_path, binaries["old"], rollouts
                    )
                    receipt["old_client_without_newer_helper"] = {
                        "tested": True,
                        "exit_code": 0,
                        "new_inference_requests": len(fixture.requests) - count,
                        "identity_and_history_preserved": True,
                        "independent_downgrade_compatibility": True,
                    }
                    for label in ("candidate", "relocated"):
                        binaries[label].with_name("offline-newer").rename(
                            binaries[label]
                        )
                    receipt["checks"].append(
                        "current native-store baseline resumes with both newer helpers absent"
                    )
                else:
                    # Do not mistake old-client/new-helper coexistence for downgrade.
                    receipt["phase"] = "old-client-with-newer-helper-absent"
                    private_read("old")
                    for label in ("candidate", "relocated"):
                        binaries[label].rename(
                            binaries[label].with_name("offline-newer")
                        )
                    count = len(fixture.requests)
                    downgraded = run(
                        "old",
                        "exec",
                        "resume",
                        "--json",
                        "--skip-git-repo-check",
                        thread_id,
                        "The old client must not depend on a hidden newer helper.",
                        success=False,
                    )
                    assert b"No such file or directory" in downgraded.stderr
                    assert str(binaries["relocated"]).encode() in downgraded.stderr
                    assert len(fixture.requests) == count
                    assert_preserved_state(
                        protected, config_path, binaries["relocated"], rollouts
                    )
                    receipt["old_client_without_newer_helper"] = {
                        "exit_code": downgraded.returncode,
                        "new_inference_requests": 0,
                        "saved_helper_missing": True,
                        "identity_and_history_preserved": True,
                        "independent_downgrade_compatibility": False,
                    }
                    for label in ("candidate", "relocated"):
                        binaries[label].with_name("offline-newer").rename(
                            binaries[label]
                        )
                    receipt["checks"].append(
                        "old client fails without the newer helper and preserves identity/history"
                    )

                receipt["phase"] = "candidate-logout"
                run("candidate", "logout")
                count = len(fixture.requests)
                run(
                    "old",
                    "exec",
                    "resume",
                    "--json",
                    "--skip-git-repo-check",
                    thread_id,
                    "A signed-out legacy client must not send this.",
                    success=False,
                )
                assert len(fixture.requests) == count
                assert all(
                    path.read_bytes().startswith(original)
                    for path, original in rollouts.items()
                )
                receipt["checks"].append(
                    "candidate logout blocks old-binary resume without deleting history"
                )
                receipt["plaintext_files_inspected"] = assert_private(
                    [state, work], transcripts, token
                )
                assert all(
                    token.decode() not in json.dumps(body)
                    for _, _, body in fixture.requests
                )
                receipt["checks"].append(
                    "synthetic key absent from plaintext state, transcripts and inference bodies"
                )
                receipt["protected_file_sha256"] = {
                    path.relative_to(state).as_posix(): hashlib.sha256(data).hexdigest()
                    for path, data in protected.items()
                }
                receipt["history_prefix_sha256"] = {
                    path.relative_to(state).as_posix(): hashlib.sha256(data).hexdigest()
                    for path, data in rollouts.items()
                }
                receipt["inference_requests"] = len(fixture.requests)
                receipt["passed"] = True
                receipt["phase"] = "complete"
            finally:
                daemon.terminate()
                daemon.wait(timeout=10)


def main():
    args = arguments()
    if sys.platform != "linux":
        raise SystemExit("This fixture requires Linux and a disposable Secret Service")
    args.receipt = args.receipt.resolve()
    for name in ("old", "candidate"):
        path = getattr(args, name + "_binary").resolve(strict=True)
        setattr(args, name + "_binary", path)
        if digest(path) != getattr(args, name + "_sha256"):
            raise SystemExit(
                "Input executable differs from its required immutable hash"
            )
    if not args.inside_dbus:
        env = dict(
            os.environ,
            AIRS_UPGRADE_PARENT_BUS=os.environ.get("DBUS_SESSION_BUS_ADDRESS", ""),
        )
        command = [
            "dbus-run-session",
            "--",
            sys.executable,
            str(Path(__file__).resolve()),
            "--old-binary",
            str(args.old_binary),
            "--old-sha256",
            args.old_sha256,
            "--candidate-binary",
            str(args.candidate_binary),
            "--candidate-sha256",
            args.candidate_sha256,
            "--receipt",
            str(args.receipt),
            "--inside-dbus",
            *(["--upgrade-only"] if args.upgrade_only else []),
        ]
        return subprocess.call(command, env=env)
    if (
        "AIRS_UPGRADE_PARENT_BUS" not in os.environ
        or not os.environ.get("DBUS_SESSION_BUS_ADDRESS")
        or os.environ["DBUS_SESSION_BUS_ADDRESS"]
        == os.environ["AIRS_UPGRADE_PARENT_BUS"]
    ):
        raise SystemExit("Refusing to use the caller's session bus")
    receipt = {
        "schema_version": 1,
        "passed": False,
        "checks": [],
        "old_binary_sha256": args.old_sha256,
        "candidate_binary_sha256": args.candidate_sha256,
        "synthetic_credentials": True,
        "private_secret_service": True,
        "loopback_gateway": True,
        "published": False,
        "a18_complete": False,
        "scenario": "current-native-upgrade"
        if args.upgrade_only
        else "legacy-helper-rollback",
        "limitations": [
            "Linux fixture, not desktop onboarding",
            "workspace key only; no OIDC migration",
            "same candidate bytes relocated, not a subsequent distinct release",
            "no signing or published package upgrade",
            "Current native-store rollback is bounded to this exact binary pair; the legacy scenario checks missing-helper failure only",
        ],
    }
    try:
        exercise(args, receipt)
    except Exception as error:
        receipt["failure_type"] = type(error).__name__
        raise
    finally:
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(json.dumps(receipt, indent=2) + "\n")


if __name__ == "__main__":
    raise SystemExit(main())
