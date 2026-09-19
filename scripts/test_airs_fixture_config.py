"""Fixture overrides preserve settings and reject unsafe or ambiguous rewrites."""

from pathlib import Path
import tempfile
import tomllib
import unittest

from airs_fixture_config import set_mcp_store


class FixtureStoreOverride(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.path = Path(temporary.name) / "config.toml"

    def test_omitted_policy_preserves_tables_and_comments(self):
        original = '# fixture\nmodel = "test"\n[mcp_servers.one]\nurl = "https://fixture/mcp"\n'
        self.path.write_text(original)
        set_mcp_store(self.path, "keyring")
        self.assertEqual(
            self.path.read_text(),
            'mcp_oauth_credentials_store = "keyring"\n' + original,
        )

    def test_existing_supported_policies_replace_only_value(self):
        for prior in ("auto", "file", "keyring"):
            for key in (
                "mcp_oauth_credentials_store",
                '"mcp_oauth_credentials_store"',
                "'mcp_oauth_credentials_store'",
            ):
                with self.subTest(prior=prior, key=key):
                    original = f"# before\n{key} = '{prior}' # retained\n[other]\nmode = 'file'\n"
                    self.path.write_text(original)
                    set_mcp_store(self.path, "keyring")
                    self.assertEqual(
                        self.path.read_text(),
                        original.replace(f"= '{prior}' #", '= "keyring" #', 1),
                    )

    def test_nested_same_name_does_not_replace_nested_value(self):
        original = '[other]\nmcp_oauth_credentials_store = "file"\n'
        self.path.write_text(original)
        set_mcp_store(self.path, "keyring")
        self.assertEqual(
            tomllib.loads(self.path.read_text()),
            {
                "mcp_oauth_credentials_store": "keyring",
                "other": {"mcp_oauth_credentials_store": "file"},
            },
        )

    def test_invalid_duplicate_and_multiline_assignments_are_unchanged(self):
        cases = [
            'mcp_oauth_credentials_store="file"\nmcp_oauth_credentials_store="auto"\n',
            "mcp_oauth_credentials_store = 42\n",
            'mcp_oauth_credentials_store = "unknown"\n',
            'note = """\nmcp_oauth_credentials_store = "file"\n"""\n',
        ]
        for original in cases:
            with self.subTest(original=original):
                self.path.write_text(original)
                with self.assertRaises(ValueError):
                    set_mcp_store(self.path, "keyring")
                self.assertEqual(self.path.read_text(), original)

    def test_invalid_requested_mode_never_changes_file(self):
        self.path.write_text("# preserved\n")
        with self.assertRaises(ValueError):
            set_mcp_store(self.path, "unknown")
        self.assertEqual(self.path.read_text(), "# preserved\n")
