"""Installed continuous TUI lifecycle fixture, with private native credentials."""

import base64
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import re
import ssl
import subprocess
import time
from urllib.request import urlopen

from airs_gateway_test_identity import GatewayTestIdentity
from airs_harness_pty import TerminalSession
from airs_lifecycle_fixture import LifecycleFixture
from airs_native_test_store import delete_record, native_store, record_exists
from test_airs_mcp_manager import McpManager


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


class LifecycleSession:
    def __init__(self, binary, root, *, token_lifetime_seconds, event_sink):
        self.binary, self.root = Path(binary).resolve(), Path(root).resolve()
        self.lifetime, self.emit = token_lifetime_seconds, event_sink
        self.stack = ExitStack()
        self.terminal = None
        self.conversation_id = None
        self.active_rollout = None
        self.login_owned = False
        self.cleanup_owned = False
        self.history = {}

    def __enter__(self):
        try:
            self.root.mkdir(mode=0o700, parents=True, exist_ok=True)
            self.work = self.root / "work"
            self.work.mkdir(mode=0o700)
            self.env = self.stack.enter_context(
                native_store(self.root / "native", os.environ)
            )
            self.fixture = LifecycleFixture(self.root, self.lifetime, self.emit)
            self.stack.callback(self.fixture.close)
            self.identity = GatewayTestIdentity(self.fixture.gateway.endpoint)
            self.env.update(
                AIRS_HARNESS_HOME=str(self.root / "state"),
                SSL_CERT_FILE=str(self.fixture.certificate),
                SSH_CONNECTION="fixture",
            )
            require(
                not record_exists(self.identity, self.env),
                "Unique lifecycle native record exists",
            )
            self.cleanup_owned = True
            self.stack.callback(self.cleanup_credentials)
            return self
        except BaseException:
            self.stack.close()
            raise

    def __exit__(self, *exception):
        return self.stack.__exit__(*exception)

    def command(self, *args):
        result = subprocess.run(
            [str(self.binary), *args],
            env=self.env,
            cwd=self.work,
            capture_output=True,
            text=True,
            timeout=40,
        )
        require(result.returncode == 0, f"Lifecycle CLI command failed: {args[0]}")
        return result.stdout

    def cleanup_credentials(self):
        try:
            if self.cleanup_owned:
                delete_record(self.identity, self.env)
        finally:
            if self.login_owned:
                self.command("--environment", "lifecycle", "logout")

    def start(self):
        issuer = self.fixture.identity
        self.command(
            "env",
            "create",
            "lifecycle",
            "--gateway-url",
            issuer.issuer + "/v1",
            "--context-window",
            "32768",
        )
        registry = json.loads((self.root / "state/environments.json").read_text())
        self.home = (
            self.root
            / "state/environments"
            / registry["environments"]["lifecycle"]["id"]
        )
        # This fresh environment belongs to the fixture even if login fails after saving.
        self.login_owned = True
        with TerminalSession(
            self.binary,
            self.env,
            self.work,
            arguments=[
                "login",
                "--issuer-url",
                issuer.issuer,
                "--oidc-client-id",
                issuer.client,
                "--audience",
                issuer.audience,
                "--no-browser",
            ],
        ) as login:
            login.wait_for(b"Open this URL to sign in:")
            pattern = rb"https://127\.0\.0\.1:[0-9]+/authorize\?[^\s]+"
            login.wait_until(lambda: re.search(pattern, login.transcript) is not None)
            authorization = re.search(pattern, login.transcript)[0].decode()
            with urlopen(
                authorization,
                context=ssl.create_default_context(
                    cafile=str(self.fixture.certificate)
                ),
                timeout=15,
            ) as response:
                require(response.status == 200, "Synthetic browser consent failed")
            login.wait_until(lambda: login.process.poll() is not None, timeout=30)
            require(login.process.returncode == 0, "Synthetic signed OIDC login failed")
        self.terminal = self.stack.enter_context(
            TerminalSession(self.binary, self.env, self.work)
        )
        self.terminal.start()
        ui = McpManager()
        ui.gateway = self.fixture.gateway
        ui.add(self.terminal, self.identity.name)
        callback = ui.callback(self.terminal)
        encoded = re.findall(
            rb"\x1b\]52;[^;]*;([A-Za-z0-9+/=]+)\x07", self.terminal.transcript
        )[-1]
        self.fixture.approve_mcp(base64.b64decode(encoded).decode(), callback)
        ui.key(self.terminal, b"\x1b[200~" + callback.encode() + b"\x1b[201~\r")
        self.terminal.wait_for(b"MCP connection updated", timeout=45)
        require(
            record_exists(self.identity, self.env),
            "MCP credential was not persisted natively",
        )
        require(
            not (self.home / ".credentials.json").exists(),
            "Lifecycle MCP used plaintext fallback",
        )
        ui.choose(self.terminal)
        self.terminal.wait_for(b"Review your draft", timeout=30)
        self.terminal.send_line("/rename Lifecycle fixture")
        self.pump_until(time.monotonic() + 1)
        return self.snapshot()

    def turn(self, label):
        require(
            re.fullmatch(r"[a-z0-9_-]{1,48}", label), "Invalid lifecycle turn label"
        )
        prompt = self.fixture.prepare_turn(label)
        offset = len(self.terminal.transcript)
        self.terminal.send_line(prompt)
        completed = ("LIFECYCLE_OK_" + label).encode()
        self.terminal.wait_until(
            lambda: (
                completed in self.terminal.transcript[offset:]
                or b"Allow for this session" in self.terminal.transcript[offset:]
            ),
            timeout=45,
        )
        if completed not in self.terminal.transcript[offset:]:
            screen = self.terminal.transcript[offset:]
            require(
                self.identity.name.encode() in screen and b"incident_lookup" in screen,
                "Unexpected tool approval in lifecycle fixture",
            )
            # Approve only this synthetic read-only lookup for the current conversation.
            time.sleep(0.2)
            os.write(self.terminal.master, b"\x1b[B\r")
        self.terminal.wait_for(completed, offset, timeout=45)
        require(
            self.fixture.failure is None,
            self.fixture.failure or "Lifecycle fixture failed",
        )
        row = self.fixture.turns[label]
        require(
            row["tool_calls"] == 1 and row["output_verified"],
            "Lifecycle actual tool result was not verified",
        )
        self.pump_until(time.monotonic() + 0.5)
        candidates = [
            path
            for path in (self.home / "sessions").rglob("*.jsonl")
            if prompt in path.read_text()
        ]
        require(
            len(candidates) == 1,
            "Lifecycle turn changed or duplicated its conversation",
        )
        path = candidates[0]
        meta = next(
            (
                json.loads(line)["payload"]
                for line in path.read_text().splitlines()
                if json.loads(line).get("type") == "session_meta"
            ),
            None,
        )
        require(meta is not None, "Lifecycle rollout lacks session metadata")
        if self.conversation_id is not None:
            require(
                meta["id"] == self.conversation_id and path == self.active_rollout,
                "Lifecycle conversation changed",
            )
        self.conversation_id, self.active_rollout = meta["id"], path
        self.emit(
            "turn_completed",
            turn=label,
            unique_call_count=1,
            final_reply_verified=True,
            conversation_id=self.conversation_id,
            process_id=self.terminal.process.pid,
        )
        return self.snapshot()

    def pump_until(self, deadline_monotonic):
        while time.monotonic() < deadline_monotonic:
            require(
                self.terminal.process.poll() is None, "Lifecycle TUI process exited"
            )
            boundary = min(deadline_monotonic, time.monotonic() + 30)
            self.terminal.wait_until(lambda: time.monotonic() >= boundary, timeout=31)

    def snapshot(self):
        self.check_retained_history()
        digest = lambda name: hashlib.sha256(
            (self.home / name).read_bytes()
        ).hexdigest()
        return {
            "process_id": self.terminal.process.pid,
            "process_alive": self.terminal.process.poll() is None,
            "conversation_id": self.conversation_id,
            "environment": "lifecycle",
            "binding_sha256": digest("credential-binding.json"),
            "auth_epoch_sha256": digest("auth-generation"),
            "gateway_origin": "synthetic-loopback",
            "inference_identity": "signed-synthetic-oidc",
            "mcp_connection": self.identity.name,
            "rollout_paths_count": len(list((self.home / "sessions").rglob("*.jsonl"))),
            "resource_generations": {
                "inference": self.fixture.inference.generation,
                "mcp": self.fixture.mcp.generation,
            },
            "resource_expiries": {
                "inference": self.fixture.inference.expires,
                "mcp": self.fixture.mcp.expires,
            },
            "event_count": self.emit.count,
            "network_request_count": len(self.fixture.identity.requests)
            + len(self.fixture.gateway.network_requests),
        }

    def check_retained_history(self):
        paths = set((self.home / "sessions").rglob("*.jsonl"))
        require(set(self.history) <= paths, "Lifecycle history file disappeared")
        require(len(paths) <= 32, "Lifecycle history file count exceeded bound")
        secrets = {"PRIVATE-MANAGER-CODE"}
        for resource in (self.fixture.inference, self.fixture.mcp):
            with resource.lock:
                secrets.update(resource.access)
                secrets.update(resource.refresh)
                secrets.update(resource.consumed)
        transcript = self.terminal.transcript
        require(
            len(transcript) <= 64 * 1024 * 1024,
            "Lifecycle terminal evidence exceeded bound",
        )
        require(
            not any(secret.encode() in transcript for secret in secrets),
            "Synthetic credential leaked into TUI transcript",
        )
        for path in paths:
            require(
                path.stat().st_size <= 16 * 1024 * 1024,
                "Lifecycle rollout exceeded evidence bound",
            )
            content = path.read_bytes()
            require(
                content.startswith(self.history.get(path, b"")),
                "Lifecycle history was rewritten",
            )
            require(
                not any(secret.encode() in content for secret in secrets),
                "Synthetic credential leaked into retained history",
            )
            self.history[path] = content
