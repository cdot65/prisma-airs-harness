"""Installed fault checks included in every candidate and registry product gate."""

import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import unittest

from airs_lifecycle_faults import exercise
from airs_lifecycle_tokens import RefreshFault
from airs_native_test_store import native_test_command


class RefreshRecovery(unittest.TestCase):
    def run_case(self, fault):
        process = subprocess.Popen(
            native_test_command(__file__, ["--case", fault.value]),
            env=os.environ.copy(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=True,
        )
        try:
            stdout, stderr = process.communicate(timeout=180)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                process.communicate(timeout=30)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.communicate(timeout=10)
            self.fail("Isolated refresh fault fixture exceeded its deadline")
        failure = {}
        if process.returncode and len(stderr) <= 4096:
            try:
                reported = json.loads(stderr)
                for key in ("error_type", "phase"):
                    value = reported.get(key)
                    if (
                        isinstance(value, str)
                        and len(value) <= 80
                        and all(c.isalnum() or c in "_-" for c in value)
                    ):
                        failure[key] = value
            except (ValueError, AttributeError):
                pass
        self.assertEqual(
            process.returncode, 0, f"Isolated refresh fault fixture failed: {failure}"
        )
        report = json.loads(stdout)
        self.assertTrue(report["passed"])
        self.assertEqual(report["case"], fault.value)
        print(json.dumps(report, sort_keys=True))

    @unittest.skipUnless(
        sys.platform.startswith("linux") or sys.platform == "darwin",
        "Released native Linux/macOS targets only",
    )
    def test_consumed_refresh_response_loss_never_replays_after_restart(self):
        self.run_case(RefreshFault.RESPONSE_LOST)

    @unittest.skipUnless(
        sys.platform.startswith("linux") or sys.platform == "darwin",
        "Released native Linux/macOS targets only",
    )
    def test_rejected_refresh_remains_signin_required_after_restart(self):
        self.run_case(RefreshFault.REJECTED)


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--case":

        def interrupted(_signal, _frame):
            raise RuntimeError("Fixture interrupted")

        signal.signal(signal.SIGTERM, interrupted)
        try:
            binary = Path(
                os.environ.get("AIRS_HARNESS_NATIVE_BIN")
                or os.environ.get(
                    "AIRS_HARNESS_BIN", "codex-rs/target/debug/airs-harness"
                )
            )
            print(json.dumps(exercise(binary, RefreshFault(sys.argv[2]))))
        except BaseException as error:
            # PTY helpers may attach callback URLs to exception messages.
            # Never emit that raw message or a traceback from this private worker.
            print(
                json.dumps(
                    {
                        "passed": False,
                        "error_type": type(error).__name__,
                        "phase": getattr(exercise, "phase", "startup"),
                    }
                ),
                file=sys.stderr,
            )
            raise SystemExit(1)
    else:
        unittest.main()
