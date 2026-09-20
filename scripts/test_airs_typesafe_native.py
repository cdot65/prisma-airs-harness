"""Installed TypeSafe key lifecycle using disposable native credential storage.

Run with AIRS_TYPESAFE_NATIVE=1 and AIRS_HARNESS_BIN pointing to the candidate.
No TypeSafe account, external traffic, or production credentials are used.
"""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from airs_native_test_store import native_store, native_test_command


def exercise(binary):
    with tempfile.TemporaryDirectory(prefix="airs-typesafe-native-") as directory:
        root = Path(directory)
        with native_store(root / "store", os.environ) as inherited:
            env = {k: v for k, v in inherited.items() if not k.startswith("TYPESAFE_")}
            env["AIRS_HARNESS_HOME"] = str(root / "harness")

            def run(*args, input_text=None, extra=None):
                result = subprocess.run(
                    [binary, *args],
                    input=input_text,
                    text=True,
                    capture_output=True,
                    env={**env, **(extra or {})},
                    timeout=30,
                )
                if result.returncode:
                    raise AssertionError("Installed TypeSafe fixture command failed")
                return result.stdout

            for name in ("judge", "other"):
                run(
                    "env", "create", name, "--gateway-url", "https://fixture.invalid/v1"
                )
            prefix = ("--environment", "judge", "env", "typesafe")
            try:
                run(
                    *prefix,
                    "set",
                    "--stdin",
                    "--model",
                    "jev-latest",
                    input_text="synthetic-typesafe-native-one\n",
                )
                run(
                    *prefix,
                    "exec",
                    "--",
                    sys.executable,
                    "-c",
                    "import os; assert os.environ['TYPESAFE_API_KEY'].endswith('-one'); "
                    "assert os.environ['TYPESAFE_DEFAULT_MODEL'] == 'jev-latest'",
                )
                run(
                    "--environment",
                    "other",
                    "env",
                    "typesafe",
                    "exec",
                    "--",
                    sys.executable,
                    "-c",
                    "import os; assert 'TYPESAFE_API_KEY' not in os.environ",
                )
                run(
                    *prefix,
                    "set",
                    "--stdin",
                    input_text="synthetic-typesafe-native-two\n",
                )
                run(
                    *prefix,
                    "exec",
                    "--",
                    sys.executable,
                    "-c",
                    "import os; assert os.environ['TYPESAFE_API_KEY'].endswith('-two')",
                )
                run(
                    *prefix,
                    "exec",
                    "--",
                    sys.executable,
                    "-c",
                    "import os; assert os.environ['TYPESAFE_API_KEY'] == 'synthetic-override'",
                    extra={"TYPESAFE_API_KEY": "synthetic-override"},
                )
                for settings in (root / "harness").rglob("*.json"):
                    if b"synthetic-typesafe-native" in settings.read_bytes():
                        raise AssertionError("Credential leaked into settings")
            finally:
                run(*prefix, "clear")
            run(
                *prefix,
                "exec",
                "--",
                sys.executable,
                "-c",
                "import os; assert 'TYPESAFE_API_KEY' not in os.environ",
            )
            if list((root / "harness").rglob("typesafe-cleanup.json")):
                raise AssertionError("Credential cleanup did not complete")
    print(
        json.dumps(
            {
                "passed": True,
                "native_storage": True,
                "environment_isolation": True,
                "real_jev_call": False,
            }
        )
    )


@unittest.skipUnless(
    os.environ.get("AIRS_TYPESAFE_NATIVE") == "1", "explicit native fixture"
)
class NativeTypeSafe(unittest.TestCase):
    def test_key_rotation_child_scope_isolation_and_cleanup(self):
        binary = str(Path(os.environ["AIRS_HARNESS_BIN"]).resolve(strict=True))
        result = subprocess.run(
            native_test_command(__file__, ["--worker", binary]),
            capture_output=True,
            text=True,
            timeout=180,
        )
        self.assertEqual(
            result.returncode, 0, "Disposable TypeSafe native fixture failed"
        )
        self.assertTrue(json.loads(result.stdout.splitlines()[-1])["passed"])


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--worker":
        exercise(sys.argv[2])
    else:
        unittest.main()
