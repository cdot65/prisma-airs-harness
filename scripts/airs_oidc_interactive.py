"""Keep a real OIDC terminal open across inference and MCP credential expiry."""

import json
import os
import time

from airs_harness_pty import TerminalSession


def verify_interactive_refresh(binary, env, work, home, expiry, output):
    def events():
        rows = []
        for path in (home / "sessions").rglob("*.jsonl"):
            for line in path.read_text().splitlines():
                try:
                    rows.append(json.loads(line).get("payload", {}))
                except json.JSONDecodeError:
                    continue
        return rows

    def completed():
        return [r for r in events() if r.get("type") == "task_complete"]

    def scans():
        result = []
        for event in events():
            item = event.get("item", {})
            if (
                event.get("type") == "item_completed"
                and item.get("type") == "McpToolCall"
                and item.get("tool") == "pan_inline_scan"
                and item.get("status") == "completed"
            ):
                scan = (
                    (item.get("result") or {})
                    .get("structuredContent", {})
                    .get("results", {})
                )
                if scan.get("action") == "allow" and scan.get("scan_id"):
                    result.append(
                        {k: scan.get(k) for k in ["action", "scan_id", "profile_name"]}
                    )
        return result

    observed = []
    with TerminalSession(binary, env, work) as terminal:
        try:
            terminal.wait_until(
                lambda: (
                    b"Yes, continue" in terminal.transcript
                    or b"permissions:" in terminal.transcript
                )
            )
            if b"Yes, continue" in terminal.transcript:
                time.sleep(0.35)
                os.write(terminal.master, b"\r")
                terminal.wait_for(b"permissions:")
            time.sleep(0.35)
            for label in [
                "before token expiry",
                "explicit route",
                "after token expiry",
            ]:
                if label == "explicit route":
                    terminal.choose_model("down", "@openai-terminal-auth/gpt-4.1")
                if label.startswith("after"):
                    terminal.choose_model("up", "airs-gateway-default")
                    terminal.wait_until(lambda: time.time() >= expiry + 5, timeout=150)
                before_turn, before_scan = len(completed()), len(scans())
                terminal.send_line(
                    "Call the executable mcp__security.pan_inline_scan tool now with "
                    'scan_request.profile "Prisma AIRS Terminal" and scan_request.response '
                    f'"Hello {label}". Use tools/call, not resource functions. '
                    "Report the actual scan action and scan_id."
                )
                terminal.wait_until(lambda: len(completed()) > before_turn, timeout=180)
                fresh = scans()[before_scan:]
                assert fresh and all(
                    s["profile_name"] == "Prisma AIRS Terminal" for s in fresh
                ), "Missing successful real MCP scan"
                observed.append({"phase": label, "scans": fresh})
                print("PASS interactive OIDC scan " + label, flush=True)
            return {
                "passed": True,
                "same_terminal_process": True,
                "continued_after_initial_token_expiry": True,
                "model_switch_passed": True,
                "model_sequence": [
                    "gateway default",
                    "@openai-terminal-auth/gpt-4.1",
                    "gateway default",
                ],
                "turns": observed,
            }
        finally:
            output.with_suffix(".private-interactive.log").write_bytes(
                terminal.transcript
            )
