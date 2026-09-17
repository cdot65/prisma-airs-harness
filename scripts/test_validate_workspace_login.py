import json
import subprocess
import traceback
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from validate_workspace_login import (
    GATEWAY_DISCLOSURE,
    GATEWAY_VERIFIED,
    MAX_LOGIN_OUTPUT_BYTES,
    LoginTranscript,
    measure_warm_local_access,
    timing_summary,
    verify_login_output,
)


class WorkspaceLoginEvidence(unittest.TestCase):
    def test_required_probe_needs_disclosure_and_success(self):
        for output in [b"Credential saved", GATEWAY_DISCLOSURE, GATEWAY_VERIFIED]:
            with self.assertRaises(AssertionError):
                verify_login_output(output, b"hidden-key-canary", True)
        self.assertEqual(
            verify_login_output(
                GATEWAY_DISCLOSURE + b"\n" + GATEWAY_VERIFIED,
                b"hidden-key-canary",
                True,
            ),
            (True, True),
        )

    def test_optional_probe_preserves_old_fixture_and_still_rejects_key_echo(self):
        self.assertEqual(
            verify_login_output(b"Credential saved", b"hidden-key-canary", False),
            (False, False),
        )
        for required in [False, True]:
            with self.assertRaises(AssertionError):
                verify_login_output(
                    GATEWAY_DISCLOSURE + GATEWAY_VERIFIED + b"hidden-key-canary",
                    b"hidden-key-canary",
                    required,
                )

    def test_transcript_enforces_limit_before_extending(self):
        output = LoginTranscript()
        output.extend(b"x" * MAX_LOGIN_OUTPUT_BYTES)
        with self.assertRaises(AssertionError):
            output.extend(b"overflow")
        self.assertEqual(len(output), MAX_LOGIN_OUTPUT_BYTES)


PRIVATE_CANARY = b"synthetic-private-canary"


class PrivateCredentialMeasurements(unittest.TestCase):
    def measure(self, runs=1):
        public = Mock()
        samples = measure_warm_local_access(
            Path("/trusted/airs-harness"),
            {},
            Path("/work"),
            Path("/state/home"),
            "binding-uuid",
            PRIVATE_CANARY,
            runs,
            public,
        )
        return samples, public

    def test_thirty_paired_processes_verify_private_output_without_retaining_it(self):
        result = subprocess.CompletedProcess([], 0, b"synthetic-private-canary\n", b"")
        # Each whole process takes 100ms. No baseline subtraction is performed.
        with (
            patch(
                "validate_workspace_login.subprocess.run", return_value=result
            ) as native,
            patch(
                "validate_workspace_login.time.perf_counter",
                side_effect=[x / 10 for x in range(180)],
            ),
        ):
            samples, public = self.measure(30)
        self.assertEqual(len(samples), 30)
        self.assertEqual(native.call_count, 30)
        self.assertEqual(public.call_count, 60)
        self.assertEqual(public.call_args_list[0].args, (["--version"],))
        self.assertEqual(public.call_args_list[1].args, (["env", "status"],))
        for call in native.call_args_list:
            self.assertEqual(
                call.args[0],
                [
                    "/trusted/airs-harness",
                    "credential",
                    "--home",
                    "/state/home",
                    "--binding",
                    "binding-uuid",
                ],
            )
            self.assertEqual(call.kwargs["timeout"], 5)
            self.assertTrue(call.kwargs["capture_output"])
        for field in ["version_ms", "status_ms", "credential_helper_ms"]:
            for sample in samples:
                self.assertAlmostEqual(sample[field], 100)
        receipt = timing_summary(samples, "credential_helper_ms")
        self.assertTrue(receipt["target_met"])
        self.assertNotIn("synthetic-private-canary", json.dumps(receipt))
        self.assertNotIn("stdout", json.dumps(receipt))

    def test_mismatch_newline_stderr_and_exit_failure_are_generic(self):
        for result in [
            subprocess.CompletedProcess([], 0, b"wrong-secret\n", b""),
            subprocess.CompletedProcess([], 0, b"synthetic-private-canary", b""),
            subprocess.CompletedProcess([], 0, b"synthetic-private-canary\r\n", b""),
            subprocess.CompletedProcess(
                [], 0, b"synthetic-private-canary\n", b"secret-error"
            ),
            subprocess.CompletedProcess([], 1, b"synthetic-private-canary\n", b""),
        ]:
            with (
                self.subTest(exit_code=result.returncode),
                patch("validate_workspace_login.subprocess.run", return_value=result),
                self.assertRaisesRegex(
                    AssertionError, "^Private credential helper measurement failed$"
                ),
            ):
                self.measure()

    def test_timeout_does_not_format_partial_secret_output_or_cause(self):
        failure = subprocess.TimeoutExpired(
            ["secret-argument-canary"],
            5,
            output=b"synthetic-private-canary",
            stderr=b"secret-error-canary",
        )
        with patch("validate_workspace_login.subprocess.run", side_effect=failure):
            try:
                self.measure()
            except AssertionError as error:
                output = "".join(traceback.format_exception(error))
                for secret in [
                    "secret-argument-canary",
                    "synthetic-private-canary",
                    "secret-error-canary",
                ]:
                    self.assertNotIn(secret, output)
            else:
                self.fail("Timeout must fail acceptance")

    def test_zero_runs_performs_no_processes_and_p95_preserves_slow_samples(self):
        with patch("validate_workspace_login.subprocess.run") as native:
            samples, public = self.measure(0)
        self.assertEqual(samples, [])
        native.assert_not_called()
        public.assert_not_called()
        samples = [{"credential_helper_ms": x} for x in [100] * 28 + [501, 900]]
        receipt = timing_summary(samples, "credential_helper_ms")
        self.assertEqual(receipt["p95_ms"], 501)
        self.assertFalse(receipt["target_met"])


if __name__ == "__main__":
    unittest.main()
