import stat
import tempfile
import unittest
from pathlib import Path

from airs_oidc_interactive import MAX_PRIVATE_TRANSCRIPT_BYTES
from airs_oidc_interactive import write_private_transcript


class PrivateTranscript(unittest.TestCase):
    def test_replacement_is_private_and_bounded(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "private.log"
            path.write_bytes(b"old diagnostic")
            path.chmod(0o644)
            transcript = b"discarded prefix" + b"x" * MAX_PRIVATE_TRANSCRIPT_BYTES
            write_private_transcript(path, transcript)
            self.assertEqual(path.read_bytes(), b"x" * MAX_PRIVATE_TRANSCRIPT_BYTES)
            self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600)
            self.assertEqual(list(path.parent.iterdir()), [path])

    def test_output_symlink_cannot_redirect_private_data(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "unrelated.log"
            target.write_bytes(b"keep this file")
            path = Path(directory) / "private.log"
            path.symlink_to(target)
            write_private_transcript(path, b"synthetic private diagnostic")
            self.assertFalse(path.is_symlink())
            self.assertEqual(target.read_bytes(), b"keep this file")
            self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600)


if __name__ == "__main__":
    unittest.main()
