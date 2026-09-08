"""Keep a real OIDC terminal open across inference and MCP credential expiry."""

import json
import os
import tempfile
import time

from airs_harness_pty import TerminalSession

MAX_PRIVATE_TRANSCRIPT_BYTES = 4 * 1024 * 1024


def write_private_transcript(path, transcript):
    """Atomically retain a bounded diagnostic tail, private from its first byte."""
    descriptor, temporary = tempfile.mkstemp(
        prefix=".private-interactive-", dir=path.parent
    )
    try:
        with os.fdopen(descriptor, "wb") as target:
            target.write(transcript[-MAX_PRIVATE_TRANSCRIPT_BYTES:])
            target.flush()
            os.fsync(target.fileno())
        os.replace(temporary, path)
    finally:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass


def verify_interactive_refresh(binary, env, work, home, observe_expiries, output):
    initial_sessions = set((home / "sessions").rglob("*.jsonl"))

    def events():
        rows = []
        for path in (home / "sessions").rglob("*.jsonl"):
            if path in initial_sessions:
                continue
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
    expiry_evidence = []
    current_expiries = None
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
                "after expiry cycle 1",
                "after expiry cycle 2",
            ]:
                if label == "explicit route":
                    terminal.choose_model("down", "@openai-terminal-auth/gpt-4.1")
                if label.startswith("after"):
                    if not expiry_evidence:
                        terminal.choose_model("up", "airs-gateway-default")
                    current_expiries = observe_expiries()
                    if expiry_evidence:
                        assert all(
                            current_expiries[name]
                            > expiry_evidence[-1]["resource_expiries"][name]
                            for name in ["inference", "mcp"]
                        ), "Both resource tokens must have renewed"
                    boundary = max(current_expiries.values()) + 5
                    # No credential helper runs after this snapshot until the
                    # post-expiry terminal task has completed successfully.
                    terminal.wait_until(
                        lambda: time.time() >= boundary,
                        timeout=max(1, boundary - time.time()) + 15,
                    )
                before_turn, before_scan = len(completed()), len(scans())
                before_events = len(events())
                local_prompt = ""
                cycle = len(expiry_evidence) + 1
                if label.startswith("after"):
                    filename = f"oidc-expiry-{cycle}.txt"
                    expected = f"OIDC_EXPIRY_{cycle}_LOCAL_OK"
                    assert not (work / filename).exists(), "Proof file already exists"
                    local_prompt = (
                        f"First use one shell command to write {filename} containing "
                        f"exactly {expected}, then read it back with one shell command. "
                        "Use at most four shell commands. "
                    )
                submitted_at = time.time()
                terminal.send_line(
                    local_prompt
                    + "Call the executable mcp__security.pan_inline_scan tool now with "
                    'scan_request.profile "Prisma AIRS Terminal" and scan_request.response '
                    f'"Hello {label}". Use tools/call, not resource functions. '
                    "Report the actual scan action and scan_id."
                )
                terminal.wait_until(lambda: len(completed()) > before_turn, timeout=180)
                fresh = scans()[before_scan:]
                assert fresh and all(
                    s["profile_name"] == "Prisma AIRS Terminal" for s in fresh
                ), "Missing successful real MCP scan"
                assert (
                    len(set((home / "sessions").rglob("*.jsonl")) - initial_sessions)
                    == 1
                ), "Expected one continuous terminal session"
                observed.append({"phase": label, "scans": fresh})
                if label.startswith("after"):
                    local_commands = [
                        row["item"]
                        for row in events()[before_events:]
                        if row.get("type") == "item_completed"
                        and row.get("item", {}).get("type") == "CommandExecution"
                    ]
                    assert (
                        1 <= len(local_commands) <= 4
                        and all(item.get("exit_code") == 0 for item in local_commands)
                        and (work / filename).read_text().strip() == expected
                    ), "Missing successful bounded local file tool execution"
                    expiry_evidence.append(
                        {
                            "cycle": cycle,
                            "resource_expiries": current_expiries,
                            "turn_submitted_at": submitted_at,
                            "after_both_expiries": all(
                                submitted_at > value
                                for value in current_expiries.values()
                            ),
                            "local_file": filename,
                            "local_file_verified": True,
                            "successful_local_commands": len(local_commands),
                            "scans": fresh,
                        }
                    )
                print("PASS interactive OIDC scan " + label, flush=True)
            return {
                "passed": True,
                "same_terminal_process": True,
                "continued_after_initial_token_expiry": True,
                "model_switch_passed": True,
                "completed_expiry_cycles": len(expiry_evidence),
                "expiry_cycles": expiry_evidence,
                "expiry_observation": "Private inference and MCP credential helper JWT exp; helpers may rotate near-expiry records before each wait, never during the wait or following task",
                "model_sequence": [
                    "gateway default",
                    "@openai-terminal-auth/gpt-4.1",
                    "gateway default",
                    "gateway default",
                ],
                "turns": observed,
            }
        finally:
            write_private_transcript(
                output.with_suffix(".private-interactive.log"), terminal.transcript
            )
