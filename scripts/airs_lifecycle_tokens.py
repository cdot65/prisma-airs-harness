"""Rotating synthetic grants; never used with production credentials."""

import secrets
from enum import Enum
import threading
import time


class RefreshFault(Enum):
    REJECTED = "rejected"
    RESPONSE_LOST = "response-lost"


class RotatingTokens:
    def __init__(self, resource, lifetime, emit, *, clock=time.time):
        if resource not in {"inference", "mcp"} or not 45 <= lifetime <= 3600:
            raise ValueError("Invalid lifecycle token configuration")
        self.resource, self.lifetime, self.emit, self.clock = (
            resource,
            lifetime,
            emit,
            clock,
        )
        self.lock = threading.RLock()
        self.generation = 0
        self.expires = 0
        self.access = {}
        self.refresh = {}
        self.consumed = set()
        self.next_fault = None

    def fail_next_refresh(self, fault):
        if not isinstance(fault, RefreshFault):
            raise ValueError("Unknown synthetic refresh fault")
        with self.lock:
            if self.next_fault is not None:
                raise ValueError("A refresh fault is already armed")
            self.next_fault = fault

    def issue(self, build, grant="authorization_code"):
        with self.lock:
            generation = self.generation + 1
            expires = int(self.clock()) + self.lifetime
            response = build(generation, expires)
            access, refresh = response["access_token"], response["refresh_token"]
            if (
                access in self.access
                or refresh in self.refresh
                or refresh in self.consumed
            ):
                raise ValueError("Fixture issuance reused a credential")
            self.access[access] = (generation, expires)
            self.refresh[refresh] = generation
            self.generation, self.expires = generation, expires
            self.emit(
                "token_issued",
                resource=self.resource,
                generation=generation,
                grant=grant,
                access_expires_unix=expires,
            )
            return response

    def exchange(self, values, build):
        token = values.get("refresh_token", [""])[0]
        with self.lock:
            prior = self.refresh.pop(token, None)
            if prior is None:
                self.emit(
                    "refresh_requested",
                    resource=self.resource,
                    previous_generation=0,
                    accepted=False,
                    reason="consumed" if token in self.consumed else "unknown",
                )
                return 400, {"error": "invalid_grant"}
            self.consumed.add(token)
            fault, self.next_fault = self.next_fault, None
            if fault is RefreshFault.REJECTED:
                self.emit(
                    "refresh_requested",
                    resource=self.resource,
                    previous_generation=prior,
                    accepted=False,
                    reason="fixture_rejection",
                )
                return 400, {"error": "invalid_grant"}
            # Consumption precedes issuance: a failed response cannot restore a predecessor.
            response = self.issue(build, "refresh_token")
            self.emit(
                "refresh_requested",
                resource=self.resource,
                previous_generation=prior,
                accepted=True,
                generation=self.generation,
            )
            return None if fault is RefreshFault.RESPONSE_LOST else (200, response)

    def authorize(self, token, operation, **details):
        with self.lock:
            generation, expires = self.access.get(token, (0, 0))
            accepted = bool(generation and self.clock() < expires)
            self.emit(
                "resource_request",
                resource=self.resource,
                generation=generation,
                accepted=accepted,
                operation=operation,
                **details,
            )
            return accepted


def opaque_response(generation, expires, lifetime):
    return {
        "access_token": "fixture-access-" + secrets.token_hex(24),
        "refresh_token": "fixture-refresh-" + secrets.token_hex(24),
        "token_type": "Bearer",
        "expires_in": lifetime,
    }
