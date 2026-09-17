#!/usr/bin/env python3
"""Exercise a real AIRS TUI model switch, local edits, MCP and self-description.

Uses isolated state and a disposable project. Calls real inference and scanning;
never changes gateway configuration or the owner's existing environment/history.
Self-description replies are retained for human review, not treated as proof of
model identity. Credential values are read only by the terminal's file helpers.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from airs_harness_pty import TerminalSession


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--gateway-url", required=True)
    parser.add_argument("--credential-file", type=Path, required=True)
    parser.add_argument("--mcp-url", required=True)
    parser.add_argument("--mcp-credential-file", type=Path, required=True)
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
    env.pop("AIRS_API_KEY", None)

    def cli(*arguments):
        result = subprocess.run(
            [str(binary), *arguments],
            env=env,
            cwd=work,
            capture_output=True,
            text=True,
            timeout=60,
        )
        if result.returncode:
            raise RuntimeError(result.stderr)

    cli(
        "env",
        "create",
        "work",
        "--gateway-url",
        args.gateway_url,
        "--model",
        args.explicit_model,
    )
    cli("login", "--credential-file", str(args.credential_file.resolve(strict=True)))
    cli(
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
    )
    subprocess.run(["git", "init", "-q", str(work)], check=True)
    (work / "calculator.py").write_text(
        "def add(left, right):\n    return left + right\n"
    )
    (work / "test_calculator.py").write_text(
        "import unittest\nfrom calculator import add\n"
        "class CalculatorTests(unittest.TestCase):\n"
        "    def test_add(self):\n        self.assertEqual(add(2, 3), 5)\n"
    )
    skill = work / ".agents/skills/review-calculator"
    skill.mkdir(parents=True)
    (skill / "SKILL.md").write_text(
        "---\nname: review-calculator\ndescription: Verify small calculator changes.\n---\n"
        "The project files calculator.py and test_calculator.py are in the current "
        "working directory, not in this skill directory. Read those two files. "
        "Preserve addition behavior and leave unrelated files and caches alone. "
        "Add requested functionality "
        "and regression tests. Run python3 -m unittest -v and correct failures.\n"
    )
    registry = json.loads((root / "state/environments.json").read_text())
    state = root / "state/environments" / registry["environments"]["work"]["id"]

    def events():
        rows = []
        for path in (state / "sessions").rglob("*.jsonl"):
            for line in path.read_text().splitlines():
                try:
                    rows.append(json.loads(line))
                except json.JSONDecodeError:
                    continue  # A writer may not have finished its last line.
        return rows

    def completed():
        return [
            r["payload"]
            for r in events()
            if r.get("type") == "event_msg"
            and r.get("payload", {}).get("type") == "task_complete"
        ]

    replies = []
    receipt = {
        "passed": False,
        "binary": str(binary),
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
    }
    with TerminalSession(binary, env, work) as terminal:
        try:
            terminal.start()
            terminal.send_line("/mcp verbose")
            terminal.wait_for(b"Credential helper")
            receipt["mcp_credential_helper_status_displayed"] = True

            def turn(prompt):
                count = len(completed())
                terminal.send_line(prompt)
                terminal.wait_until(lambda: len(completed()) > count, timeout=240)
                reply = completed()[-1]["last_agent_message"]
                replies.append({"prompt": prompt, "reply": reply})
                print(f"PASS completed turn: {prompt[:70]}", flush=True)
                return reply

            for prompt in (
                "what model are you?",
                "tell me about your codebase",
                "what mcp servers are you connected to?",
            ):
                turn(prompt)
            assert "security" in replies[2]["reply"], (
                "MCP answer omitted the configured server"
            )
            assert "pan_inline_scan" in replies[2]["reply"], (
                "MCP answer omitted the configured tool"
            )
            turn(
                "$review-calculator Add a multiply function and tests covering positive, "
                "negative, and zero inputs. Run the tests. Then use pan_inline_scan to "
                'scan "Hello from AIRS Harness". Report the actual test results, scan '
                "action, and scan ID."
            )
            terminal.choose_model("down", args.explicit_model)
            turn("Review the changes you just made and run the tests again.")
            turn(
                'Use pan_inline_scan to scan "Hello from the explicit route" and '
                "report the actual scan action and scan ID."
            )
            terminal.choose_model("up", "airs-gateway-default")
            turn(
                'Use pan_inline_scan to scan "Hello from AIRS Harness again" and '
                "report the actual scan action and scan ID."
            )
            validation = subprocess.run(
                ["python3", "-m", "unittest", "-v"],
                cwd=work,
                capture_output=True,
                text=True,
                timeout=30,
            )
            (root / "tests.log").write_text(validation.stdout + validation.stderr)
            assert validation.returncode == 0, "Independent calculator tests failed"
            validation = subprocess.run(
                [
                    "python3",
                    "-c",
                    "from calculator import add, multiply; "
                    "assert add(2,3)==5; assert multiply(2,3)==6; "
                    "assert multiply(-2,3)==-6; assert multiply(2,-3)==-6; "
                    "assert multiply(0,3)==0",
                ],
                cwd=work,
                capture_output=True,
                text=True,
                timeout=30,
            )
            assert validation.returncode == 0, validation.stderr
            scans = []
            for row in events():
                payload = row.get("payload", {})
                item = payload.get("item", {})
                if (
                    payload.get("type") == "item_completed"
                    and item.get("type") == "McpToolCall"
                    and item.get("server") == "security"
                    and item.get("tool") == "pan_inline_scan"
                ):
                    assert item.get("status") == "completed", item
                    scan = (
                        (item.get("result") or {})
                        .get("structuredContent", {})
                        .get("results", {})
                    )
                    assert scan.get("action") == "allow" and scan.get("scan_id"), item
                    scans.append(scan)
            assert len(scans) >= 3, "Missing completed MCP calls across route changes"
            receipt.update(
                passed=True,
                completed_turns=len(replies),
                local_tests_passed=True,
                independent_assertions_passed=True,
                mcp_calls=scans,
                default_explicit_default_switch_passed=True,
                self_description_replies_require_review=True,
            )
        finally:
            (root / "transcript.log").write_bytes(terminal.transcript)
            (root / "replies.json").write_text(json.dumps(replies, indent=2) + "\n")
            (root / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(f"PASS interactive E2E; receipts: {root}")


if __name__ == "__main__":
    main()
