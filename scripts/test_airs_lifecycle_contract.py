"""Deliberately fabricated records test the validator, never runtime acceptance."""

from copy import deepcopy
import unittest

from airs_lifecycle_contract import EventRecorder, validate_observation


def observation(profile="quick"):
    events = []

    def emit(at, kind, **fields):
        events.append(
            {
                "sequence": len(events) + 1,
                "observed_monotonic": at,
                "observed_unix": 1000 + at,
                "kind": kind,
                **fields,
            }
        )

    def issue(at, generation):
        for resource in ("inference", "mcp"):
            emit(
                at,
                "token_issued",
                resource=resource,
                generation=generation,
                grant="authorization_code" if generation == 1 else "refresh_token",
                access_expires_unix=1100 + at,
            )
            if generation > 1:
                emit(
                    at,
                    "refresh_requested",
                    resource=resource,
                    previous_generation=generation - 1,
                    accepted=True,
                    generation=generation,
                )

    def turn(at, generation):
        label = "turn_" + str(generation)
        emit(
            at,
            "resource_request",
            resource="inference",
            generation=generation,
            accepted=True,
            operation="responses",
            turn=label,
        )
        emit(
            at,
            "resource_request",
            resource="mcp",
            generation=generation,
            accepted=True,
            operation="tools/call",
            turn=label,
            call_id=label,
        )
        emit(at, "tool_completed", turn=label, call_id=label, nonce_verified=True)
        emit(
            at,
            "turn_completed",
            turn=label,
            unique_call_count=1,
            final_reply_verified=True,
            conversation_id="conversation",
            process_id=123,
        )

    def snapshot(generation, issued_at):
        return {
            "process_id": 123,
            "process_alive": True,
            "conversation_id": "conversation",
            "environment": "lifecycle",
            "binding_sha256": "a" * 64,
            "auth_epoch_sha256": "b" * 64,
            "gateway_origin": "synthetic-loopback",
            "inference_identity": "signed-synthetic-oidc",
            "mcp_connection": "airs-test-" + "c" * 24,
            "rollout_paths_count": 1,
            "resource_generations": {"inference": generation, "mcp": generation},
            "resource_expiries": {
                "inference": 1100 + issued_at,
                "mcp": 1100 + issued_at,
            },
            "event_count": len(events),
            "network_request_count": len(events),
        }

    issue(0, 1)
    turn(1, 1)
    before = snapshot(1, 0)
    quiet_snapshot = deepcopy(before) if profile == "idle" else None
    times = (
        [2103]
        if profile == "idle"
        else ([102, 204] if profile == "quick" else list(range(102, 3800, 200)))
    )
    for generation, at in enumerate(times, 2):
        issue(at, generation)
        turn(at + 1, generation)
    return {
        "schema_version": 1,
        "profile": profile,
        "started_monotonic": 2,
        "finished_monotonic": times[-1] + 2,
        "quiet_finished_monotonic": 2102 if profile == "idle" else None,
        "quiet_snapshot": quiet_snapshot,
        "baseline": before,
        "final": snapshot(len(times) + 1, times[-1]),
        "events": events,
    }


class LifecycleContract(unittest.TestCase):
    def reject(self, value):
        with self.assertRaises(ValueError):
            validate_observation(value)

    def test_complete_synthetic_contract_examples(self):
        for profile in ("quick", "active", "idle"):
            with self.subTest(profile=profile):
                self.assertTrue(validate_observation(observation(profile))["passed"])

    def test_short_elapsed_time_cannot_claim_timed_acceptance(self):
        for profile in ("active", "idle"):
            value = observation(profile)
            if profile == "active":
                value["finished_monotonic"] = value["started_monotonic"] + 3599
            else:
                value["quiet_finished_monotonic"] = value["started_monotonic"] + 2099
            self.reject(value)

    def test_missing_refresh_is_not_replaced_by_generation_increment(self):
        value = observation()
        row = next(row for row in value["events"] if row["kind"] == "refresh_requested")
        row.update(kind="network_request", operation="POST")
        for key in ("previous_generation", "generation", "accepted"):
            row.pop(key)
        self.reject(value)

    def test_expired_access_cannot_be_accepted(self):
        value = observation()
        row = next(
            row
            for row in value["events"]
            if row["kind"] == "resource_request" and row["generation"] == 2
        )
        row["generation"] = 1
        self.reject(value)

    def test_duplicate_or_unverified_tool_completion_fails(self):
        for field, replacement in (
            ("nonce_verified", False),
            ("call_id", "wrong_call"),
        ):
            value = observation()
            next(row for row in value["events"] if row["kind"] == "tool_completed")[
                field
            ] = replacement
            self.reject(value)

    def test_false_prose_success_does_not_prove_mcp_call(self):
        value = observation()
        for row in value["events"]:
            if row["kind"] == "resource_request" and row["resource"] == "mcp":
                row["operation"] = "tools/list"
        self.reject(value)

    def test_identity_or_process_change_fails(self):
        for field, replacement in (
            ("process_id", 999),
            ("conversation_id", "new"),
            ("environment", "other"),
            ("auth_epoch_sha256", "d" * 64),
            ("rollout_paths_count", 2),
        ):
            value = observation()
            value["final"][field] = replacement
            self.reject(value)

    def test_clock_reversal_fails(self):
        for clock in ("observed_monotonic", "observed_unix"):
            value = observation()
            value["events"][-1][clock] = -1
            self.reject(value)

    def test_idle_metadata_traffic_is_not_silence(self):
        value = observation("idle")
        value["quiet_snapshot"]["network_request_count"] += 1
        self.reject(value)

    def test_unbound_baseline_expiry_and_generation_fail(self):
        for field in ("resource_expiries", "resource_generations"):
            value = observation()
            value["baseline"][field]["mcp"] += 1
            self.reject(value)

    def test_unknown_secret_field_is_rejected_before_recording(self):
        recorder = EventRecorder()
        with self.assertRaises(ValueError):
            recorder("resource_request", resource="mcp", authorization="SECRET-CANARY")
        self.assertEqual(recorder.count, 0)

    def test_empty_or_out_of_order_evidence_fails(self):
        value = observation()
        value["events"] = []
        self.reject(value)
        value = observation()
        value["events"][0]["sequence"] = 2
        self.reject(value)

    def test_unknown_initial_credential_denial_is_recordable(self):
        recorder = EventRecorder()
        recorder(
            "resource_request",
            resource="mcp",
            generation=0,
            accepted=False,
            operation="initialize",
        )
        self.assertEqual(recorder.count, 1)
        with self.assertRaises(ValueError):
            recorder(
                "resource_request",
                resource="mcp",
                generation=0,
                accepted=True,
                operation="initialize",
            )


if __name__ == "__main__":
    unittest.main()
