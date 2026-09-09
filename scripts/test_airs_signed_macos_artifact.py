"""Signed intake rejection and trust-gate checks; real Apple checks run on macOS."""

import hashlib
from pathlib import Path
import stat
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import airs_signed_macos_artifact as signed


DETAILS = "CodeDirectory v=20500 flags=0x10000(runtime)\nAuthority=Developer ID Application: Example (G5QLZ5A8TA)\nTeamIdentifier=G5QLZ5A8TA\nTimestamp=Sep 9, 2026\n"


class SignedIntakeTests(unittest.TestCase):
    def archive(self, root, entries):
        path = root / "intake.zip"
        with zipfile.ZipFile(path, "w") as archive:
            for name, content, mode in entries:
                info = zipfile.ZipInfo(name)
                info.external_attr = mode << 16
                archive.writestr(info, content)
        return path

    def test_exact_nested_binary_preserved_sidecar_not_installed(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            payload = b"signed candidate exact bytes"
            archive = self.archive(
                root,
                [
                    (signed.MEMBER, payload, stat.S_IFREG | 0o700),
                    (signed.SIDECAR, b"metadata", stat.S_IFREG | 0o600),
                ],
            )
            binary = signed.restore(
                archive,
                root / "out",
                signed.digest(archive),
                hashlib.sha256(payload).hexdigest(),
            )
            self.assertEqual(binary.read_bytes(), payload)
            self.assertEqual(
                [p.name for p in binary.parent.iterdir()], ["airs-harness"]
            )
            with self.assertRaises(FileExistsError):
                signed.restore(
                    archive, root / "out", signed.digest(archive), signed.digest(binary)
                )

    def test_hash_archive_layout_and_link_rejections_precede_execution(self):
        base = [
            (signed.MEMBER, b"binary", stat.S_IFREG | 0o700),
            (signed.SIDECAR, b"metadata", stat.S_IFREG | 0o600),
        ]
        variants = [
            base + [("../escape", b"x", stat.S_IFREG)],
            base + [base[0]],
            [(signed.MEMBER, b"target", stat.S_IFLNK)] + base[1:],
            [(signed.MEMBER, b"", stat.S_IFREG)] + base[1:],
            base + [("extra", b"x", stat.S_IFREG)],
        ]
        for entries in variants:
            with (
                self.subTest(entries=entries),
                tempfile.TemporaryDirectory() as temporary,
            ):
                root = Path(temporary)
                archive = self.archive(root, entries)
                with self.assertRaises(ValueError):
                    signed.restore(
                        archive, root / "out", signed.digest(archive), "0" * 64
                    )
                self.assertFalse((root / "out").exists())
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = self.archive(root, base)
            with self.assertRaisesRegex(ValueError, "archive hash"):
                signed.restore(archive, root / "wrong-archive", "0" * 64, "0" * 64)
            self.assertFalse((root / "wrong-archive").exists())
            with self.assertRaisesRegex(ValueError, "executable hash"):
                signed.restore(
                    archive, root / "wrong-native", signed.digest(archive), "0" * 64
                )
            self.assertFalse((root / "wrong-native/airs-harness").exists())

    def test_wrong_team_adhoc_and_missing_runtime_fail(self):
        signed.check_details(DETAILS)
        for details in [
            DETAILS.replace("G5QLZ5A8TA", "OTHERTTEAM"),
            DETAILS.replace("0x10000(runtime)", "0x0(none)"),
            DETAILS.replace(
                "Authority=Developer ID Application: Example (G5QLZ5A8TA)",
                "Signature=adhoc",
            ),
            DETAILS.replace("Timestamp=Sep 9, 2026", ""),
        ]:
            with self.subTest(details=details), self.assertRaises(ValueError):
                signed.check_details(details)

    def test_no_apple_tool_runs_before_native_digest_matches(self):
        with (
            tempfile.TemporaryDirectory() as temporary,
            patch.object(signed.subprocess, "run") as run,
        ):
            root = Path(temporary)
            binary = root / "airs-harness"
            binary.write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "hash changed"):
                signed.verify(
                    binary,
                    "0" * 64,
                    root / "SIGNING.json",
                    "a" * 40,
                    "1",
                    "b" * 64,
                    "reported",
                )
            run.assert_not_called()
            self.assertFalse((root / "SIGNING.json").exists())

    def test_notarization_exit_controls_receipt_even_without_success_prose(self):
        for status in [0, 1]:
            with (
                self.subTest(status=status),
                tempfile.TemporaryDirectory() as temporary,
            ):
                root = Path(temporary)
                binary = root / "airs-harness"
                binary.write_bytes(b"exact")
                commands = []

                def native(command, **kwargs):
                    commands.append(command)
                    output = (
                        "arm64\n"
                        if Path(command[0]).name == "lipo"
                        else DETAILS
                        if "-d" in command
                        else ""
                    )
                    return signed.subprocess.CompletedProcess(
                        command, status if "=notarized" in command else 0, output, ""
                    )

                with patch.object(signed.subprocess, "run", side_effect=native):
                    if status:
                        with self.assertRaisesRegex(ValueError, "notarization"):
                            signed.verify(
                                binary,
                                signed.digest(binary),
                                root / "SIGNING.json",
                                "a" * 40,
                                "1",
                                "b" * 64,
                                "reported",
                            )
                    else:
                        signed.verify(
                            binary,
                            signed.digest(binary),
                            root / "SIGNING.json",
                            "a" * 40,
                            "1",
                            "b" * 64,
                            "reported",
                        )
                self.assertEqual((root / "SIGNING.json").exists(), status == 0)
                self.assertEqual(
                    commands[-1],
                    [
                        "/usr/bin/codesign",
                        "--verify",
                        "--strict",
                        "--verbose=4",
                        "--check-notarization",
                        "-R",
                        "=notarized",
                        str(binary),
                    ],
                )
                self.assertTrue((root / "SIGNING-notarization.log").exists())
                if status == 0:
                    import json

                    receipt = json.loads((root / "SIGNING.json").read_text())
                    self.assertTrue(receipt["notarization_verified"])
                    self.assertEqual(
                        receipt["notarization_method"],
                        "codesign-online-explicit-notarized-requirement",
                    )
                    self.assertEqual(
                        receipt["gatekeeper_app_assessment"], "not-applicable-raw-cli"
                    )


if __name__ == "__main__":
    unittest.main()
