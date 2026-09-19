"""Exercise retained lifecycle evidence against corruption and credential leaks."""

from pathlib import Path
import tempfile
import threading
from types import SimpleNamespace
import unittest

from airs_lifecycle_session import LifecycleSession


class LifecycleHistory(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.session = LifecycleSession.__new__(LifecycleSession)
        self.session.home = Path(temporary.name)
        self.session.history = {}
        self.session.terminal = SimpleNamespace(transcript=b"Ready\n")
        resources = {}
        self.canaries = ["PRIVATE-MANAGER-CODE"]
        for name in ("inference", "mcp"):
            access, refresh, consumed = (
                f"synthetic-{name}-{kind}-private"
                for kind in ("access", "refresh", "consumed")
            )
            resources[name] = SimpleNamespace(
                lock=threading.RLock(),
                access={access: (1, 99)},
                refresh={refresh: 1},
                consumed={consumed},
            )
            self.canaries.extend((access, refresh, consumed))
        self.session.fixture = SimpleNamespace(**resources)
        self.rollout = self.session.home / "sessions" / "fixture.jsonl"
        self.rollout.parent.mkdir()
        self.original = b'{"type":"session_meta","id":"conversation"}\n'
        self.rollout.write_bytes(self.original)
        self.session.check_retained_history()

    def test_each_credential_canary_in_transcript_is_rejected_without_disclosure(self):
        for canary in self.canaries:
            with self.subTest(credential=self.canaries.index(canary)):
                self.session.terminal.transcript = b"Ready\n" + canary.encode()
                with self.assertRaisesRegex(RuntimeError, "TUI transcript") as failure:
                    self.session.check_retained_history()
                self.assertNotIn(canary, str(failure.exception))

    def test_each_credential_canary_in_rollout_is_rejected_without_disclosure(self):
        for canary in self.canaries:
            with self.subTest(credential=self.canaries.index(canary)):
                self.rollout.write_bytes(self.original + canary.encode())
                with self.assertRaisesRegex(
                    RuntimeError, "retained history"
                ) as failure:
                    self.session.check_retained_history()
                self.assertNotIn(canary, str(failure.exception))

    def test_rewritten_or_truncated_history_is_rejected(self):
        for changed in (b"replacement\n", self.original[:-1]):
            with self.subTest(length=len(changed)):
                self.rollout.write_bytes(changed)
                with self.assertRaisesRegex(RuntimeError, "history was rewritten"):
                    self.session.check_retained_history()

    def test_deleted_history_is_rejected(self):
        self.rollout.unlink()
        with self.assertRaisesRegex(RuntimeError, "history file disappeared"):
            self.session.check_retained_history()

    def test_valid_append_updates_retained_prefix(self):
        appended = self.original + b'{"type":"message","text":"synthetic read"}\n'
        self.rollout.write_bytes(appended)
        self.session.check_retained_history()
        self.assertEqual(self.session.history, {self.rollout: appended})
        self.rollout.write_bytes(self.original)
        with self.assertRaisesRegex(RuntimeError, "history was rewritten"):
            self.session.check_retained_history()
