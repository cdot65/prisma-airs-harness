"""Lifecycle provenance must fail on replaced binaries or changed helper inputs."""

from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import validate_airs_lifecycle as validator


class LifecycleProvenance(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.binary = self.root / "airs"
        self.binary.write_bytes(b"\x7fELFsynthetic-native-content")
        self.original = validator.native_identity(self.binary, self.binary)

    def test_unrelated_native_argument_cannot_name_launcher_bytes(self):
        launcher = self.root / "airs.js"
        launcher.write_text("#!/usr/bin/env node\n")
        with self.assertRaises(ValueError):
            validator.native_identity(launcher, self.binary)
        with self.assertRaises(ValueError):
            validator.native_identity(launcher, launcher)

    def test_replaced_native_bytes_fail_before_success_receipt(self):
        self.binary.write_bytes(b"\x7fELFreplacement-native-content")
        with patch.object(
            validator, "tooling_identity", return_value={"helper.py": "original"}
        ):
            with self.assertRaises(ValueError):
                validator.verify_identity(
                    self.binary, self.binary, self.original, {"helper.py": "original"}
                )

    def test_changed_transitive_helper_or_key_fails(self):
        for name in (
            "scripts/test_airs_mcp_manager.py",
            "codex-rs/airs-identity/src/fixtures/test-only-private.pem",
        ):
            with self.subTest(name=name):
                with patch.object(
                    validator, "tooling_identity", return_value={name: "changed"}
                ):
                    with self.assertRaises(ValueError):
                        validator.verify_identity(
                            self.binary, self.binary, self.original, {name: "initial"}
                        )

    def test_unchanged_identity_is_accepted(self):
        with patch.object(
            validator, "tooling_identity", return_value={"helper.py": "unchanged"}
        ):
            validator.verify_identity(
                self.binary, self.binary, self.original, {"helper.py": "unchanged"}
            )


if __name__ == "__main__":
    unittest.main()
