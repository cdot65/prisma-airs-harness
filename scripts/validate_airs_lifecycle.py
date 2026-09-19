#!/usr/bin/env python3
"""Observe real installed OIDC/MCP renewal in one continuous synthetic session."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time

from airs_lifecycle_contract import EventRecorder, require, validate_observation
from airs_native_test_store import native_test_command
from airs_release_receipts import tooling_files


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def tooling_identity():
    root = Path(__file__).resolve().parents[1]
    return {
        path.relative_to(root).as_posix(): digest(path) for path in tooling_files(root)
    }


def native_identity(binary, native):
    require(
        binary == native, "Lifecycle observations require the direct native executable"
    )
    with binary.open("rb") as stream:
        require(
            stream.read(4) in {b"\x7fELF", b"\xcf\xfa\xed\xfe"},
            "Lifecycle executable is not a released native target",
        )
    return digest(binary)


def verify_identity(binary, native, native_hash, tooling):
    require(
        native_identity(binary, native) == native_hash,
        "Observed native executable changed during run",
    )
    require(
        tooling_identity() == tooling, "Lifecycle tooling changed during observation"
    )


def atomic_json(path, value):
    temporary = path.with_suffix(path.suffix + ".tmp")
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "w") as stream:
        json.dump(value, stream, indent=2, allow_nan=False)
        stream.write("\n")
    os.replace(temporary, path)


def checkpoint(output, profile, phase, elapsed, recorder):
    atomic_json(
        output / "PROGRESS.json",
        {
            "profile": profile,
            "phase": phase,
            "elapsed_seconds": elapsed,
            "event_count": recorder.count,
            "production_acceptance": False,
        },
    )


def drain(session, deadline, output, profile, started, recorder):
    while time.monotonic() < deadline:
        session.pump_until(min(deadline, time.monotonic() + 30))
        checkpoint(output, profile, "observing", time.monotonic() - started, recorder)


def observe(args):
    from airs_lifecycle_session import LifecycleSession

    recorder = EventRecorder()
    output = args.output.resolve(strict=True)
    binary = args.binary.resolve(strict=True)
    native = (args.native_binary or args.binary).resolve(strict=True)
    native_hash = native_identity(binary, native)
    tooling = tooling_identity()
    atomic_json(
        output / "RUN-IDENTITY.json",
        {"native_sha256": native_hash, "tooling_files": tooling},
    )
    lifetime = 90 if args.profile == "quick" else 300
    cache = Path.home() / ".cache" / "airs-lifecycle-tests"
    cache.mkdir(mode=0o700, parents=True, exist_ok=True)
    require(not cache.is_symlink(), "Lifecycle fixture cache must not be a symlink")
    with tempfile.TemporaryDirectory(prefix="session-", dir=cache) as directory:
        try:
            with LifecycleSession(
                binary,
                Path(directory),
                token_lifetime_seconds=lifetime,
                event_sink=recorder,
            ) as session:
                session.start()
                session.turn("initial")
                # Let startup discovery finish before pinning the quiet/timed boundary.
                session.pump_until(time.monotonic() + 2)
                baseline = session.snapshot()
                started = time.monotonic()
                recorder("phase_started", profile=args.profile, phase="observe")
                quiet_end = quiet_snapshot = None
                if args.profile == "quick":
                    for cycle in range(1, 3):
                        previous = session.snapshot()
                        until_expired = (
                            max(previous["resource_expiries"].values())
                            - time.time()
                            + 1
                        )
                        require(
                            until_expired <= lifetime + 5,
                            "Fixture expiry exceeded bounded quick profile",
                        )
                        drain(
                            session,
                            time.monotonic() + max(until_expired, 0),
                            output,
                            args.profile,
                            started,
                            recorder,
                        )
                        session.turn(f"quick_{cycle}")
                        checkpoint(
                            output,
                            args.profile,
                            f"cycle_{cycle}",
                            time.monotonic() - started,
                            recorder,
                        )
                elif args.profile == "active":
                    deadline = started + 3600
                    turn = 0
                    while time.monotonic() < deadline:
                        drain(
                            session,
                            min(deadline, time.monotonic() + 240),
                            output,
                            args.profile,
                            started,
                            recorder,
                        )
                        turn += 1
                        session.turn(f"active_{turn}")
                        checkpoint(
                            output,
                            args.profile,
                            f"turn_{turn}",
                            time.monotonic() - started,
                            recorder,
                        )
                else:
                    drain(
                        session, started + 2100, output, args.profile, started, recorder
                    )
                    quiet_end = time.monotonic()
                    quiet_snapshot = session.snapshot()
                    session.turn("idle_return")
                recorder("phase_finished", profile=args.profile, phase="observe")
                final = session.snapshot()
                finished = time.monotonic()
                observation = {
                    "schema_version": 1,
                    "profile": args.profile,
                    "started_monotonic": started,
                    "finished_monotonic": finished,
                    "quiet_finished_monotonic": quiet_end,
                    "quiet_snapshot": quiet_snapshot,
                    "baseline": baseline,
                    "final": final,
                    "events": recorder.events(),
                }
                atomic_json(output / "OBSERVATION.json", observation)
                summary = validate_observation(observation)
            # A successful receipt is written only after credential/process cleanup.
            verify_identity(binary, native, native_hash, tooling)
            receipt = {
                **summary,
                "schema_version": 1,
                "native_sha256": native_hash,
                "executable_sha256": native_hash,
                "tooling_files": tooling,
                "token_lifetime_seconds": lifetime,
                "observation_sha256": digest(output / "OBSERVATION.json"),
                "native_cleanup_completed": True,
                "production_sso": False,
                "production_servicenow": False,
                "owner_ubuntu_acceptance": False,
            }
            atomic_json(output / "LIFECYCLE-ACCEPTANCE.json", receipt)
            print(
                json.dumps(
                    {
                        "passed": True,
                        "profile": args.profile,
                        "elapsed_seconds": summary["elapsed_seconds"],
                    }
                )
            )
        except BaseException as error:
            atomic_json(
                output / "FAILED.json",
                {
                    "passed": False,
                    "profile": args.profile,
                    "error_type": type(error).__name__,
                    "event_count": recorder.count,
                    "native_sha256": native_hash,
                    "production_acceptance": False,
                },
            )
            raise


def terminate_worker(process):
    if process.poll() is None:
        os.killpg(process.pid, signal.SIGTERM)
        try:
            process.wait(timeout=30)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=10)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--native-binary", type=Path)
    parser.add_argument("--profile", choices=("quick", "active", "idle"), required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--worker", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.worker:
        signal.signal(signal.SIGTERM, lambda _signum, _frame: sys.exit(143))
        observe(args)
        return 0
    require(
        not args.output.exists() and not args.output.is_symlink(),
        "Use a fresh lifecycle evidence directory",
    )
    args.output.mkdir(mode=0o700, parents=True)
    arguments = [
        "--worker",
        "--binary",
        str(args.binary.resolve(strict=True)),
        "--profile",
        args.profile,
        "--output",
        str(args.output.resolve()),
    ]
    if args.native_binary:
        arguments.extend(
            ["--native-binary", str(args.native_binary.resolve(strict=True))]
        )
    timeout = {"quick": 600, "active": 4200, "idle": 2700}[args.profile]
    log = args.output / "worker.private.log"
    descriptor = os.open(log, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "w") as stream:
        process = subprocess.Popen(
            native_test_command(__file__, arguments),
            stdout=stream,
            stderr=stream,
            start_new_session=True,
        )
        deadline = time.monotonic() + timeout
        try:
            while process.poll() is None:
                if time.monotonic() > deadline:
                    raise TimeoutError("Bounded lifecycle worker timed out")
                try:
                    process.wait(timeout=30)
                except subprocess.TimeoutExpired:
                    pass
        finally:
            terminate_worker(process)
    require(
        process.returncode == 0,
        "Lifecycle observation failed; inspect private worker diagnostics",
    )
    receipt = json.loads((args.output / "LIFECYCLE-ACCEPTANCE.json").read_text())
    require(receipt.get("passed") is True, "Lifecycle receipt did not pass")
    print(
        json.dumps(
            {
                "passed": True,
                "profile": args.profile,
                "receipt": str(args.output / "LIFECYCLE-ACCEPTANCE.json"),
            }
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, RuntimeError, TimeoutError) as error:
        print(
            f"Lifecycle check failed ({type(error).__name__}); private diagnostics retained",
            file=sys.stderr,
        )
        raise SystemExit(1)
