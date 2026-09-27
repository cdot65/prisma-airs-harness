# Official Gateway OpenAPI alignment: 0.1.3-alpha.7.mcp.1

Source and frozen tooling: `61013886e841c8996e952692f1fcb8475cdd9ec3`. The signed/notarized Apple Silicon preview
is published at `https://npm.cdot.io` under `mac-preview`. It bundles CLI 7.2.0
and SDK 0.34.0. Stable remains 0.1.2; Linux builds await owner Mac acceptance.

The SDK audit distinguishes API planes and declared enum paths. Eight explicit
admin guardrail operations and typed API-key creation receipts are exposed by
the SDK; the CLI and Gateway skill preserve tenant/authentication boundaries.
The scoped source audit matches 132/187 operations. It does not claim complete
OpenAPI support or live mutation acceptance. Admin listing was verified read-only;
new write contracts were fixture-tested. Existing CAS, inference and native MCP
OAuth paths were not changed.

SDK: 11,755 tests passed. CLI: 2,041 tests passed, with built-executable wire tests
and a clean production dependency audit. Launcher: 33 passed; native skills: 51;
home-directory crate: 8. The owned Mac workflow passed. The signed binary ran 60
integration tests, with five expected skips before CLI bundling. Installed
candidate and fresh registry acceptance passed all ten stages plus native Jev,
actual-agent approval, Gateway skill and recovery checks. Stable 0.1.2 and prior
alpha.6 upgrade/rollback passed. The historical orchestration directory named
`alpha4-roundtrip` contains the actual alpha.6 roundtrip; read its version fields.
No paid Jev call, production mutation or new owner SSO/MCP login is claimed.

The public SDK, CLI and reference-architecture guides are deployed. The final
reference-architecture update passed all 23 browser checks. See READINESS.json
for bounded self-review scores and REQUIREMENTS-REVIEW.json for limitations.
SHA256SUMS inventories the retained evidence. Mac CI success is recorded by the
source-bound Forgejo run receipt; signed and installed test logs are retained.
