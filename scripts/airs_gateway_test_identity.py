"""Give a live acceptance run its own OS-keyring account, even across homes."""

from dataclasses import dataclass, field
import hashlib
import json
import secrets


@dataclass(frozen=True)
class GatewayTestIdentity:
    endpoint: str
    name: str = field(
        default_factory=lambda: "airs-test-" + secrets.token_hex(12)
    )

    def accounts(self):
        # Match the native client's two serde_json map ordering configurations.
        payloads = [
            {"headers": {}, "type": "http", "url": self.endpoint},
            {"type": "http", "url": self.endpoint, "headers": {}},
        ]
        return [
            self.name
            + "|"
            + hashlib.sha256(
                json.dumps(payload, separators=(",", ":")).encode()
            ).hexdigest()[:16]
            for payload in payloads
        ]

    def verify_record(self, record):
        if record.get("url") != self.endpoint or record.get("server_name") != self.name:
            raise ValueError("Credential record does not belong to this acceptance run")
