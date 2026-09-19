"""Bounded, secret-free evidence for real installed lifecycle observations."""

import math
import re
import threading
import time


RESOURCES = {"inference", "mcp"}
PROFILES = {"quick", "active", "idle"}
LIMIT = 10000
FIELDS = {
    "token_issued": {"resource", "generation", "grant", "access_expires_unix"},
    "refresh_requested": {
        "resource",
        "previous_generation",
        "accepted",
        "generation",
        "reason",
    },
    "resource_request": {
        "resource",
        "generation",
        "accepted",
        "operation",
        "turn",
        "call_id",
    },
    "network_request": {"resource", "operation"},
    "tool_completed": {"turn", "call_id", "nonce_verified"},
    "turn_completed": {
        "turn",
        "unique_call_count",
        "final_reply_verified",
        "conversation_id",
        "process_id",
    },
    "phase_started": {"profile", "phase"},
    "phase_finished": {"profile", "phase"},
}
PINNED = (
    "process_id",
    "conversation_id",
    "environment",
    "binding_sha256",
    "auth_epoch_sha256",
    "gateway_origin",
    "inference_identity",
    "mcp_connection",
    "rollout_paths_count",
)
SNAPSHOT_FIELDS = set(PINNED) | {
    "process_alive",
    "resource_generations",
    "resource_expiries",
    "event_count",
    "network_request_count",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def number(value):
    return type(value) in (float, int) and math.isfinite(value)


def positive_int(value):
    return type(value) is int and value > 0


def safe_name(value, maximum=128):
    return (
        isinstance(value, str)
        and 0 < len(value) <= maximum
        and re.fullmatch(r"[A-Za-z0-9_.:-]+", value) is not None
    )


def validate_fields(kind, fields):
    require(
        kind in FIELDS and set(fields) <= FIELDS[kind],
        "Unknown lifecycle event or fields",
    )
    for name, value in fields.items():
        if name == "resource":
            require(value in RESOURCES, "Unknown credential resource")
        elif name in {
            "generation",
            "previous_generation",
            "process_id",
            "unique_call_count",
        }:
            unknown_denial = (
                name in {"generation", "previous_generation"}
                and type(value) is int
                and value == 0
                and fields.get("accepted") is False
            )
            require(
                positive_int(value) or unknown_denial,
                "Invalid lifecycle generation or count",
            )
        elif name in {"accepted", "nonce_verified", "final_reply_verified"}:
            require(type(value) is bool, "Invalid lifecycle boolean")
        elif name == "access_expires_unix":
            require(number(value) and value > 0, "Invalid token expiry")
        elif name == "grant":
            require(
                value in {"authorization_code", "refresh_token"}, "Unknown grant type"
            )
        elif name == "reason":
            require(
                value in {"consumed", "unknown", "idle_expired", "fixture_rejection"},
                "Unknown rejection category",
            )
        elif name == "operation":
            require(
                value
                in {
                    "responses",
                    "tools/list",
                    "tools/call",
                    "initialize",
                    "notifications/initialized",
                    "resources/list",
                    "resources/templates/list",
                    "GET",
                    "POST",
                },
                "Unknown fixture operation",
            )
        elif name == "profile":
            require(value in PROFILES, "Unknown lifecycle profile")
        else:
            require(safe_name(value), "Unsafe lifecycle identifier")


class EventRecorder:
    def __init__(self):
        self._events = []
        self._lock = threading.Lock()

    def __call__(self, kind, **fields):
        validate_fields(kind, fields)
        with self._lock:
            require(len(self._events) < LIMIT, "Lifecycle event limit exceeded")
            self._events.append(
                {
                    "sequence": len(self._events) + 1,
                    "kind": kind,
                    "observed_monotonic": time.monotonic(),
                    "observed_unix": time.time(),
                    **fields,
                }
            )

    @property
    def count(self):
        with self._lock:
            return len(self._events)

    def events(self):
        with self._lock:
            return [dict(event) for event in self._events]


def validate_snapshot(value):
    require(
        isinstance(value, dict) and set(value) <= SNAPSHOT_FIELDS,
        "Unknown snapshot fields",
    )
    require(
        set(PINNED)
        | {"process_alive", "resource_generations", "resource_expiries", "event_count"}
        <= set(value),
        "Incomplete lifecycle snapshot",
    )
    require(
        value["process_alive"] is True and positive_int(value["process_id"]),
        "Continuous harness process is not alive",
    )
    require(value["environment"] == "lifecycle", "Unexpected selected environment")
    require(
        value["gateway_origin"] == "synthetic-loopback"
        and value["inference_identity"] == "signed-synthetic-oidc",
        "Fixture scope or identity changed",
    )
    require(safe_name(value["conversation_id"]), "Missing conversation identity")
    require(
        re.fullmatch(r"airs-test-[a-f0-9]{24}", value["mcp_connection"] or "")
        is not None,
        "MCP fixture identity is not namespaced",
    )
    for name in ("binding_sha256", "auth_epoch_sha256"):
        require(
            isinstance(value[name], str)
            and re.fullmatch(r"[a-f0-9]{64}", value[name]) is not None,
            "Missing pinned credential identity",
        )
    require(
        positive_int(value["rollout_paths_count"]), "No retained conversation history"
    )
    for name in ("event_count", "network_request_count"):
        if name in value:
            require(
                type(value[name]) is int and value[name] >= 0, "Invalid snapshot count"
            )
    for name in ("resource_generations", "resource_expiries"):
        require(set(value[name]) == RESOURCES, "Missing resource snapshot")
        require(
            all(number(item) and item > 0 for item in value[name].values()),
            "Invalid resource snapshot",
        )
    require(
        all(positive_int(item) for item in value["resource_generations"].values()),
        "Non-integer resource generation",
    )


def validate_observation(observation):
    """Reject elapsed-only, replayed, unbound, secret-bearing or mocked-success proof."""
    require(
        set(observation)
        == {
            "schema_version",
            "profile",
            "started_monotonic",
            "finished_monotonic",
            "quiet_finished_monotonic",
            "quiet_snapshot",
            "baseline",
            "final",
            "events",
        },
        "Unexpected observation fields",
    )
    require(
        observation["schema_version"] == 1 and observation["profile"] in PROFILES,
        "Unknown observation schema/profile",
    )
    profile = observation["profile"]
    started, finished = (
        observation["started_monotonic"],
        observation["finished_monotonic"],
    )
    require(
        number(started) and number(finished) and finished > started,
        "Observation clock did not advance",
    )
    before, after = observation["baseline"], observation["final"]
    validate_snapshot(before)
    validate_snapshot(after)
    require(
        all(before[key] == after[key] for key in PINNED),
        "Process, conversation, environment, identity or history changed",
    )
    events = observation["events"]
    require(
        isinstance(events, list) and 0 < len(events) <= LIMIT,
        "Empty or oversized lifecycle observation",
    )
    require(
        after["event_count"] == len(events),
        "Final event count is not bound to evidence",
    )
    issued, refreshes, requests, completions, turns = {}, {}, [], {}, {}
    last_monotonic = last_unix = -math.inf
    base_keys = {"sequence", "kind", "observed_monotonic", "observed_unix"}
    quiet_end = observation["quiet_finished_monotonic"]
    if profile == "idle":
        require(
            number(quiet_end) and quiet_end - started >= 2100 and finished >= quiet_end,
            "Idle observation is shorter than 35 minutes",
        )
        quiet_snapshot = observation["quiet_snapshot"]
        validate_snapshot(quiet_snapshot)
        require(
            all(before[key] == quiet_snapshot[key] for key in PINNED),
            "Idle session identity changed",
        )
        require(
            "network_request_count" in before
            and quiet_snapshot.get("network_request_count")
            == before["network_request_count"],
            "HTTP traffic occurred during idle interval",
        )
    else:
        require(
            quiet_end is None and observation["quiet_snapshot"] is None,
            "Quiet interval belongs only to idle profile",
        )
    if profile == "active":
        require(
            finished - started >= 3600, "Active observation is shorter than 60 minutes"
        )
    for index, event in enumerate(events, 1):
        require(
            isinstance(event, dict) and base_keys <= set(event),
            "Incomplete lifecycle event",
        )
        require(
            event["sequence"] == index, "Lifecycle event sequence is not continuous"
        )
        kind = event["kind"]
        validate_fields(
            kind, {key: value for key, value in event.items() if key not in base_keys}
        )
        mono, unix = event["observed_monotonic"], event["observed_unix"]
        require(
            number(mono)
            and number(unix)
            and mono >= last_monotonic
            and unix >= last_unix
            and mono <= finished,
            "Lifecycle clock reversed or exceeds observation",
        )
        last_monotonic, last_unix = mono, unix
        if profile == "idle" and started < mono <= quiet_end:
            require(
                kind
                not in {
                    "network_request",
                    "refresh_requested",
                    "resource_request",
                    "token_issued",
                    "tool_completed",
                    "turn_completed",
                },
                "Network or model/tool work occurred during idle interval",
            )
        if kind == "token_issued":
            require(FIELDS[kind] <= set(event), "Incomplete token issuance")
            key = (event["resource"], event["generation"])
            require(
                key not in issued and event["access_expires_unix"] > unix,
                "Duplicate or expired token issuance",
            )
            previous_generations = [
                generation
                for resource, generation in issued
                if resource == event["resource"]
            ]
            require(
                event["generation"] == max(previous_generations, default=0) + 1,
                "Token issuance skipped a generation",
            )
            issued[key] = event
        elif kind == "refresh_requested":
            require(
                {"resource", "previous_generation", "accepted"} <= set(event),
                "Incomplete refresh observation",
            )
            require(
                event["accepted"] is True and "generation" in event,
                "Refresh failed in success observation",
            )
            key = (event["resource"], event["previous_generation"])
            require(
                key not in refreshes
                and event["generation"] == event["previous_generation"] + 1,
                "Rotating grant replayed or generation skipped",
            )
            refreshes[key] = event
        elif kind == "resource_request":
            require(
                {"resource", "accepted", "operation"} <= set(event),
                "Incomplete resource request",
            )
            if event["accepted"]:
                key = (event["resource"], event.get("generation"))
                require(
                    key in issued and unix < issued[key]["access_expires_unix"],
                    "Resource accepted absent or expired credentials",
                )
            requests.append(event)
        elif kind == "tool_completed":
            require(FIELDS[kind] <= set(event), "Incomplete tool completion")
            key = (event["turn"], event["call_id"])
            require(
                key not in completions and event["nonce_verified"] is True,
                "Tool replay or missing actual result nonce",
            )
            completions[key] = event
        elif kind == "turn_completed":
            require(FIELDS[kind] <= set(event), "Incomplete turn completion")
            require(
                event["turn"] not in turns
                and event["unique_call_count"] == 1
                and event["final_reply_verified"] is True,
                "Duplicate or unsuccessful turn",
            )
            require(
                event["process_id"] == before["process_id"]
                and event["conversation_id"] == before["conversation_id"],
                "Turn escaped pinned session",
            )
            turns[event["turn"]] = event
    require(
        before["event_count"] <= len(events) and before["event_count"] > 0,
        "Missing initial observation",
    )
    require(
        events[before["event_count"] - 1]["observed_monotonic"] <= started,
        "Baseline includes later events",
    )
    require(
        all(
            event["observed_monotonic"] >= started
            for event in events[before["event_count"] :]
        ),
        "Baseline omits earlier events",
    )
    for snapshot in (before, after):
        for resource in RESOURCES:
            generation = snapshot["resource_generations"][resource]
            key = (resource, generation)
            require(
                key in issued and issued[key]["sequence"] <= snapshot["event_count"],
                "Snapshot generation has not been issued",
            )
            known = [
                row
                for row in issued.values()
                if row["resource"] == resource
                and row["sequence"] <= snapshot["event_count"]
            ]
            require(
                generation == max(row["generation"] for row in known),
                "Snapshot omits newer credential generation",
            )
            require(
                snapshot["resource_expiries"][resource]
                == issued[key]["access_expires_unix"],
                "Snapshot expiry is not bound to issued token",
            )
    for key, token in issued.items():
        resource, generation = key
        if generation == 1:
            require(
                token["grant"] == "authorization_code",
                "Initial credential did not use OAuth",
            )
        else:
            predecessor = (resource, generation - 1)
            require(
                predecessor in issued
                and predecessor in refreshes
                and token["grant"] == "refresh_token",
                "Generation issued without real rotating refresh",
            )
    for (resource, generation), refresh in refreshes.items():
        require(
            (resource, generation) in issued and (resource, generation + 1) in issued,
            "Refresh has no issued generation evidence",
        )
        require(
            issued[(resource, generation)]["sequence"] < refresh["sequence"],
            "Refresh precedes its original grant",
        )
    for request in requests:
        if request["accepted"] and request["generation"] > 1:
            predecessor = (request["resource"], request["generation"] - 1)
            require(
                refreshes[predecessor]["sequence"] < request["sequence"],
                "Resource was used before observed refresh",
            )
    require(turns, "No actual completed tool turns")
    for turn, event in turns.items():
        calls = [(key, value) for key, value in completions.items() if key[0] == turn]
        require(
            len(calls) == 1, "Turn did not contain exactly one verified tool result"
        )
        call_id = calls[0][0][1]
        matching = [
            row for row in requests if row.get("turn") == turn and row["accepted"]
        ]
        require(
            any(
                row["resource"] == "inference" and row["operation"] == "responses"
                for row in matching
            ),
            "Turn lacks authenticated inference",
        )
        actual_calls = [
            row
            for row in matching
            if row["resource"] == "mcp"
            and row["operation"] == "tools/call"
            and row.get("call_id") == call_id
        ]
        require(len(actual_calls) == 1, "Tool result lacks a unique actual MCP request")
        require(
            actual_calls[0]["sequence"] < calls[0][1]["sequence"] < event["sequence"],
            "Tool completion precedes its request or follows completed turn",
        )
        require(
            all(row["sequence"] < event["sequence"] for row in matching),
            "Request occurred after its completed turn",
        )
    require(
        all(turn in turns for turn, _call in completions),
        "Tool completion has no completed turn",
    )
    required_cycles = 1 if profile == "idle" else 2
    counts = {}
    for resource in RESOURCES:
        cycles = 0
        for (kind, previous), refresh in refreshes.items():
            if kind != resource:
                continue
            prior = issued[(resource, previous)]
            operation = "responses" if resource == "inference" else "tools/call"
            if any(
                row["resource"] == resource
                and row["operation"] == operation
                and row.get("turn") in turns
                and row["accepted"]
                and row.get("generation") == previous + 1
                and row["observed_unix"] >= prior["access_expires_unix"]
                and row["observed_monotonic"] >= started
                for row in requests
            ):
                cycles += 1
        require(
            cycles >= required_cycles,
            "Missing actual post-expiry resource refresh cycles",
        )
        require(
            after["resource_generations"][resource]
            == max(generation for kind, generation in issued if kind == resource),
            "Final resource generation differs from observed issuance",
        )
        counts[resource] = cycles
    if profile == "active":
        times = (
            [started]
            + sorted(
                row["observed_monotonic"]
                for row in turns.values()
                if row["observed_monotonic"] >= started
            )
            + [finished]
        )
        require(
            all(end - begin <= 300 for begin, end in zip(times, times[1:])),
            "Active observation contains an unobserved gap over five minutes",
        )
    return {
        "passed": True,
        "profile": profile,
        "elapsed_seconds": finished - started,
        "post_expiry_cycles": counts,
        "completed_turns": len(turns),
        "event_count": len(events),
        "scope": "continuous installed process with synthetic HTTPS OIDC and gateway MCP; not production acceptance",
    }
