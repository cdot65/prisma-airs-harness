"""Installed environment management never migrates existing MCP credentials."""

import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest

from airs_fixture_config import set_mcp_store
from airs_gateway_test_identity import GatewayTestIdentity

BINARY = Path(
    os.environ.get("AIRS_HARNESS_BIN", "codex-rs/target/debug/airs-harness")
).resolve()


class McpPolicyPreservation(unittest.TestCase):
    def test_legacy_modes_credentials_and_history_survive_management(self):
        with tempfile.TemporaryDirectory(prefix="airs-mcp-preserve-") as temporary:
            root = Path(temporary)
            env = {
                key: value
                for key, value in os.environ.items()
                if not key.startswith(("AIRS_", "OPENAI_"))
                and key not in {"CODEX_HOME", "CODEX_SQLITE_HOME"}
            }
            env.update(
                AIRS_HARNESS_HOME=str(root / "state"),
                HOME=str(root),
                DBUS_SESSION_BUS_ADDRESS="unix:path=" + str(root / "no-bus"),
            )

            def command(*args):
                return subprocess.run(
                    [str(BINARY), *args],
                    env=env,
                    cwd=root,
                    capture_output=True,
                    text=True,
                    timeout=30,
                )

            snapshots = {}
            for mode in ("omitted", "file", "auto", "keyring", "other"):
                result = command(
                    "env", "create", mode, "--gateway-url", "https://fixture.invalid/v1"
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                registry = json.loads((root / "state/environments.json").read_text())
                home = (
                    root / "state/environments" / registry["environments"][mode]["id"]
                )
                config = home / "config.toml"
                if mode == "omitted":
                    config.write_text(
                        re.sub(
                            r'^mcp_oauth_credentials_store = "keyring"\n',
                            "",
                            config.read_text(),
                            flags=re.MULTILINE,
                        )
                    )
                elif mode != "other":
                    set_mcp_store(config, mode)
                (home / "history.jsonl").write_text(
                    '{"session_id":"preserve-fixture","text":"Existing draft and history"}\n'
                )
                (home / ".credentials.json").write_text(
                    json.dumps(
                        {
                            GatewayTestIdentity(
                                "https://fixture.invalid/mcp", name="preserved"
                            ).accounts()[0]: {
                                "server_name": "preserved",
                                "server_url": "https://fixture.invalid/mcp",
                                "client_id": "fixture",
                                "access_token": "PRIVATE-PRESERVATION-CANARY",
                                "refresh_token": "PRIVATE-PRESERVATION-REFRESH",
                                "scopes": [],
                            }
                        }
                    )
                )
                snapshots[mode] = (
                    home,
                    {
                        path.name: path.read_bytes()
                        for path in home.iterdir()
                        if path.is_file()
                    },
                )

            for mode in ("omitted", "file", "auto", "keyring"):
                with self.subTest(mode=mode):
                    self.assertEqual(command("env", "use", mode).returncode, 0)
                    self.assertEqual(command("env", "list").returncode, 0)
                    inspection = command("doctor", "--json")
                    self.assertIn('"checks"', inspection.stdout)
                    self.assertNotIn(
                        "PRIVATE-PRESERVATION", inspection.stdout + inspection.stderr
                    )
                    duplicate = command(
                        "env",
                        "create",
                        mode,
                        "--gateway-url",
                        "https://different.invalid/v1",
                    )
                    self.assertNotEqual(duplicate.returncode, 0)
                    self.assertIn("already exists", duplicate.stderr)
                    self.assertEqual(
                        command("env", "rename", mode, mode + "-renamed").returncode, 0
                    )
                    self.assertEqual(
                        command("env", "rename", mode + "-renamed", mode).returncode, 0
                    )
                    for home, before in snapshots.values():
                        self.assertEqual(
                            {name: (home / name).read_bytes() for name in before},
                            before,
                        )
                        added = {
                            path.name for path in home.iterdir() if path.is_file()
                        } - before.keys()
                        self.assertLessEqual(added, {".configuration.lock"})
