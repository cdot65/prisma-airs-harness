"""Actual installed native MCP defaults, with disposable gateway credentials.

The positive case is required on both released native operating systems. Only
Linux's unavailable private session-bus scenario is platform-specific.
"""

from contextlib import nullcontext
import json
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import signal
import tempfile
import time
import unittest

import test_airs_harness as harness
from airs_gateway_test_identity import GatewayTestIdentity
from airs_harness_pty import TerminalSession
from airs_mcp_manager_fixture import McpGatewayFixture
from airs_native_test_store import (
    delete_record,
    isolated_environment,
    native_store,
    native_test_command,
    record_exists,
)
import test_airs_mcp_manager as manager


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def exercise(case):
    require(case in {"positive", "unavailable"}, "Unknown native fixture case")
    require(
        case != "unavailable" or sys.platform.startswith("linux"),
        "Unavailable Secret Service case requires Linux",
    )
    inference = harness.TerminalIntegration()
    inference.setUp()
    try:
        context = (
            native_store(inference.root / "native", os.environ)
            if case == "positive"
            else nullcontext(
                isolated_environment(inference.root / "native", os.environ)
            )
        )
        with context as env:
            if case == "unavailable":
                env["DBUS_SESSION_BUS_ADDRESS"] = "unix:path=" + str(
                    inference.root / "no-native-bus"
                )
            inference.env = dict(
                env,
                AIRS_HARNESS_HOME=str(inference.home),
                AIRS_TEST_CREDENTIAL="test-only-credential",
            )
            inference.configure()
            gateway = McpGatewayFixture(inference.root)
            identity = GatewayTestIdentity(gateway.endpoint)
            cleanup_owned = False
            try:
                env = dict(
                    inference.env,
                    SSL_CERT_FILE=str(gateway.certificate),
                    SSH_CONNECTION="fixture",
                )
                ui = manager.McpManager()
                ui.inference, ui.home, ui.gateway, ui.env = (
                    inference,
                    inference.home,
                    gateway,
                    env,
                )
                before = (inference.home / "config.toml").read_bytes()
                require(
                    not (inference.home / ".credentials.json").exists(),
                    "Fresh environment contains fallback credentials",
                )
                if case == "positive":
                    require(
                        not record_exists(identity, env),
                        "Unique native record already exists",
                    )
                    cleanup_owned = True
                    result = positive(ui, identity, env)
                else:
                    result = unavailable(ui, identity, env)
                require(
                    not (inference.home / ".credentials.json").exists(),
                    "MCP login wrote plaintext fallback credentials",
                )
                for rollout in inference.home.rglob("*.jsonl"):
                    text = rollout.read_text()
                    require(
                        "PRIVATE-MANAGER-CODE" not in text
                        and "fixture-mcp-access" not in text,
                        "MCP credential leaked into history",
                    )
                # Connection edits may change the configuration; inference fields
                # and original policy remain byte-identical in the retained prefix.
                require(
                    (inference.home / "config.toml").read_bytes().startswith(before),
                    "MCP workflow changed existing inference configuration",
                )
                with harness.BINARY.open("rb") as native:
                    digest = hashlib.file_digest(native, "sha256").hexdigest()
                return {
                    "case": case,
                    "passed": True,
                    "platform": sys.platform,
                    "executable_sha256": digest,
                    "native_mode_unmodified": True,
                    **result,
                }
            finally:
                try:
                    if cleanup_owned:
                        delete_record(identity, env)
                finally:
                    gateway.close()
    finally:
        inference.doCleanups()


def positive(ui, identity, env):
    with TerminalSession(harness.BINARY, env, ui.inference.work) as terminal:
        terminal.start()
        ui.add(terminal, identity.name)
        callback = ui.callback(terminal)
        ui.key(terminal, b"\x1b[200~" + callback.encode() + b"\x1b[201~\r")
        terminal.wait_for(b"MCP connection updated", timeout=45)
        terminal.wait_for(b"connected")
        require(
            record_exists(identity, env),
            "OAuth did not persist a native record",
        )
        require(
            not (ui.home / ".credentials.json").exists(), "OAuth wrote a fallback file"
        )
        require(
            "PRIVATE-MANAGER-CODE".encode() not in terminal.transcript,
            "Callback was echoed",
        )
        ui.choose(terminal)  # Explicit new conversation, preserving old history.
        terminal.wait_for(b"Review your draft")
    initial_discovery = sum(
        request["method"] == "tools/list" for request in ui.gateway.requests
    )
    require(initial_discovery > 0, "Native login did not discover MCP tools")
    require(
        len(ui.gateway.tokens) == 1, "Initial login exchanged the code more than once"
    )

    # A separate real process loads the committed generation. This directory
    # was already trusted by the first terminal, so no second trust prompt.
    with TerminalSession(harness.BINARY, env, ui.inference.work) as terminal:
        terminal.wait_for(b"permissions:")
        # The header renders before native MCP startup finishes. Retry only
        # the passive menu command after its explicit busy response; never
        # replay sign-in, reconnect, or a tool operation.
        deadline = time.monotonic() + 30
        busy = (
            b"Finish or interrupt the current request before managing MCP connections."
        )
        while True:
            offset = len(terminal.transcript)
            terminal.send_line("/mcp")
            terminal.wait_until(
                lambda: (
                    b"Add gateway MCP server" in terminal.transcript[offset:]
                    or busy in terminal.transcript[offset:]
                ),
                timeout=max(0, deadline - time.monotonic()),
            )
            if b"Add gateway MCP server" in terminal.transcript[offset:]:
                break
            require(time.monotonic() < deadline, "MCP startup did not become ready")
        ui.choose(terminal)
        terminal.wait_for(b"Reconnect and verify")
        offset = len(terminal.transcript)
        ui.choose(terminal, 2)
        terminal.wait_for(b"MCP connection updated", offset, timeout=45)
        require(
            sum(request["method"] == "tools/list" for request in ui.gateway.requests)
            > initial_discovery,
            "Second process did not discover MCP tools",
        )
        require(
            len(ui.gateway.tokens) == 1,
            "Second process performed an unnecessary OAuth exchange",
        )
        ui.choose(terminal)
        terminal.wait_for(b"Review your draft", offset)
        terminal.send_line("/mcp")
        terminal.wait_for(b"MCP connections", offset)
        ui.choose(terminal)
        terminal.wait_for(b"Sign out", offset)
        ui.choose(terminal, 1)
        terminal.wait_for(f"Sign out of {identity.name}?".encode(), offset)
        ui.choose(terminal, 1)
        terminal.wait_for(b"signed out", offset, timeout=45)
        require(not record_exists(identity, env), "Logout retained the native token")
        ui.choose(terminal, 1)  # Manage, without replaying a turn.
        offset = len(terminal.transcript)
        terminal.wait_for(b"MCP connections", offset)
        ui.choose(terminal)
        terminal.wait_for(b"Remove connection", offset)
        ui.choose(terminal, 3)
        terminal.wait_for(f"Remove {identity.name}?".encode(), offset)
        ui.choose(terminal, 1)
        terminal.wait_for(b"removed from this environment", offset, timeout=45)
        ui.choose(terminal)
        terminal.wait_for(b"Review your draft", offset)
        ui.inference.requests.append(("fixture-bootstrap", {}, {}))
        ui.inference.phase_replies = {
            "Confirm inference survives native MCP": "Native inference preserved"
        }
        terminal.send_line("Confirm inference survives native MCP")
        terminal.wait_for(b"Native inference preserved", timeout=45)
        require(
            b"PRIVATE-MANAGER-CODE" not in terminal.transcript,
            "Callback leaked into terminal output",
        )
    require(
        identity.name not in (ui.home / "config.toml").read_text(),
        "Removed connection remains configured",
    )
    for path, headers, body in ui.inference.requests[1:]:
        require(path == "/prefix/v1/responses", "Inference destination changed")
        require(
            "fixture-mcp-access" not in json.dumps(headers)
            and "PRIVATE-MANAGER-CODE" not in json.dumps(body),
            "MCP secret reached inference",
        )
    require(len(ui.inference.requests) > 1, "Post-MCP inference was not exercised")
    return {
        "native_record_persisted": True,
        "second_process_verified": True,
        "native_logout_verified": True,
        "inference_preserved": True,
        "oauth_exchanges": len(ui.gateway.tokens),
    }


def unavailable(ui, identity, env):
    # Use the same built-in login adapter as /mcp; unlike an add-and-login TUI,
    # a CLI subprocess exposes an unambiguous exit status for the safe failure.
    config = ui.home / "config.toml"
    config.write_text(
        config.read_text()
        + f'\n[mcp_servers.{identity.name}]\nurl = "{identity.endpoint}"\n'
    )
    with TerminalSession(
        harness.BINARY,
        env,
        ui.inference.work,
        arguments=["mcp", "login", identity.name, "--no-browser"],
    ) as terminal:
        terminal.wait_until(
            lambda: (
                b"Callback URL (input hidden):" in terminal.transcript
                or terminal.process.poll() is not None
            )
        )
        if terminal.process.poll() is None:
            import re
            from urllib.parse import parse_qs, urlencode, urlsplit, urlunsplit

            authorization = re.search(
                rb"https://127\.0\.0\.1:[0-9]+/authorize\?[^\s]+", terminal.transcript
            )
            require(
                authorization is not None, "Fixture authorization URL was not offered"
            )
            params = parse_qs(urlsplit(authorization[0].decode()).query)
            callback = urlunsplit(
                urlsplit(params["redirect_uri"][0])._replace(
                    query=urlencode(
                        {
                            "state": params["state"][0],
                            "code": "PRIVATE-MANAGER-CODE",
                            "iss": ui.gateway.base,
                        }
                    )
                )
            )
            terminal.send_line(callback)
        terminal.wait_until(lambda: terminal.process.poll() is not None)
        require(
            terminal.process.returncode != 0,
            "Unavailable native store reported successful login",
        )
        require(
            b"Successfully logged in" not in terminal.transcript,
            "Unavailable native store claimed success",
        )
        require(
            b"PRIVATE-MANAGER-CODE" not in terminal.transcript, "Callback was echoed"
        )
        require(
            b"keyring" in terminal.transcript.lower(),
            "Failure omitted credential-store context",
        )
    require(
        ui.inference.requests == [],
        "Failed MCP sign-in unexpectedly submitted inference",
    )
    return {
        "fallback_absent": True,
        "login_failed_safely": True,
        "oauth_exchanges": len(ui.gateway.tokens),
    }


class NativeMcpDefault(unittest.TestCase):
    def run_case(self, case):
        with subprocess.Popen(
            native_test_command(__file__, ["--case", case]),
            env=os.environ.copy(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=True,
        ) as process:
            try:
                stdout, stderr = process.communicate(timeout=180)
                result = subprocess.CompletedProcess(
                    process.args, process.returncode, stdout, stderr
                )
            except subprocess.TimeoutExpired:
                # Reap only this fixture's process group. The worker handles
                # SIGTERM so its PTY and exact-record cleanup can finish.
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    stdout, stderr = process.communicate(timeout=30)
                except subprocess.TimeoutExpired:
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    stdout, stderr = process.communicate(timeout=5)
                result = subprocess.CompletedProcess(
                    process.args, 124, stdout, stderr + "\nNative fixture timed out"
                )
        if result.returncode:
            with tempfile.NamedTemporaryFile(
                prefix="airs-native-mcp-failure-", suffix=".log", mode="w", delete=False
            ) as diagnostics:
                diagnostics.write(result.stdout + result.stderr)
            self.fail(
                f"Native MCP fixture failed (status {result.returncode}); private diagnostics: {diagnostics.name}"
            )
        report = json.loads(result.stdout)
        self.assertTrue(report["passed"])
        self.assertEqual(report["case"], case)
        print(json.dumps(report, sort_keys=True))

    @unittest.skipUnless(
        sys.platform.startswith("linux") or sys.platform == "darwin",
        "Released Linux/macOS native targets only",
    )
    def test_new_environment_uses_native_storage_across_processes(self):
        self.run_case("positive")

    @unittest.skipUnless(
        sys.platform.startswith("linux"),
        "Unavailable private Secret Service is Linux-specific",
    )
    def test_unavailable_native_store_never_falls_back_to_file(self):
        self.run_case("unavailable")


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--case":

        def interrupted(_signal, _frame):
            raise RuntimeError("Native fixture interrupted; cleaning up owned state")

        signal.signal(signal.SIGTERM, interrupted)
        print(json.dumps(exercise(sys.argv[2])))
    else:
        unittest.main()
