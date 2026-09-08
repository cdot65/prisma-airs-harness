#!/usr/bin/env python3
"""Live Linux workspace-key acceptance using an isolated native credential store.

Only a redacted receipt is retained. The supplied key is entered through the real
harness prompt, never CLI arguments or an environment variable. This operator
fixture provisions its own disposable Secret Service; it is not onboarding UX.
"""

import argparse
import hashlib
import json
import os
import secrets
import select
import stat
import subprocess
import sys
import tempfile
import time
import uuid
from pathlib import Path
from urllib.parse import urlsplit

import tomllib
from airs_harness_pty import TerminalSession

MAX_LOGIN_OUTPUT_BYTES = 256 * 1024
GATEWAY_VERIFIED = b"Gateway access verified by one inference response. MCP permissions were not tested."
GATEWAY_DISCLOSURE = b"Checking gateway access with one minimal inference request (up to 16 output tokens)."


class LoginTranscript(bytearray):
    def extend(self, value):
        if len(self) + len(value) > MAX_LOGIN_OUTPUT_BYTES:
            raise AssertionError("Guided login output exceeded its capture limit")
        super().extend(value)


def verify_login_output(output, token, require_gateway_verification):
    assert_private([], [output], token)
    disclosed = GATEWAY_DISCLOSURE in output
    verified = GATEWAY_VERIFIED in output
    if require_gateway_verification and not (disclosed and verified):
        reasons = {
            b"selected gateway or model configuration is invalid": "configuration",
            b"saved credential could not be read or refreshed": "credential",
            b"Authentication changed or this environment was signed out": "signed-out",
            b"gateway connection failed": "connection",
            b"access check reached its time limit": "timeout",
            b"gateway denied access": "denied",
            b"gateway requested a redirect": "redirect",
            b"gateway rejected the probe": "rejected",
            b"gateway did not return a successful Responses API result": "invalid-response",
            b"response exceeded the 64 KiB": "oversized-response",
        }
        category = next(
            (label for text, label in reasons.items() if text in output), "unclassified"
        )
        raise AssertionError(
            f"Gateway verification failed: disclosed={disclosed}, verified={verified}, category={category}"
        )
    return disclosed, verified


def credential(path):
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(descriptor, "rb") as source:
        metadata = os.fstat(source.fileno())
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_uid != os.getuid():
            raise ValueError("Credential file must be a user-owned regular file")
        if metadata.st_mode & 0o077 or metadata.st_size > 16384:
            raise ValueError("Credential file must be owner-only and at most 16 KiB")
        raw = source.read(16385).strip()
    if not raw or len(raw) > 16384 or any(value < 33 or value > 126 for value in raw):
        raise ValueError("Credential file must contain one bounded printable token")
    return raw


def daemon_ready(env, daemon):
    for _ in range(50):
        if daemon.poll() is not None:
            raise RuntimeError("Isolated credential service exited")
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
            check=False,
            capture_output=True,
            timeout=5,
        )
        if b"boolean true" in result.stdout:
            return
        time.sleep(0.2)
    raise RuntimeError("Isolated credential service did not start")


def events(output):
    result = []
    for line in output.splitlines():
        try:
            value = json.loads(line)
            if isinstance(value, dict):
                result.append(value)
        except ValueError:
            pass
    return result


def assert_private(roots, transcripts, token):
    markers = [token, json.dumps(token.decode())[1:-1].encode()]
    for output in transcripts:
        if any(marker in output for marker in markers):
            raise AssertionError("Credential appeared in process output")
    inspected = 0
    for root in roots:
        for path in root.rglob("*"):
            if path.is_symlink():
                raise AssertionError("Unexpected symlink in fixture state")
            if path.is_file():
                if path.stat().st_size > 64 * 1024 * 1024:
                    raise AssertionError("Fixture state exceeds leak inspection limit")
                data = path.read_bytes()
                if any(marker in data for marker in markers):
                    raise AssertionError(
                        "Credential appeared in plaintext fixture state"
                    )
                inspected += 1
    return inspected


def validate(args, receipt):
    token = credential(args.credential_file)
    transcripts = []
    with tempfile.TemporaryDirectory(prefix="airs-workspace-login-") as temporary:
        root = Path(temporary)
        work = root / "work"
        work.mkdir(mode=0o700)
        state = root / "state"
        env = dict(os.environ)
        for name in ["AIRS_API_KEY", "AIRS_TERMINAL_HOME", "OPENAI_API_KEY"]:
            env.pop(name, None)
        env["AIRS_HARNESS_HOME"] = str(state)
        for variable, directory in [
            ("XDG_DATA_HOME", "data"),
            ("XDG_CONFIG_HOME", "config"),
            ("XDG_RUNTIME_DIR", "runtime"),
        ]:
            path = root / directory
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
        )
        try:
            daemon.stdin.write(secrets.token_urlsafe(48).encode())
            daemon.stdin.close()
            daemon_ready(env, daemon)

            def run(arguments, *, success=True, timeout=240):
                result = subprocess.run(
                    [str(args.binary), *arguments],
                    env=env,
                    cwd=work,
                    check=False,
                    capture_output=True,
                    timeout=timeout,
                )
                transcripts.extend([result.stdout, result.stderr])
                if (result.returncode == 0) != success:
                    raise AssertionError("Unexpected harness command exit status")
                return result.stdout

            receipt["phase"] = "configure"
            run(
                [
                    "setup",
                    "--environment",
                    "workspace-e2e",
                    "--gateway-url",
                    args.gateway_url,
                ]
            )
            registry = json.loads((state / "environments.json").read_text())
            environment_id = registry["environments"]["workspace-e2e"]["id"]
            uuid.UUID(environment_id)
            home = state / "environments" / environment_id
            config = tomllib.loads((home / "config.toml").read_text())
            if config["model"] != "airs-gateway-default":
                raise AssertionError(
                    "Fixture is not configured for gateway default routing"
                )
            receipt["phase"] = "guided-login"
            receipt["gateway_login_verification"] = {
                "required": args.require_gateway_verification,
                "disclosed_probes": 0,
                "verified_probes": 0,
                "max_output_tokens_per_probe": 16,
                "captured_output_limit_bytes": MAX_LOGIN_OUTPUT_BYTES,
                "wire_inspected": False,
            }

            def guided_login():
                with TerminalSession(
                    args.binary, env, work, arguments=["login"]
                ) as terminal:
                    terminal.transcript = LoginTranscript()
                    terminal.wait_for(b"Choose 1 or 2", timeout=30)
                    os.write(terminal.master, b"2\n")
                    terminal.wait_for(b"Workspace API key (input hidden", timeout=30)
                    terminal.send_line(token.decode())
                    terminal.wait_until(
                        lambda: terminal.process.poll() is not None, timeout=90
                    )
                    # The process can exit before the driver's predicate reads
                    # its final message. Drain queued PTY bytes with the same cap.
                    while select.select([terminal.master], [], [], 0.1)[0]:
                        try:
                            remaining = os.read(terminal.master, 65536)
                        except OSError:
                            break
                        if not remaining:
                            break
                        terminal.transcript.extend(remaining)
                    output = bytes(terminal.transcript)
                    disclosed, verified = verify_login_output(
                        output, token, args.require_gateway_verification
                    )
                    transcripts.append(output)
                    receipt["gateway_login_verification"]["disclosed_probes"] += int(
                        disclosed
                    )
                    receipt["gateway_login_verification"]["verified_probes"] += int(
                        verified
                    )
                    if terminal.process.returncode:
                        raise AssertionError("Guided workspace login failed")
                binding = json.loads((home / "credential-binding.json").read_text())
                if binding["source"] != {"kind": "keyring-v2"}:
                    raise AssertionError(
                        "Workspace key was not bound to native storage"
                    )
                status_output = run(["status"])
                receipt["status_metadata_only"] = (
                    b"availability and gateway access not checked" in status_output
                )

            lifecycle_started = time.monotonic()
            guided_login()
            receipt["checks"].extend(
                [
                    "guided-hidden-login",
                    "native-keyring-binding",
                    "separate-process-status-configuration"
                    if receipt["status_metadata_only"]
                    else "separate-process-status-legacy-local-read",
                ]
            )
            if args.require_gateway_verification:
                receipt["checks"].append("bounded-authenticated-login-probe")

            if args.warm_status_runs:
                receipt["phase"] = "warm-local-status"
                samples = []
                for _ in range(args.warm_status_runs):
                    started = time.perf_counter()
                    run(["--version"], timeout=5)
                    baseline = (time.perf_counter() - started) * 1000
                    started = time.perf_counter()
                    run(["status"], timeout=5)
                    elapsed = (time.perf_counter() - started) * 1000
                    samples.append({"version_ms": baseline, "status_ms": elapsed})
                ordered = sorted(item["status_ms"] for item in samples)
                p95 = ordered[(95 * len(ordered) + 99) // 100 - 1]
                receipt["warm_local_status"] = {
                    "runs": len(samples),
                    "samples": samples,
                    "p95_ms": p95,
                    "target_ms": 500,
                    "target_met": p95 <= 500,
                    "scope": (
                        "whole status process including startup and saved native-binding metadata; no native credential read or network"
                        if receipt["status_metadata_only"]
                        else "whole legacy status process including startup, configuration and native credential access; no network"
                    ),
                    "credential_resolution_isolated": False,
                    "native_credential_resolution_measured": not receipt[
                        "status_metadata_only"
                    ],
                }

            receipt["phase"] = "live-exec"
            proof = "WORKSPACE_OK_" + secrets.token_hex(8)
            memory = "MEMORY_" + secrets.token_hex(8)
            output = run(
                [
                    "exec",
                    "--skip-git-repo-check",
                    "--sandbox",
                    "workspace-write",
                    "--json",
                    (
                        f"Write workspace-proof.txt containing exactly {proof} using one local shell "
                        f"command, then verify the file with one shell command. Remember the session-only "
                        f"marker {memory}; do not write that marker to project files. "
                        "Finish with the marker and actual verification result. Do not call MCP tools."
                    ),
                ]
            )
            proof_path = work / "workspace-proof.txt"
            if (
                proof_path.is_symlink()
                or proof_path.read_bytes().strip() != proof.encode()
            ):
                raise AssertionError("Local filesystem proof did not match")
            parsed = events(output)
            if not any(
                item.get("item", {}).get("type") == "command_execution"
                and item["item"].get("exit_code") == 0
                for item in parsed
            ):
                raise AssertionError("No successful local command event")
            thread = next(
                item["thread_id"]
                for item in parsed
                if item.get("type") == "thread.started"
            )
            uuid.UUID(thread)
            receipt["checks"].extend(
                ["live-default-route-inference", "local-file-effect"]
            )

            receipt["phase"] = "new-process-resume"
            resumed = run(
                [
                    "exec",
                    "resume",
                    "--skip-git-repo-check",
                    "--json",
                    thread,
                    "Reply only with the session-only marker from our previous turn. Do not use tools.",
                ]
            )
            replies = [
                item.get("item", {}).get("text", "")
                for item in events(resumed)
                if item.get("item", {}).get("type") == "agent_message"
            ]
            if not any(memory in reply for reply in replies):
                raise AssertionError("New-process resume did not retain session memory")
            receipt["checks"].append("new-process-resume-memory")
            receipt["phase"] = "logout"
            run(["logout"])
            run(["status"], success=False)
            run(
                ["exec", "--skip-git-repo-check", "--json", "Reply LOGOUT_MUST_BLOCK"],
                success=False,
            )
            receipt["checks"].extend(
                [
                    "logout",
                    "post-logout-status-blocked",
                    "post-logout-new-process-exec-blocked",
                ]
            )
            receipt["credential_lifecycle"] = {
                "auth_mode": "workspace-key",
                "requested_runs": args.lifecycle_runs,
                "completed_runs": 1,
                "first_run_included_live_inference_and_resume": True,
                "repeat_runs_include_inference_or_refresh": False,
                "durations_seconds": [time.monotonic() - lifecycle_started],
            }
            for index in range(1, args.lifecycle_runs):
                receipt["phase"] = f"native-lifecycle-{index + 1}"
                started = time.monotonic()
                guided_login()
                run(["logout"], timeout=10)
                run(["status"], success=False, timeout=5)
                receipt["credential_lifecycle"]["completed_runs"] += 1
                receipt["credential_lifecycle"]["durations_seconds"].append(
                    time.monotonic() - started
                )
            receipt["credential_lifecycle"][
                "repeat_runs_include_inference_or_refresh"
            ] = receipt["gateway_login_verification"]["disclosed_probes"] > 1
            receipt["credential_lifecycle"]["repeat_inference_scope"] = (
                "one disclosed fixed connectivity probe per guided login; no conversation or refresh"
                if receipt["gateway_login_verification"]["disclosed_probes"] > 1
                else "no repeated inference observed"
            )
            receipt["phase"] = "plaintext-inspection"
            receipt["files_inspected"] = assert_private(
                [state, work], transcripts, token
            )
            receipt["checks"].append("no-plaintext-key-in-state-or-transcripts")
        finally:
            daemon.terminate()
            try:
                daemon.wait(timeout=10)
            except subprocess.TimeoutExpired:
                daemon.kill()
                daemon.wait(timeout=5)
    receipt.update(passed=True, phase="completed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--gateway-url", required=True)
    parser.add_argument("--credential-file", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--warm-status-runs", type=int, choices=range(101), default=0)
    parser.add_argument("--lifecycle-runs", type=int, choices=range(1, 101), default=1)
    parser.add_argument(
        "--require-gateway-verification",
        action="store_true",
        help="Require a disclosed successful bounded probe on every login; 20 lifecycle runs make 20 minimal inference requests",
    )
    parser.add_argument("--inside-dbus", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    os.umask(0o077)
    if not sys.platform.startswith("linux"):
        parser.error(
            "This fixture requires Linux; it does not prove macOS or Windows support"
        )
    args.binary = args.binary.resolve(strict=True)
    args.credential_file = args.credential_file.absolute()
    args.output = args.output.resolve()
    gateway = urlsplit(args.gateway_url)
    if (
        gateway.scheme != "https"
        or not gateway.hostname
        or gateway.username
        or gateway.password
        or gateway.query
        or gateway.fragment
    ):
        parser.error("Gateway must use HTTPS without credentials, query or fragment")
    if not args.inside_dbus:
        return subprocess.call(
            [
                "dbus-run-session",
                "--",
                sys.executable,
                str(Path(__file__).resolve()),
                "--binary",
                str(args.binary),
                "--gateway-url",
                args.gateway_url,
                "--credential-file",
                str(args.credential_file),
                "--output",
                str(args.output),
                "--warm-status-runs",
                str(args.warm_status_runs),
                "--lifecycle-runs",
                str(args.lifecycle_runs),
                *(
                    ["--require-gateway-verification"]
                    if args.require_gateway_verification
                    else []
                ),
                "--inside-dbus",
            ]
        )
    with args.binary.open("rb") as binary:
        digest = hashlib.file_digest(binary, "sha256").hexdigest()
    receipt = {
        "schema_version": 1,
        "passed": False,
        "platform": sys.platform,
        "binary_sha256": digest,
        "gateway": args.gateway_url,
        "checks": [],
        "phase": "credential-read",
        "credential_mode": "workspace-native-keyring",
        "fixture_store": "disposable-dbus-secret-service",
        "raw_logs_retained": False,
        "mcp_tested": False,
        "live_process_logout_invalidation_tested": False,
        "model_route": "gateway-default",
        "model_field_wire_inspected": False,
        "plaintext_scan_excludes": ["encrypted-native-keyring-records"],
        "started_at_unix": int(time.time()),
    }
    try:
        validate(args, receipt)
    except Exception as error:  # noqa: BLE001 — redact potentially secret-bearing tool errors
        # TerminalSession and subprocess errors can carry complete output/keys.
        # Retain only the safe phase and exception class, never their messages.
        receipt["failure_class"] = type(error).__name__
        if isinstance(error, AssertionError) and str(error).startswith(
            "Gateway verification failed:"
        ):
            receipt["verification_failure"] = str(error)
    receipt["finished_at_unix"] = int(time.time())
    args.output.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    descriptor = os.open(
        args.output, os.O_WRONLY | os.O_CREAT | os.O_TRUNC | os.O_NOFOLLOW, 0o600
    )
    with os.fdopen(descriptor, "w") as output:
        json.dump(receipt, output, indent=2)
        output.write("\n")
    print(json.dumps(receipt))
    return 0 if receipt["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
