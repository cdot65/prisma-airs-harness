# Native MCP defaults and lifecycle evidence — September 19, 2026

Phases 1–3 passed independent review at **9.2**, **9.3** and **9.4/10**. These receipts precede the mcp.4 release candidate and do not establish its publication or installed acceptance. See the final phase-3 review for exact evidence and limitations.

## Runtime scope

The runtime change defaults newly created environments to native MCP credential storage. Existing environments, authentication identities and storage modes are preserved; there is no automatic credential migration. The development binary identifies itself as mcp.3 but is not the published package: SHA256 `bad190f6df618fa73d1d0fbe598d22c0cc84354498d3b91762ae4d7aafc4aae4`. Published baseline identities are recorded separately.

The continuous development-process observations used the immutable 152-file tooling source at `e8eb3a5997c66d006c99f6fdd97fb9654cc44b1c`. Active: 3601.08 seconds, seven actual post-expiry refresh cycles for each OIDC/MCP resource, 16 nonce-verified tool calls. Idle: 2100.09 seconds without HTTP traffic, followed by successful renewal and tool execution. Both retain one process/conversation, environment, binding, epoch and append-only history. Native credentials are synthetic and cleanup is scoped to those records.

Corrected native fixture tooling at `77bc03ebd8f71fa34095f30074c1beeb539da5e3` contains 153 files. Quick lifecycle acceptance against exact published mcp.3 binaries passed on native Linux ARM64, Apple Silicon and the fresh Ubuntu x64 host. The Darwin fixture preserves GUI HOME and uses exact Keychain metadata lookup; real binary reuse proves token usability.

## Checks and limits

Affected CLI suite: 955 passed. Current focused TUI recovery tests: two passed, 4351 not selected. Validator/fixture/provenance checks:25 passed under optimized Python; five history negative tests additionally cover14 secret placements and rewrite/truncate/delete/appendrollback using the actual frozen checker. The final review records installed onboarding and native fixture results separately.

Earlier partial-review wording and initial failures remain historical. The managed baseline failure was a launcher-context issue; unchanged tests pass through the installed launcher. The original Mac fixture used an unsuitable temporary HOME; its corrected exact native positive test is recorded. No speculative runtime refresh fix is claimed.

Production company SSO, real workspace-key policy, upstream ServiceNow OAuth and the owner's original credential-session incident remain deferred. Successful synthetic refresh does not establish rejected/ambiguous-grant recovery, concurrent races or rotated-generation durability in a newly launched process. The historical full workspace result remains 18,097 passed,150 failed,34 skipped, not a new or green suite run.

`COPIED-EVIDENCE.json` hashes the durable files. Private worker logs, tokens and credential databases are excluded. Final release acceptance must independently bind mcp.4 candidate native hashes and committed tooling on all three platforms.
