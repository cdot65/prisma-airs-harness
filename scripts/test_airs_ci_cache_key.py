"""Check cache invalidation without invoking a compiler or network client."""

from pathlib import Path
import subprocess
import tempfile
import unittest

from airs_ci_cache_key import identity


class CacheIdentityTests(unittest.TestCase):
    def test_sdk_compiler_profile_and_lock_changes_invalidate(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in (
                "Cargo.lock",
                "Cargo.toml",
                "rust-toolchain.toml",
                ".cargo/config.toml",
            ):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(name)
            environment = {
                "RUNNER_OS": "macOS",
                "RUNNER_ARCH": "ARM64",
                "AIRS_CLI_OPT_LEVEL": "1",
            }
            outputs = {
                ("rustc", "-vV"): "rustc 1.95 host aarch64-apple-darwin",
                ("sccache", "--version"): "sccache 0.16.0",
                ("xcrun", "--sdk", "macosx", "--show-sdk-version"): "15.5",
                ("xcodebuild", "-version"): "Xcode 16.4",
            }

            def current(env=environment):
                return identity(root, env, lambda command: outputs[tuple(command)])

            baseline = current()
            # Source/tooling commit IDs belong in separate provenance, not in
            # compiler identity. Unrelated secret-bearing environment is omitted.
            changed = current(
                {
                    **environment,
                    "GITHUB_SHA": "new-tooling",
                    "AIRS_RUNTIME_SOURCE": "new-runtime",
                    "TOKEN": "secret-canary",
                }
            )
            self.assertEqual(changed, baseline)
            self.assertNotIn("secret-canary", str(changed))
            for key, value in {
                "RUNNER_ARCH": "X64",
                "AIRS_CLI_OPT_LEVEL": "3",
                "CARGO_PROFILE_RELEASE_LTO": "thin",
                "RUSTFLAGS": "-C debuginfo=2",
            }.items():
                with self.subTest(environment=key):
                    self.assertNotEqual(
                        current({**environment, key: value})["cache_key"],
                        baseline["cache_key"],
                    )
            for command, value in list(outputs.items()):
                with self.subTest(command=command):
                    outputs[command] = value + " changed"
                    self.assertNotEqual(current()["cache_key"], baseline["cache_key"])
                    outputs[command] = value
            for name in baseline["inputs"]["files"]:
                with self.subTest(file=name):
                    (root / name).write_text("changed")
                    self.assertNotEqual(current()["cache_key"], baseline["cache_key"])
                    (root / name).write_text(name)
            self.assertEqual(current(), baseline)

    def test_command_line_tools_without_full_xcode(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for name in ("Cargo.lock", "Cargo.toml", "rust-toolchain.toml", ".cargo/config.toml"):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(name)
            version = "version: 26.0"

            def output(command):
                if command[0] == "xcodebuild":
                    raise subprocess.CalledProcessError(1, command)
                if command[0] == "pkgutil":
                    return version
                return " ".join(command)

            baseline = identity(root, {"RUNNER_OS": "macOS"}, output)
            self.assertEqual(baseline["inputs"]["command_line_tools"], version)
            self.assertNotIn("xcode", baseline["inputs"])
            version = "version: 26.1"
            self.assertNotEqual(
                identity(root, {"RUNNER_OS": "macOS"}, output)["cache_key"],
                baseline["cache_key"],
            )


if __name__ == "__main__":
    unittest.main()
