import base64
import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

from airs_oidc_evidence import invocation_provenance, jwt_expiry, observe_expiries


def token(audience, expiry=200):
    claims = {"sub": "fixture-user", "aud": audience, "exp": expiry}
    encoded = base64.urlsafe_b64encode(json.dumps(claims).encode()).decode()
    return "header." + encoded + ".synthetic-secret-canary"


class PrivateObservation(unittest.TestCase):
    @patch("airs_oidc_evidence.time.time", return_value=100)
    def test_two_private_resources_return_only_actual_expiries(self, _clock):
        run = Mock(
            side_effect=[
                subprocess.CompletedProcess(
                    [], 0, token("airs-terminal-inference") + "\n", ""
                ),
                subprocess.CompletedProcess(
                    [],
                    0,
                    json.dumps(
                        {"x-portkey-api-key": token("airs-terminal-security", 230)}
                    ),
                    "",
                ),
            ]
        )
        self.assertEqual(
            observe_expiries(run, ["credential"], ["mcp-credential"], "fixture-user"),
            {"inference": 200, "mcp": 230},
        )

    @patch("airs_oidc_evidence.time.time", return_value=100)
    def test_wrong_resource_malformed_expiry_and_timeout_do_not_expose_secrets(
        self, _clock
    ):
        for raw in [
            "synthetic-secret-canary",
            token("wrong-resource"),
            token("expected", True),
            token("expected", 99),
            token("expected", 701),
        ]:
            with self.subTest(raw_is_canary=raw == "synthetic-secret-canary"):
                with self.assertRaisesRegex(
                    AssertionError, "Invalid private credential observation"
                ) as error:
                    jwt_expiry(raw, "fixture-user", "expected")
                self.assertNotIn(raw, str(error.exception))
        run = Mock(
            side_effect=subprocess.TimeoutExpired(
                ["credential"], 30, output=b"synthetic-secret-canary"
            )
        )
        with self.assertRaisesRegex(
            AssertionError, "Private credential observation failed"
        ) as error:
            observe_expiries(run, ["credential"], ["mcp"], "fixture-user")
        self.assertTrue(error.exception.__suppress_context__)
        self.assertNotIn("synthetic-secret-canary", str(error.exception))


class InvocationProvenance(unittest.TestCase):
    def test_distinguishes_installed_launcher_and_verified_native(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            launcher = root / "bin/airs-harness.js"
            launcher.parent.mkdir()
            launcher.write_text("synthetic launcher")
            native = root / "native/bin/airs-harness"
            native.parent.mkdir(parents=True)
            native.write_bytes(b"synthetic native")
            digest = hashlib.sha256(native.read_bytes()).hexdigest()
            info = native.parent.parent / "BUILD-INFO.json"
            info.write_text(
                json.dumps({"source_commit": "fixture-source", "binary_sha256": digest})
            )
            result = subprocess.CompletedProcess([], 0, str(native) + "\n", "")
            with patch(
                "airs_oidc_evidence.subprocess.run", return_value=result
            ) as resolve:
                actual = invocation_provenance(launcher)
            self.assertEqual(actual["binary_sha256"], digest)
            self.assertNotEqual(actual["invocation_sha256"], digest)
            self.assertEqual(actual["source_commit"], "fixture-source")
            self.assertTrue(resolve.call_args.args[0][-1].endswith("lib/launcher.js"))
            info.write_text(
                json.dumps(
                    {"source_commit": "fixture-source", "binary_sha256": "wrong"}
                )
            )
            with patch("airs_oidc_evidence.subprocess.run", return_value=result):
                with self.assertRaisesRegex(
                    AssertionError, "differs from its build provenance"
                ):
                    invocation_provenance(launcher)


if __name__ == "__main__":
    unittest.main()
