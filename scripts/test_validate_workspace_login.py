import unittest

from validate_workspace_login import GATEWAY_DISCLOSURE
from validate_workspace_login import GATEWAY_VERIFIED
from validate_workspace_login import LoginTranscript
from validate_workspace_login import MAX_LOGIN_OUTPUT_BYTES
from validate_workspace_login import verify_login_output


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


if __name__ == "__main__":
    unittest.main()
