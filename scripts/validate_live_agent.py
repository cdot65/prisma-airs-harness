#!/usr/bin/env python3
"""Exercise the installed local agent, skill, tests, AIRS MCP and history resume.

All writes stay in a new output directory. Inference and the configured read-only
scanner are real calls. Logs contain fixture content, never credential values.
"""

import argparse
import json
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--gateway-url", required=True)
    parser.add_argument("--credential-file", type=Path, required=True)
    parser.add_argument("--mcp-url", required=True)
    parser.add_argument("--mcp-credential-file", type=Path, required=True)
    parser.add_argument("--scan-profile", default="Prisma AIRS Terminal")
    parser.add_argument("--explicit-model", default="@openai/gpt-4.1")
    parser.add_argument("--output-directory", type=Path, required=True)
    args = parser.parse_args()
    os.umask(0o077)
    root = args.output_directory.resolve()
    root.mkdir(mode=0o700)
    work = root / "work"
    work.mkdir()
    binary = args.binary.resolve(strict=True)
    env = dict(os.environ, AIRS_HARNESS_HOME=str(root / "state"))

    def run(arguments, log):
        result = subprocess.run(
            [str(binary), *arguments],
            env=env,
            cwd=work,
            text=True,
            capture_output=True,
            timeout=240,
        )
        (root / log).write_text(result.stdout + "\nSTDERR\n" + result.stderr)
        if result.returncode:
            raise RuntimeError(
                f"Command failed ({result.returncode}); see {root / log}"
            )
        return result

    run(
        [
            "setup",
            "--environment",
            "work",
            "--gateway-url",
            args.gateway_url,
            "--model",
            args.explicit_model,
        ],
        "setup.log",
    )
    run(
        ["login", "--credential-file", str(args.credential_file.resolve(strict=True))],
        "login.log",
    )
    run(
        [
            "setup-mcp",
            "--name",
            "security",
            "--url",
            args.mcp_url,
            "--credential-file",
            str(args.mcp_credential_file.resolve(strict=True)),
            "--tool",
            "pan_inline_scan",
            "--required",
        ],
        "mcp-setup.log",
    )
    (work / "test_calc.py").write_text(
        "import unittest\nfrom calc import add\nclass AdditionTest(unittest.TestCase):\n"
        "    def test_addition(self):\n        self.assertEqual(add(2, 3), 5)\n"
        "        self.assertEqual(add(-2, 3), 1)\n"
    )
    skill = work / ".agents/skills/verify-addition"
    skill.mkdir(parents=True)
    (skill / "SKILL.md").write_text(
        "---\nname: verify-addition\ndescription: Diagnose and verify the addition fixture.\n---\n"
        "Read calc.py and test_calc.py. Make the smallest correction to addition. "
        "Run python3 -m unittest -v. After all tests pass, create skill-verification.txt "
        "containing exactly SKILL_VERIFIED. Report the actual test results.\n"
    )
    checks = []
    for mode, extra in [("default", []), ("explicit", ["-m", args.explicit_model])]:
        (work / "calc.py").write_text("def add(a, b):\n    return a - b\n")
        (work / "skill-verification.txt").unlink(missing_ok=True)
        prompt = (
            "$verify-addition Fix calc.py using the local verify-addition skill and run its tests. "
            "Then use the security pan_inline_scan MCP tool once to scan the benign text "
            "Hello from AIRS Harness, with scan_request.profile "
            + args.scan_profile
            + " and app_name prisma-airs-harness. Report the scan action and ID. "
            "The session-only marker is ORBIT-746; include it in your final reply "
            "but do not write it into any files."
        )
        result = run(
            [
                "exec",
                "--skip-git-repo-check",
                "-s",
                "workspace-write",
                "--json",
                *extra,
                prompt,
            ],
            mode + ".log",
        )
        events = [
            json.loads(line)
            for line in result.stdout.splitlines()
            if line.startswith("{")
        ]
        assert (work / "skill-verification.txt").read_text().strip() == "SKILL_VERIFIED"
        tests = subprocess.run(
            ["python3", "-m", "unittest", "-v"],
            cwd=work,
            capture_output=True,
            text=True,
            timeout=30,
        )
        (root / (mode + "-independent-tests.log")).write_text(
            tests.stdout + tests.stderr
        )
        assert tests.returncode == 0, "Independent verification failed"
        scans = [
            (event["item"].get("result") or {})
            .get("structured_content", {})
            .get("results", {})
            for event in events
            if event.get("item", {}).get("type") == "mcp_tool_call"
            and event["item"].get("server") == "security"
            and event["item"].get("tool") == "pan_inline_scan"
            and event["item"].get("status") == "completed"
            and event["item"].get("error") is None
        ]
        scan = next(
            (
                scan
                for scan in scans
                if scan.get("action") == "allow"
                and scan.get("profile_name") == args.scan_profile
                and scan.get("scan_id")
            ),
            None,
        )
        assert scan, "No successful scan result from the required profile"
        thread = next(
            event["thread_id"] for event in events if event["type"] == "thread.started"
        )
        checks.append(
            {
                "mode": mode,
                "thread_id": thread,
                "local_tests_passed": True,
                "skill_passed": True,
                "mcp_completed": True,
                "scan_id": scan["scan_id"],
            }
        )
        print(f"PASS {mode}: local skill, edit, tests and remote MCP", flush=True)
    resumed = run(
        [
            "exec",
            "resume",
            "--skip-git-repo-check",
            "--json",
            checks[0]["thread_id"],
            "What was the session-only marker from our first turn? Do not read files or execute any tool.",
        ],
        "resume.log",
    )
    assert "ORBIT-746" in resumed.stdout, "Session marker was not retained"
    receipt = {
        "schema_version": 1,
        "binary": str(binary),
        "checks": checks,
        "resume_passed": True,
        "passed": True,
    }
    (root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(f"PASS resume; receipt: {root / 'receipt.json'}")


if __name__ == "__main__":
    main()
