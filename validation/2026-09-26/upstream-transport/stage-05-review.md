# Connected transport adaptation — review in progress

Sources: upstream `888e02db34` (PR 47389), plus realtime shared-connector prerequisite `142360dac8` (PR 47101). Prerequisite stages 1–4 are already separate commits. This remaining change is larger than the normal review target because replacing the shared request/error/response API requires its HTTP, WebSocket, SSE and caller conversions together to retain a compiling tree; adding temporary unmanaged adapters would weaken enforcement. New implementation modules stay below 500 lines. No public AIRS feature is activated by changing the existing realtime transport.

## Scope differences

- Keep AIRS `core::client::build_api_transport` no-redirect gateway handling unchanged.
- Do not import ChatGPT WebSocket cookie plumbing merely as patch context. Existing cookie behavior remains unchanged.
- Adapt guardian sampler errors in its existing module; do not import unrelated sampler execution extraction.
- Use this fork's two-argument plugin service constructor in the revoked-upload test; do not add upstream product SKU behavior.
- Realtime uses the shared connector and existing custom-CA semantics. Do not pull in the absent Windows platform-trust prerequisite or activate voice.
- Absent upstream TUI test reorganizations are not copied. Existing response construction is updated at actual call sites; full TUI compilation remains required.

## Adversarial review

Managed clients acquire authorization before proxy selection, check every redirect and retain permits for response consumption. Split WebSocket read/write sides have independent revocation wakeups. Loopback routing does not waive application policy. Typed denial survives HTTP error bodies, SSE parsing, realtime writers and connection closure, and retry orchestration treats it as terminal. Failed upload bodies no longer silently turn into success. Explicitly constructed unmanaged/direct clients remain legacy exceptions; application policy ownership and AIRS private clients are a separate pending integration stage and must be audited before feature completion.

Added fork regression cases exercise denial before route resolution, denied direct loopback without opening a socket, all four denial variants with every retry option enabled, and partial SSE output followed by typed revocation with exactly one request. These checks support transport behavior, not claims about credential retention, live CAS or remote gateway acceptance, which require the later integration tests.

No feature scores yet. The completed checks and limitations are recorded below. Application policy ownership and final workspace delivery gates remain pending.

## Completed checks so far

- Linux transport/API/guardian/debug-context run: 432 passed, zero skipped. This preceded the two additional WebSocket denial cases and the final enum boxing.
- Mac HTTP/WebSocket/API/client run: 334 passed, zero skipped, including both additional WebSocket cases. Additional no-replay test: one passed. SSE suite with new partial-output revocation case: two passed.
- Linux broader caller/API/protocol run: 1,454 passed, 146 failed, three skipped. All failures are executor tests: 144 fail to dispatch `--listen`, one delayed-child constructor dispatch fails, and one birthtime test assumes standard-library creation-time support matches rustix. Standalone probes reproduce both musl limitations; affected fixture and filesystem sources are byte-identical to the preceding commit. These are not silently removed or counted as passes.
- Mac caller/backend/plugin/feedback/executor/protocol run: 1,380 passed, zero failed, three skipped. This includes the constructor-dispatch cases that fail under musl; the Linux-only birthtime test is not applicable.
- New core configured-proxy integration: one passed on Linux, 1,709 excluded by the explicit focused filter. Network sandbox early-return variables were absent.
- Native standard HTTP/WebSocket/API/client lint passed. Broad local scoped lint completed with one guardian enum-size warning (392-byte HTTP versus 152-byte WebSocket), now corrected by boxing the HTTP variant; focused sampler validation and lint follow.
- Native Bazel lock regeneration and strict drift check passed; MODULE.bazel.lock did not change. Cargo.lock adds only tokio-util to the HTTP client package.

The final full workspace must run in the owned GNU validation environment with actual fixture binaries. No Linux package is authorized by these checks. No connected-feature score yet.

Final correction: boxed HTTP sampler client passes all 99 guardian tests with zero skips; follow-up scoped lint is warning-free. Formatter and source diff checks pass. The 18 unrelated formatter changes were restored. No additional tests were run merely because of formatting.
