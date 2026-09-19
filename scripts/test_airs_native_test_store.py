"""Failure-path ownership and isolation checks for native acceptance helpers."""

import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from airs_gateway_test_identity import GatewayTestIdentity
import airs_native_test_store as store


class NativeFixtureStore(unittest.TestCase):
    def setUp(self):
        self.identity = GatewayTestIdentity("https://127.0.0.1:43210/mcp")
        self.record = {"server_name": self.identity.name, "url": self.identity.endpoint}

    def reply(self, record=None, status=0):
        return subprocess.CompletedProcess(
            [], status, json.dumps(record).encode() if record else b"", b""
        )

    def test_unowned_names_are_rejected_before_query_or_delete(self):
        identity = GatewayTestIdentity(self.identity.endpoint, name="owner-service-now")
        with patch.object(store.subprocess, "run") as run:
            for operation in (store.read_record, store.delete_record):
                with self.assertRaises(ValueError):
                    operation(identity, {})
            run.assert_not_called()

    def test_all_found_variants_are_verified_before_any_deletion(self):
        wrong = dict(self.record, server_name="owner-service-now")
        with (
            patch.object(store.sys, "platform", "darwin"),
            patch.object(
                store.subprocess,
                "run",
                side_effect=[self.reply(self.record), self.reply(wrong)],
            ) as run,
        ):
            with self.assertRaises(ValueError):
                store.delete_record(self.identity, {})
            self.assertTrue(
                all(
                    call.args[0][1] == "find-generic-password"
                    for call in run.call_args_list
                )
            )

    def test_cleanup_names_only_verified_exact_account(self):
        with (
            patch.object(store.sys, "platform", "darwin"),
            patch.object(
                store.subprocess,
                "run",
                side_effect=[
                    self.reply(self.record),
                    self.reply(status=44),
                    self.reply(),
                    self.reply(status=44),
                    self.reply(status=44),
                ],
            ) as run,
        ):
            store.delete_record(self.identity, {})
            deletion = run.call_args_list[2].args[0]
            self.assertEqual(
                deletion,
                [
                    "security",
                    "delete-generic-password",
                    "-s",
                    store.SERVICE,
                    "-a",
                    self.identity.accounts()[0],
                ],
            )

    def test_unavailable_native_store_is_not_absence(self):
        with (
            patch.object(store.sys, "platform", "darwin"),
            patch.object(store.subprocess, "run", return_value=self.reply(status=36)),
        ):
            with self.assertRaisesRegex(RuntimeError, "status 36"):
                store.read_record(self.identity, {})

    def test_linux_context_refuses_unmarked_parent_bus(self):
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.object(store.sys, "platform", "linux"),
            patch.object(store.subprocess, "run") as run,
        ):
            with self.assertRaisesRegex(RuntimeError, "private D-Bus"):
                with store.native_store(
                    Path(temporary), {"DBUS_SESSION_BUS_ADDRESS": "owner-bus"}
                ):
                    self.fail("Owner bus must not be used")
            run.assert_not_called()

    def test_child_environment_removes_inherited_credential_sources(self):
        inherited = {
            "HOME": "/owner",
            "AIRS_API_KEY": "private",
            "OPENAI_API_KEY": "private",
            "CODEX_HOME": "/owner/code",
            "CODEX_CA_CERTIFICATE": "/owner/ca",
            "SSL_CERT_FILE": "/owner/ca",
            "PATH": "/bin",
        }
        before = inherited.copy()
        with tempfile.TemporaryDirectory() as temporary:
            env = store.isolated_environment(Path(temporary), inherited)
            self.assertEqual(inherited, before)
            self.assertEqual(
                set(env)
                & {
                    "AIRS_API_KEY",
                    "OPENAI_API_KEY",
                    "CODEX_HOME",
                    "CODEX_CA_CERTIFICATE",
                    "SSL_CERT_FILE",
                },
                set(),
            )
            self.assertEqual(env["HOME"], str(Path(temporary) / "home"))
