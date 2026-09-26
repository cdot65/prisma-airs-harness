# Application ownership adaptation — validation in progress

Remaining ownership slice of upstream `22a3f6d5d8` after separately committed HTTP/config/gRPC prerequisites. Shared policy lives with ConfigManager; local requirements constrain bootstrap, successful configuration loads capture a matching policy snapshot, failed loads revoke traffic, and authenticated clients bind to the current account. Typed policy denial remains terminal rather than becoming a credential failure.

This is a coherent cross-crate ownership API change, larger than the normal diff target. The prerequisite splits are already committed; partial caller conversions would either stop compilation or introduce unmanaged compatibility paths. No connected-feature score or release claim yet.

Fork adaptations:

- Apply policy to the existing session-layer rebuild path. Do not import absent provider-enforcement methods or their unrelated test organization.
- Preserve the existing proxy behavior under managed request admission. Do not add the upstream proxy-fallback feature.
- Keep the existing direct refresh implementation; preserve typed policy errors through request send and response-body reading without importing an absent OAuth client extraction.
- Bind browser authorization-code exchange to its exact token endpoint, retaining the existing exchange implementation.
- Keep normal backend cloud GET behavior; the absent upstream proxy-bootstrap retry helper is not needed for policy ownership.
- Adapt session/auth test helper interfaces and synchronous model-cache fixture calls to the fork.
- Keep private CAS rotating-refresh semantics as a separate required integration boundary; shared Codex auth tests cannot prove them.

Validation to date:

- Production app-server, TUI and CLI checks pass on Linux and Apple Silicon.
- Focused ownership regressions: 52 passed on each platform. A Mac output-handle leak in the overlapping-reloads test did not reproduce in the full suite or the later focused recheck.
- Linux five-package suite: 1,812 passed, one failed, two excluded by existing test configuration. The failing vendored zsh binary requires the missing GNU loader on Alpine; direct execution reproduces ENOENT outside the harness. The unchanged fixture must be exercised by the GNU workspace gate.
- Mac initial five-package suite: 1,781 passed, 32 failed, one timed out, one excluded. Six command-exec assertions encountered the user's RVM profile writing a process-info denial to stderr; a standalone sandbox-exec probe reproduces that warning. Their implementation and test source are unchanged in this slice. The other failures used an obsolete AIRS Codex fixture or a leaked Cargo runner override.
- Built a fresh native ordinary Codex test fixture and removed the Apple Cargo runner override in test children, matching the Linux runner behavior. The affected 39-test recheck passed; one executor/MCP startup test timed out first and passed on retry. This flake remains visible in the receipt.
- Core integration regression passed: policy-denied OAuth recovery is terminal and preserves the stored credential bytes.
- Lint found a missing pair of arguments in the TUI's test-only picker helper; it now passes isolated test loader overrides. The full native TUI suite is in progress; final scoped lint passes. Formatting passes with 18 unrelated formatter-only files restored.

No full-suite or release pass is claimed. Embedded startup propagation and AIRS CAS policy admission/rotating refresh remain required before rating this connected feature. Implementation/design/quality/completeness scores remain unassigned.
