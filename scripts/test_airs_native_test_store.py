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
            for operation in (store.record_exists, store.delete_record):
                with self.assertRaises(ValueError):
                    operation(identity, {})
            run.assert_not_called()

    def test_all_variants_are_queried_before_any_deletion(self):
        with (
            patch.object(store.sys, "platform", "darwin"),
            patch.object(
                store.subprocess,
                "run",
                side_effect=[self.reply(), self.reply(status=36)],
            ) as run,
        ):
            with self.assertRaisesRegex(RuntimeError, "status 36"):
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
                    self.reply(),
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
                store.record_exists(self.identity, {})

    def test_darwin_presence_never_requests_password_data(self):
        with (
            patch.object(store.sys, "platform", "darwin"),
            patch.object(
                store.subprocess,
                "run",
                side_effect=[self.reply(), self.reply(status=44)],
            ) as run,
        ):
            self.assertIs(store.record_exists(self.identity, {}), True)
            self.assertEqual(
                [call.args[0] for call in run.call_args_list],
                [
                    [
                        "security",
                        "find-generic-password",
                        "-s",
                        store.SERVICE,
                        "-a",
                        account,
                    ]
                    for account in self.identity.accounts()
                ],
            )

    def test_darwin_missing_is_absence(self):
        with patch.object(store.sys, "platform", "darwin"):
            with patch.object(
                store.subprocess, "run", return_value=self.reply(status=44)
            ):
                self.assertIs(store.record_exists(self.identity, {}), False)

    def test_linux_verifies_record_identity_before_cleanup(self):
        with (
            patch.object(store.sys, "platform", "linux"),
            patch.object(store, "_has_secret_service", return_value=True),
            patch.object(
                store.subprocess,
                "run",
                side_effect=[
                    self.reply(self.record),
                    self.reply(dict(self.record, server_name="owner-service-now")),
                ],
            ) as run,
        ):
            with self.assertRaises(ValueError):
                store.delete_record(self.identity, {})
            self.assertTrue(
                all(call.args[0][1] == "lookup" for call in run.call_args_list)
            )

    def test_darwin_keeps_gui_home_and_isolates_fixture_directories(self):
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.object(store.sys, "platform", "darwin"),
        ):
            inherited = {"HOME": "/Users/fixture", "AIRS_HARNESS_HOME": "/owner/airs"}
            env = store.isolated_environment(Path(temporary), inherited)
            self.assertEqual(env["HOME"], inherited["HOME"])
            self.assertNotIn("AIRS_HARNESS_HOME", env)
            for variable, directory in [
                ("XDG_DATA_HOME", "data"),
                ("XDG_CONFIG_HOME", "config"),
                ("XDG_RUNTIME_DIR", "runtime"),
            ]:
                self.assertEqual(env[variable], str(Path(temporary) / directory))
            for invalid in ({}, {"HOME": "relative"}):
                with self.assertRaisesRegex(ValueError, "GUI session HOME"):
                    store.isolated_environment(Path(temporary), invalid)

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
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.object(store.sys, "platform", "linux"),
        ):
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
