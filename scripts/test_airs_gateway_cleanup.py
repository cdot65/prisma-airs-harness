import json
from pathlib import Path
import tempfile
import unittest

from airs_gateway_cleanup import release_finished_peers, wait_for_cleanup_release


class CleanupTests(unittest.TestCase):
    def test_controller_holds_failed_peer_until_other_workflow_finishes(self):
        values = {"linux": {"nonce": "a" * 48, "workflow_passed": False}, "mac": None}
        writes = []
        release = lambda peer, value: writes.append((peer, value))
        self.assertFalse(release_finished_peers(values, values.get, release))
        self.assertEqual(writes, [])
        values["mac"] = {"nonce": "b" * 48, "workflow_passed": True}
        self.assertTrue(release_finished_peers(values, values.get, release))
        self.assertEqual(writes, [("linux", {"nonce": "a" * 48}), ("mac", {"nonce": "b" * 48})])

    def test_failed_peer_waits_for_matching_controller_release(self):
        with tempfile.TemporaryDirectory() as root:
            directory = Path(root) / "cleanup"
            waits = []

            def sleep(seconds):
                ready = json.loads((directory / "ready.json").read_text())
                self.assertFalse(ready["workflow_passed"])
                waits.append(seconds)
                # An unrelated/stale release cannot revoke an active peer.
                nonce = "wrong" if len(waits) == 1 else ready["nonce"]
                (directory / "release.json").write_text(json.dumps({"nonce": nonce}))

            wait_for_cleanup_release(directory, workflow_passed=False, sleep=sleep)
            self.assertEqual(waits, [5, 5])

    def test_missing_controller_times_out(self):
        with tempfile.TemporaryDirectory() as root:
            now = [0]

            def sleep(seconds):
                now[0] += seconds

            with self.assertRaises(TimeoutError):
                wait_for_cleanup_release(
                    Path(root) / "cleanup",
                    workflow_passed=True,
                    timeout_seconds=7,
                    monotonic=lambda: now[0],
                    sleep=sleep,
                )
            self.assertEqual(now[0], 7)

    def test_existing_coordination_state_is_rejected(self):
        with tempfile.TemporaryDirectory() as root:
            with self.assertRaises(FileExistsError):
                wait_for_cleanup_release(Path(root), workflow_passed=False)


if __name__ == "__main__":
    unittest.main()
