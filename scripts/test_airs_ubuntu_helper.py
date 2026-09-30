"""The shipped Ubuntu helper must install, check and fetch the launcher it ships with."""

from pathlib import Path
import re
import unittest

from airs_test_release_spec import LAUNCHER
from airs_ubuntu_helper import verify_ubuntu_helper

SCRIPTS = Path(__file__).resolve().parent
SOURCE = (SCRIPTS / "prepare_airs_ubuntu.sh").read_text()
VERSION = re.search(r"^version=\$\{AIRS_TEST_VERSION:-(\S+)\}$", SOURCE, re.M)[1]


def helper_text(launcher=LAUNCHER, version=VERSION):
    return SOURCE.replace(f"package={LAUNCHER}\n", f"package={launcher}\n").replace(
        f"version=${{AIRS_TEST_VERSION:-{VERSION}}}",
        f"version=${{AIRS_TEST_VERSION:-{version}}}",
    )


class UbuntuHelperContract(unittest.TestCase):
    def test_repository_helper_names_the_shipped_launcher_once(self):
        report = verify_ubuntu_helper(SOURCE, LAUNCHER, VERSION)
        self.assertEqual(report["launcher"], LAUNCHER)
        verify_ubuntu_helper(SOURCE.encode(), LAUNCHER, VERSION)

    def test_renamed_launcher_and_version_are_checked_exactly(self):
        with self.assertRaises(ValueError):
            verify_ubuntu_helper(
                helper_text(launcher="prisma-airs-harness"), LAUNCHER, VERSION
            )
        with self.assertRaises(ValueError):
            verify_ubuntu_helper(SOURCE, "prisma-airs-harness", VERSION)
        with self.assertRaises(ValueError):
            verify_ubuntu_helper(helper_text(version="0.1.4"), LAUNCHER, "0.1.5")

    def test_paths_outside_the_declaration_are_rejected(self):
        # The 0.1.4 defect: an install line for the new name while a derived path
        # still names the previous launcher directory.
        stale = SOURCE.replace(
            '"$prefix/lib/node_modules/$package/package.json"',
            '"$prefix/lib/node_modules/prisma-airs-harness/package.json"',
        )
        with self.assertRaises(ValueError):
            verify_ubuntu_helper(stale, LAUNCHER, VERSION)
        literal = SOURCE.replace(
            '"$registry/$package_url/$version"',
            f'"$registry/{LAUNCHER.replace("/", "%2f")}/$version"',
        )
        with self.assertRaises(ValueError):
            verify_ubuntu_helper(literal, LAUNCHER, VERSION)
        twice = SOURCE.replace(f"package={LAUNCHER}\n", f"package={LAUNCHER}\n" * 2)
        with self.assertRaises(ValueError):
            verify_ubuntu_helper(twice, LAUNCHER, VERSION)
        with self.assertRaises(ValueError):
            verify_ubuntu_helper(SOURCE + "x" * 70000, LAUNCHER, VERSION)


if __name__ == "__main__":
    unittest.main()
