"""Installed refresh failures using only disposable, signed local identities."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import uuid

from airs_lifecycle_contract import EventRecorder, require
from airs_harness_pty import TerminalSession
from airs_lifecycle_session import LifecycleSession
from airs_lifecycle_tokens import RefreshFault
from airs_native_test_store import record_exists
from validate_airs_lifecycle import native_identity, tooling_identity, verify_identity


def identity_record_exists(account, environment):
    # Only the binding created in this run is queried; no credential enumeration.
    uuid.UUID(account)
    if os.uname().sysname == "Darwin":
        command = [
            "security",
            "find-generic-password",
            "-s",
            "io.cdot.airs-terminal",
            "-a",
            account,
        ]
        missing = 44
    else:
        command = [
            "secret-tool",
            "lookup",
            "service",
            "io.cdot.airs-terminal",
            "username",
            account,
        ]
        missing = 1
    result = subprocess.run(command, env=environment, capture_output=True, timeout=10)
    require(result.returncode in (0, missing), "Native identity lookup failed")
    if result.returncode == missing and missing == 1:
        require(not result.stderr.strip(), "Native identity store became unavailable")
    return result.returncode == 0


def exercise(binary, fault):
    exercise.phase = "identity"
    binary = Path(binary).resolve(strict=True)
    native_hash = native_identity(binary, binary)
    tooling = tooling_identity()
    recorder = EventRecorder()
    marker = {
        RefreshFault.REJECTED: "AIRS_CREDENTIAL_STATUS:sign_in_required",
        RefreshFault.RESPONSE_LOST: "AIRS_CREDENTIAL_STATUS:refresh_outcome_unknown",
    }[fault]
    cache = Path.home() / ".cache/airs-lifecycle-tests"
    cache.mkdir(mode=0o700, parents=True, exist_ok=True)
    require(not cache.is_symlink(), "Lifecycle fixture cache must not be linked")
    with tempfile.TemporaryDirectory(prefix="refresh-fault-", dir=cache) as temporary:
        with LifecycleSession(
            binary, Path(temporary), token_lifetime_seconds=60, event_sink=recorder
        ) as session:
            session.fixture.mcp.lifetime = 600
            exercise.phase = "initial-real-turn"
            session.start()
            session.turn("before_fault")
            session.command(
                "env",
                "create",
                "unaffected",
                "--gateway-url",
                session.fixture.identity.issuer + "/v1",
            )
            session.command("env", "use", "lifecycle")
            state = session.root / "state"
            registry = state / "environments.json"
            other_id = json.loads(registry.read_text())["environments"]["unaffected"][
                "id"
            ]
            other = state / "environments" / other_id
            protected_paths = [
                registry,
                session.home / "config.toml",
                session.home / "credential-binding.json",
                session.home / "auth-generation",
            ]
            protected_paths.extend(p for p in other.rglob("*") if p.is_file())
            protected = {p: p.read_bytes() for p in protected_paths}
            binding = json.loads((session.home / "credential-binding.json").read_text())
            require(
                identity_record_exists(binding["id"], session.env),
                "Initial native identity missing",
            )
            original_history = dict(session.history)
            require(original_history, "A real initial conversation is required")
            original_tools = sum(
                turn["tool_calls"] for turn in session.fixture.turns.values()
            )
            require(original_tools == 1, "Initial read-only MCP result not observed")
            initial_mcp_generation = session.fixture.mcp.generation

            helper = [
                str(binary),
                "credential",
                "--home",
                str(session.home),
                "--binding",
                binding["id"],
            ]
            exercise.phase = "positive-refresh-control"
            delay = max(0, session.fixture.inference.expires - time.time() - 29)
            require(delay <= 60, "Synthetic expiry exceeded bounded wait")
            session.pump_until(time.monotonic() + delay)
            control = subprocess.run(
                helper,
                env=session.env,
                cwd=session.work,
                capture_output=True,
                timeout=35,
            )
            require(
                control.returncode == 0 and not control.stderr,
                "Normal refresh control failed",
            )
            require(
                control.stdout.decode().strip() in session.fixture.inference.access,
                "Normal refresh did not return its actual issued credential",
            )
            del control
            require(
                session.fixture.inference.generation == 2,
                "Normal refresh did not rotate the grant",
            )
            consumed_before = set(session.fixture.inference.consumed)
            predecessor = set(session.fixture.inference.refresh)
            require(
                len(consumed_before) == 1 and len(predecessor) == 1,
                "Normal refresh control has unexpected grant inventory",
            )

            # Authentication helpers refresh inside their final 30-second window.
            # Observe real expiry instead of mutating clocks or stored credentials.
            session.fixture.inference.fail_next_refresh(fault)
            exercise.phase = "armed-refresh-fault"
            delay = max(0, session.fixture.inference.expires - time.time() - 29)
            require(delay <= 60, "Synthetic expiry exceeded bounded wait")
            session.pump_until(time.monotonic() + delay)
            outputs = []
            for _ in range(3):
                result = subprocess.run(
                    helper,
                    env=session.env,
                    cwd=session.work,
                    capture_output=True,
                    timeout=35,
                )
                outputs.append(result.stdout + result.stderr)
                require(
                    result.returncode != 0 and result.stdout == b"",
                    "Failed refresh exposed an access credential",
                )
                require(
                    marker.encode() in result.stderr.splitlines(),
                    "Restarted helper lost the precise recovery classification",
                )
                require(
                    b"/signin" in result.stderr,
                    "Failed refresh lacks explicit recovery action",
                )
                events = [
                    e
                    for e in recorder.events()
                    if e["kind"] == "refresh_requested" and e["resource"] == "inference"
                ]
                require(len(events) == 2, "A consumed refresh grant was replayed")
            require(
                session.fixture.inference.consumed - consumed_before == predecessor,
                "Wrong refresh predecessor consumed",
            )
            expected_generation = 3 if fault is RefreshFault.RESPONSE_LOST else 2
            require(
                session.fixture.inference.generation == expected_generation,
                "Fault did not reach the intended issuer phase",
            )
            require(
                events[1]["accepted"] is (fault is RefreshFault.RESPONSE_LOST),
                "Fault evidence disagrees with the issuer decision",
            )

            exercise.phase = "restarted-tui"
            # Stop only this fixture's idle terminal, then resume its real saved
            # conversation in a different process with the failed grant.
            session.terminal.process.terminate()
            session.terminal.wait_until(
                lambda: session.terminal.process.poll() is not None, timeout=15
            )
            resume_requests = len(session.fixture.identity.requests)
            with TerminalSession(
                binary,
                session.env,
                session.work,
                arguments=["--no-alt-screen", "resume", session.conversation_id],
            ) as restarted:
                exercise.phase = "restarted-tui-startup"
                recovery_text = (
                    b"Your sign-in needs to be restored"
                    if fault is RefreshFault.RESPONSE_LOST
                    else b"Your work session has ended"
                )
                restarted.wait_for(recovery_text, timeout=30)
                restarted.wait_for(
                    b"airs --environment lifecycle login --restore-session", timeout=5
                )
                restarted.wait_until(
                    lambda: restarted.process.poll() is not None, timeout=10
                )
                require(
                    restarted.process.returncode != 0,
                    "Resume incorrectly opened a failed identity",
                )
                outputs.append(bytes(restarted.transcript))
            require(
                len(session.fixture.identity.requests) == resume_requests,
                "Resume sent network requests before restoring its failed identity",
            )
            require(
                len(
                    [
                        e
                        for e in recorder.events()
                        if e["kind"] == "refresh_requested"
                        and e["resource"] == "inference"
                    ]
                )
                == 2,
                "Restarted TUI replayed the consumed grant",
            )

            # Explicit access verification must stop at the failed local helper;
            # the existing unauthenticated health check is allowed separately.
            request_count = len(session.fixture.identity.requests)
            exercise.phase = "doctor-and-preservation"
            doctor = subprocess.run(
                [
                    str(binary),
                    "--environment",
                    "lifecycle",
                    "doctor",
                    "--verify-access",
                    "--json",
                ],
                env=session.env,
                cwd=session.work,
                capture_output=True,
                timeout=40,
            )
            outputs.append(doctor.stdout + doctor.stderr)
            report = json.loads(doctor.stdout)
            require(
                isinstance(report.get("checks"), list),
                "Restarted doctor did not return structured checks",
            )
            access = [
                check
                for check in report["checks"]
                if check.get("name") == "gateway_access"
            ]
            require(
                doctor.returncode != 0
                and len(access) == 1
                and access[0].get("passed") is False,
                "Explicit doctor access check did not report the failed credential",
            )
            require(
                all(
                    request == ("GET", "/v1/health")
                    for request in session.fixture.identity.requests[request_count:]
                ),
                "Doctor retried authentication or inference despite the failed helper",
            )
            require(
                all(p.read_bytes() == content for p, content in protected.items()),
                "Refresh failure changed environment or binding",
            )
            session.check_retained_history()
            require(
                all(
                    p.read_bytes().startswith(content)
                    for p, content in original_history.items()
                ),
                "Refresh failure rewrote the conversation",
            )
            require(
                sum(turn["tool_calls"] for turn in session.fixture.turns.values())
                == original_tools,
                "Recovery replayed an MCP tool",
            )
            require(
                session.fixture.mcp.generation == initial_mcp_generation,
                "Inference failure changed the MCP grant",
            )
            require(
                record_exists(session.identity, session.env),
                "Inference failure removed the independent native MCP record",
            )
            require(
                not list(state.rglob(".credentials.json")),
                "Refresh failure wrote a plaintext credential fallback",
            )
            secrets = (
                set(session.fixture.inference.access)
                | set(session.fixture.inference.refresh)
                | session.fixture.inference.consumed
            )
            for output in outputs:
                require(
                    not any(value.encode() in output for value in secrets),
                    "Refresh diagnostic leaked a synthetic credential",
                )
            for p in [
                session.home / "config.toml",
                session.home / "credential-binding.json",
                *original_history,
            ]:
                require(
                    not any(value.encode() in p.read_bytes() for value in secrets),
                    "Credential appeared in persisted configuration or history",
                )

            exercise.phase = "native-cleanup"
            session.cleanup_credentials()
            require(
                not identity_record_exists(binding["id"], session.env),
                "Native inference identity remained after cleanup",
            )
            result = {
                "schema_version": 1,
                "case": fault.value,
                "passed": True,
                "native_sha256": native_hash,
                "tooling_sha256": hashlib.sha256(
                    json.dumps(tooling, sort_keys=True).encode()
                ).hexdigest(),
                "refresh_requests": len(events),
                "fault_refresh_requests": 1,
                "normal_refresh_control_passed": True,
                "fault_predecessor_consumed": True,
                "issuer_generation": expected_generation,
                "restarted_helper_checks": 3,
                "recovery_status": marker.split(":", 1)[1],
                "resume_startup_blocked_with_recovery_message": True,
                "configuration_binding_other_environment_preserved": True,
                "real_conversation_preserved": True,
                "actual_mcp_calls": original_tools,
                "independent_mcp_grant_preserved": True,
                "doctor_no_authentication_or_inference_network": True,
                "plaintext_fallback_absent": True,
                "native_cleanup_completed": True,
                "production_acceptance": False,
            }
        verify_identity(binary, binary, native_hash, tooling)
        return result
