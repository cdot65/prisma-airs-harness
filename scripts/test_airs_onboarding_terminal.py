"""Regression checks for capture fidelity; run with pyte==0.8.2 installed."""

import unittest

import pyte
from validate_airs_onboarding_preview import AlternateScreen


class AlternateScreenTests(unittest.TestCase):
    def test_fragmented_switch_restores_primary_after_resize(self):
        screen = AlternateScreen(80, 24)
        stream = pyte.ByteStream(screen)
        stream.feed(b"earlier warning\r\n")
        for byte in b"\x1b[?1049hbrand":
            stream.feed(bytes([byte]))
        self.assertTrue(screen.display[0].startswith("brand"))
        self.assertNotIn("earlier warning", "\n".join(screen.display))
        stream.feed(b"\x1b[?1049h")
        self.assertTrue(screen.display[0].startswith("brand"))
        screen.resize(columns=30, lines=10)
        stream.feed(b"\x1b[?1049l")
        self.assertEqual((screen.columns, screen.lines), (30, 10))
        self.assertTrue(screen.display[0].startswith("earlier warning"))
        stream.feed(b"\x1b[?1049hsecond")
        self.assertTrue(screen.display[0].startswith("second"))
        self.assertNotIn("brand", "\n".join(screen.display))


if __name__ == "__main__":
    unittest.main()
