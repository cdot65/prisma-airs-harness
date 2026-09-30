"""Actual installed stable upgrade/rollback with disposable native identities."""

import hashlib
import json
import os
import shutil
from pathlib import Path
import tempfile

from airs_harness_pty import TerminalSession
from airs_lifecycle_contract import EventRecorder, require
from airs_lifecycle_faults import identity_record_exists
from airs_lifecycle_session import LifecycleSession
from airs_native_test_store import record_exists
from validate_airs_lifecycle import native_identity, tooling_identity


def link_package(link):
    """Name of the npm package directory an installed command link resolves into."""
    target = Path(os.path.normpath(Path(link).parent / os.readlink(link)))
    parts = target.parts
    require("node_modules" in parts, "Command link does not point into npm packages")
    return parts[parts.index("node_modules") + 1]


def stop_terminal(session):
    terminal = session.terminal
    terminal.process.terminate()
    terminal.wait_until(lambda: terminal.process.poll() is not None, timeout=15)


def observe_roundtrip(
    prefix,
    previous,
    candidate,
    install,
    inspect_native,
    expected_previous_hash,
    expected_previous_source,
    link_packages=None,
):
    """Install callbacks execute exact versions into one retained npm prefix.

    ``link_packages`` maps each phase to the launcher package name its commands
    must resolve into when the previous release shipped under another name.
    Without it, every command link must stay byte-identical across the roundtrip.
    ``install`` returns whether it removed the other launcher package by name.
    """
    tooling = tooling_identity()
    recorder = EventRecorder()
    observe_roundtrip.phase = "install-previous"
    uninstalled = [bool(install("previous"))]
    previous_native, previous_info = inspect_native(prefix)
    previous_hash = native_identity(previous_native, previous_native)
    require(
        previous_hash == previous_info["binary_sha256"] == expected_previous_hash
        and previous_info["source_commit"] == expected_previous_source,
        "Previous native provenance mismatch",
    )
    links = {
        name: os.readlink(prefix / "bin" / name) for name in ("airs", "airs-harness")
    }

    def check_links(phase):
        if link_packages is None:
            require(
                {name: os.readlink(prefix / "bin" / name) for name in links} == links,
                "Upgrade changed npm command links",
            )
        else:
            require(
                all(
                    link_package(prefix / "bin" / name) == link_packages[phase]
                    for name in links
                ),
                "Command links do not resolve into the expected launcher package",
            )

    check_links("previous")
    cache = Path.home() / ".cache/airs-lifecycle-tests"
    cache.mkdir(mode=0o700, parents=True, exist_ok=True)
    require(not cache.is_symlink(), "Lifecycle fixture cache must not be a symlink")
    with tempfile.TemporaryDirectory(prefix="upgrade-", dir=cache) as directory:
        with LifecycleSession(
            previous_native,
            Path(directory),
            token_lifetime_seconds=900,
            event_sink=recorder,
        ) as session:
            # npm may fail while replacing the prefix. Keep an immutable owned
            # executable available to delete only this fixture's native identity.
            cleanup_binary = session.root / "cleanup-airs"
            shutil.copy2(previous_native, cleanup_binary)
            session.stack.callback(setattr, session, "binary", cleanup_binary)
            observe_roundtrip.phase = "previous-live-turn"
            session.start()
            session.turn("before_upgrade")
            session.check_retained_history()
            binding = json.loads((session.home / "credential-binding.json").read_text())
            account = binding["id"]
            protected = {
                path: path.read_bytes()
                for path in (
                    session.root / "state/environments.json",
                    session.home / "config.toml",
                    session.home / "credential-binding.json",
                    session.home / "auth-generation",
                )
            }
            initial_identity = session.snapshot()
            rounds = []
            for phase, label in (
                ("candidate", "after_upgrade"),
                ("previous", "after_rollback"),
            ):
                observe_roundtrip.phase = phase + "-install"
                stop_terminal(session)
                uninstalled.append(bool(install(phase)))
                binary, info = inspect_native(prefix)
                digest = native_identity(binary, binary)
                require(
                    digest == info["binary_sha256"],
                    "Reinstalled native provenance mismatch",
                )
                check_links(phase)
                require(
                    all(
                        path.read_bytes() == content
                        for path, content in protected.items()
                    ),
                    "Installation changed configured identity",
                )
                require(
                    all(
                        path.read_bytes().startswith(content)
                        for path, content in session.history.items()
                    ),
                    "Installation changed real conversation",
                )
                require(
                    record_exists(session.identity, session.env)
                    and identity_record_exists(account, session.env),
                    "Installation removed native credential",
                )
                if phase == "previous":
                    require(
                        digest == previous_hash,
                        "Rollback did not restore exact previous native bytes",
                    )
                else:
                    require(
                        digest != previous_hash,
                        "Candidate must differ from previous native bytes",
                    )
                session.binary = binary
                observe_roundtrip.phase = phase + "-resumed-turn"
                session.terminal = session.stack.enter_context(
                    TerminalSession(
                        binary,
                        session.env,
                        session.work,
                        arguments=[
                            "--no-alt-screen",
                            "resume",
                            session.conversation_id,
                        ],
                    )
                )
                session.terminal.wait_for(b"permissions:", timeout=40)
                session.turn(label)
                current = session.snapshot()
                for key in (
                    "conversation_id",
                    "binding_sha256",
                    "auth_epoch_sha256",
                    "resource_generations",
                ):
                    require(
                        current[key] == initial_identity[key],
                        "Resumed turn changed identity or conversation",
                    )
                require(
                    all(
                        path.read_bytes() == content
                        for path, content in protected.items()
                    ),
                    "Resumed turn changed configured identity",
                )
                require(
                    not (session.home / ".credentials.json").exists(),
                    "Roundtrip used plaintext fallback",
                )
                rounds.append(
                    {
                        "phase": phase,
                        "binary_sha256": digest,
                        "source_commit": info["source_commit"],
                        "real_turn_completed": True,
                    }
                )
            require(
                sum(turn["tool_calls"] for turn in session.fixture.turns.values()) == 3,
                "Roundtrip duplicated or lost a tool call",
            )
            observe_roundtrip.phase = "native-cleanup"
            stop_terminal(session)
            session.cleanup_credentials()
            require(
                not identity_record_exists(account, session.env),
                "Inference credential cleanup failed",
            )
            require(
                not record_exists(session.identity, session.env),
                "MCP credential cleanup failed",
            )
            receipt = {
                "passed": True,
                "previous_version": previous,
                "candidate_version": candidate,
                "restored_version": previous,
                "previous_binary_sha256": previous_hash,
                "candidate_binary_sha256": rounds[0]["binary_sha256"],
                "restored_binary_sha256": rounds[1]["binary_sha256"],
                "candidate_source_commit": rounds[0]["source_commit"],
                "previous_source_commit": previous_info["source_commit"],
                "command_links_preserved": True,
                "command_links_retargeted": link_packages is not None,
                "configuration_preserved": True,
                "real_conversation_preserved": True,
                "inference_credential_reused": True,
                "mcp_credential_reused": True,
                "native_cleanup_completed": True,
                "real_mcp_turns": 3,
                "uninstall_used": any(uninstalled),
                "force_used": False,
                "production_acceptance": False,
            }
    require(
        tooling_identity() == tooling, "Roundtrip tooling changed during observation"
    )
    receipt["tooling_sha256"] = hashlib.sha256(
        json.dumps(tooling, sort_keys=True).encode()
    ).hexdigest()
    return receipt
