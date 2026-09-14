"""Acceptance-oracle checks for narrowly permitted helper path migration."""

from pathlib import Path
import tempfile
import unittest

from validate_airs_upgrade import assert_preserved_state


class UpgradeOracle(unittest.TestCase):
    def test_bounded_timeout_migration_is_explicit_and_rejects_other_changes(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "config.toml"
            original = (
                b'[model_providers.airs.auth]\ncommand = "/old"\ntimeout_ms = 60000\n'
            )
            protected = {path: original}
            migrated = original.replace(b"/old", b"/new").replace(b"60000", b"30000")
            path.write_bytes(migrated)
            with self.assertRaisesRegex(AssertionError, "Unexpected configuration"):
                assert_preserved_state(protected, path, "/new", {})
            assert_preserved_state(
                protected, path, "/new", {}, bounded_helper_timeout=True
            )
            for changed in [
                migrated.replace(b"30000", b"1"),
                migrated + b"extra = true\n",
            ]:
                path.write_bytes(changed)
                with self.assertRaisesRegex(AssertionError, "Unexpected configuration"):
                    assert_preserved_state(
                        protected, path, "/new", {}, bounded_helper_timeout=True
                    )
            protected[path] = original.replace(b"60000", b"45000")
            with self.assertRaisesRegex(AssertionError, "managed helper timeout"):
                assert_preserved_state(
                    protected, path, "/new", {}, bounded_helper_timeout=True
                )

    def test_owned_helper_path_only_and_append_only_history_are_accepted(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            config, binding, history = (
                root / "config.toml",
                root / "binding.json",
                root / "history.jsonl",
            )
            original = b"""model = "airs-gateway-default"
[model_providers.airs]
base_url = "https://gateway.invalid/v1"
[model_providers.airs.auth]
command = "/old/airs-harness"
args = ["credential", "--home", "/state", "--binding", "fixture-id"]
cwd = "/state"
timeout_ms = 60000
"""
            binding.write_bytes(b'{"stable":"identity"}')
            history.write_bytes(b'{"original":"history"}\n')
            protected = {config: original, binding: binding.read_bytes()}
            rollouts = {history: history.read_bytes()}
            migrated = original.replace(b"/old/airs-harness", b"/new/airs-harness")
            config.write_bytes(migrated + b"\n# serialization-only change\n")
            history.write_bytes(rollouts[history] + b'{"new":"turn"}\n')
            assert_preserved_state(protected, config, "/new/airs-harness", rollouts)
            mutations = [
                (b'"airs-gateway-default"', b'"@unexpected/model"'),
                (b"timeout_ms = 60000", b"timeout_ms = 60000\nunexpected = true"),
                (b"https://gateway.invalid/v1", b"https://other.invalid/v1"),
                (b'"--binding", "fixture-id"', b'"--binding", "changed-id"'),
                (b'cwd = "/state"', b'cwd = "/elsewhere"'),
                (b"timeout_ms = 60000", b"timeout_ms = 1"),
                (b"/new/airs-harness", b"/unexpected/airs-harness"),
            ]
            for before, after in mutations:
                with self.subTest(change=before):
                    config.write_bytes(migrated.replace(before, after))
                    with self.assertRaisesRegex(
                        AssertionError, "Unexpected configuration change"
                    ):
                        assert_preserved_state(
                            protected, config, "/new/airs-harness", rollouts
                        )
            config.write_bytes(migrated)
            binding.write_bytes(b'{"stable":"changed"}')
            with self.assertRaisesRegex(AssertionError, "binding changed"):
                assert_preserved_state(protected, config, "/new/airs-harness", rollouts)
            binding.write_bytes(protected[binding])
            history.write_bytes(b'{"original":"rewritten"}\n')
            with self.assertRaisesRegex(AssertionError, "history was rewritten"):
                assert_preserved_state(protected, config, "/new/airs-harness", rollouts)


if __name__ == "__main__":
    unittest.main()
