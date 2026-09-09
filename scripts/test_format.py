"""Formatter routing must preserve release evidence as JSON."""

import unittest
from unittest.mock import patch

import format as formatter


class FormatterRouting(unittest.TestCase):
    def test_buildifier_excludes_json_evidence_but_keeps_starlark_inputs(self):
        files = [
            "validation/run/BUILD.json",
            "validation/run/WORKSPACE.json",
            "BUILD",
            "module/BUILD.bazel",
            "MODULE.bazel",
            "tools/rules.bzl",
        ]
        with patch.object(
            formatter.subprocess, "check_output", return_value="\0".join(files).encode()
        ):
            group = formatter.buildifier_formatter_group(check=True)
        self.assertEqual(
            list(group.commands[0].args[4:]),
            ["BUILD", "MODULE.bazel", "module/BUILD.bazel", "tools/rules.bzl"],
        )


if __name__ == "__main__":
    unittest.main()
