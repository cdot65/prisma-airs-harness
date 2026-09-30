"""Guard the build/acceptance boundary without running a native compiler."""

from pathlib import Path
import re
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]


class MacWorkflowBoundary(unittest.TestCase):
    def test_forgejo_preview_checks_auth_early_and_publishes_staged_packages(self):
        text = (ROOT / ".forgejo/workflows/airs-harness-macos-build.yml").read_text()
        build = text.split("\n  build:\n", 1)[1].split("\n  acceptance:\n", 1)[0]
        self.assertLess(
            build.index("npm whoami"), build.index("name: Build native agent")
        )
        publish = text.split("name: Publish the accepted Mac candidate", 1)[1]
        self.assertLess(
            publish.index("stage_airs_macos_preview.py"), publish.index("npm publish")
        )
        self.assertIn('tarballs="$RUNNER_TEMP/airs-preview/tarballs"', publish)
        self.assertNotIn('tarballs="$RUNNER_TEMP/airs-npm/tarballs"', publish)
        self.assertIn('--evidence "$RUNNER_TEMP/airs-evidence"', publish)
        self.assertIn('= "$native_latest_before"', publish)

    def test_acceptance_cannot_compile_or_promote_and_pipelines_fail_closed(self):
        for name in ["release", "revalidate"]:
            text = (
                ROOT / f".github/workflows/airs-harness-macos-{name}.yml"
            ).read_text()
            with self.subTest(workflow=name):
                # GitHub's explicit bash shell adds -e -o pipefail; unspecified
                # shell defaults must not hide a compiler failure behind tee.
                self.assertRegex(text, r"(?m)^defaults:\n  run:\n    shell: bash$")
                self.assertEqual(re.findall(r"(?m)^\s+shell:\s*(.+)$", text), ["bash"])
                acceptance = text.split("\n  acceptance:\n", 1)[1]
                self.assertNotRegex(
                    acceptance,
                    r"(?m)^\s+(?:run: )?cargo\b[^\n]*\b(?:build|test|nextest)\b",
                )
                self.assertIn("artifact-ids:", acceptance)
                self.assertIn("--unvalidated-candidate", acceptance)
                self.assertNotIn("--validation ", acceptance)
                if name == "release":
                    build = text.split("\n  build:\n", 1)[1].split(
                        "\n  acceptance:\n", 1
                    )[0]
                    self.assertIn("if: ${{ !inputs.preflight_only }}", build)
                    self.assertIn("needs: build", acceptance)
                    self.assertLess(
                        build.index("id: executables"),
                        build.index("id: save_build_cache"),
                    )
                    # Downloads can survive a later compiler failure. Compiled
                    # outputs still follow immutable executable preservation.
                    self.assertLess(
                        build.index("run: cargo fetch --locked"),
                        build.index("id: save_source_cache"),
                    )
                    self.assertLess(
                        build.index("id: save_source_cache"),
                        build.index("name: Build native agent"),
                    )
                    self.assertEqual(build.count("continue-on-error: true"), 2)
                else:
                    self.assertRegex(
                        text,
                        r"(?m)^        options:\n          - macos-15\n          - macos-26$",
                    )
                    self.assertIn("default: macos-15", text)
                    self.assertIn("runs-on: ${{ inputs.runner }}", acceptance)
                    self.assertIn('test "$(uname -m)" = arm64', acceptance)
                    self.assertIn('test "$RUNNER_ARCH" = ARM64', acceptance)
                    self.assertIn("acceptance-host.json", acceptance)
        # Exercise the shell failure behavior relied on by the workflow.
        result = subprocess.run(
            [
                "bash",
                "-e",
                "-o",
                "pipefail",
                "-c",
                "false | cat; printf unsafe-continuation",
            ],
            capture_output=True,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b"")


if __name__ == "__main__":
    unittest.main()
