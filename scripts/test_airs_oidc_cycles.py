import json
import re
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from airs_oidc_interactive import verify_interactive_refresh


class TwoExpiryCycles(unittest.TestCase):
    def exercise(self, *, renewed=True, local_tool=True, mcp=True):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            home = root / "home"
            sessions = home / "sessions"
            sessions.mkdir(parents=True)
            (sessions / "previous.jsonl").write_text(
                '{"payload":{"type":"task_complete"}}\n'
            )
            clock = [100]
            actions = []
            snapshots = iter(
                [
                    {"inference": 180, "mcp": 200},
                    {"inference": 300, "mcp": 330}
                    if renewed
                    else {"inference": 180, "mcp": 200},
                ]
            )

            def observe():
                actions.append(("observe", clock[0]))
                return next(snapshots)

            class Terminal:
                def __init__(self, *_args):
                    actions.append(("start", clock[0]))
                    self.transcript = b"permissions:"
                    self.turn = 0

                def __enter__(self):
                    return self

                def __exit__(self, *_args):
                    actions.append(("close", clock[0]))

                def choose_model(self, direction, model):
                    actions.append(("model", model))

                def wait_until(self, predicate, timeout=30):
                    if not predicate():
                        clock[0] += timeout - 15
                        actions.append(("idle", clock[0]))
                    assert predicate(), "Synthetic terminal predicate failed"

                def send_line(self, prompt):
                    self.turn += 1
                    actions.append(("turn", clock[0]))
                    rows = []
                    match = re.search(
                        r"write (oidc-expiry-\d.txt) containing exactly (OIDC_EXPIRY_\d_LOCAL_OK)",
                        prompt,
                    )
                    if match:
                        (root / match[1]).write_text(match[2])
                        if local_tool:
                            rows.append(
                                {
                                    "type": "item_completed",
                                    "item": {
                                        "type": "CommandExecution",
                                        "exit_code": 0,
                                    },
                                }
                            )
                    if mcp or self.turn < 3:
                        rows.append(
                            {
                                "type": "item_completed",
                                "item": {
                                    "type": "McpToolCall",
                                    "tool": "pan_inline_scan",
                                    "status": "completed",
                                    "result": {
                                        "structuredContent": {
                                            "results": {
                                                "action": "allow",
                                                "scan_id": f"synthetic-scan-{self.turn}",
                                                "profile_name": "Prisma AIRS Terminal",
                                            }
                                        }
                                    },
                                },
                            }
                        )
                    rows.append({"type": "task_complete"})
                    with (sessions / "continuous.jsonl").open("a") as stream:
                        stream.writelines(
                            json.dumps({"payload": row}) + "\n" for row in rows
                        )

            with (
                patch("airs_oidc_interactive.TerminalSession", Terminal),
                patch("airs_oidc_interactive.time.sleep"),
                patch("airs_oidc_interactive.time.time", side_effect=lambda: clock[0]),
            ):
                result = verify_interactive_refresh(
                    "binary", {}, root, home, observe, root / "receipt.json"
                )
            return result, actions

    def test_same_terminal_two_actual_resource_boundaries_and_local_tools(self):
        result, actions = self.exercise()
        self.assertEqual(result["completed_expiry_cycles"], 2)
        self.assertEqual(
            [c["turn_submitted_at"] for c in result["expiry_cycles"]], [205, 335]
        )
        self.assertTrue(
            all(
                c["after_both_expiries"] and c["local_file_verified"]
                for c in result["expiry_cycles"]
            )
        )
        self.assertEqual(
            [a[0] for a in actions],
            [
                "start",
                "turn",
                "model",
                "turn",
                "model",
                "observe",
                "idle",
                "turn",
                "observe",
                "idle",
                "turn",
                "close",
            ],
        )
        self.assertEqual(
            result["model_sequence"],
            [
                "gateway default",
                "@openai-terminal-auth/gpt-4.1",
                "gateway default",
                "gateway default",
            ],
        )

    def test_rejects_stale_second_resource_observation(self):
        with self.assertRaisesRegex(
            AssertionError, "Both resource tokens must have renewed"
        ):
            self.exercise(renewed=False)

    def test_file_alone_without_local_tool_evidence_does_not_pass(self):
        with self.assertRaisesRegex(
            AssertionError, "Missing successful bounded local file tool"
        ):
            self.exercise(local_tool=False)

    def test_failed_real_scan_cannot_be_replaced_by_turn_completion(self):
        with self.assertRaisesRegex(AssertionError, "Missing successful real MCP scan"):
            self.exercise(mcp=False)


if __name__ == "__main__":
    unittest.main()
