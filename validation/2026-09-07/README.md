# September 7 Linux pilot evidence

These receipts contain request/scan IDs and outcomes, without reusable credentials.
`gateway-contract.json` and `mcp-sessions.json` exercise the final stateless scanner
deployment. `scanner-final-runtime-agent.json` verifies both routes and resume on
that image using the preceding optimized binary. The final installed binary is covered by the installed-doctor, installed-interactive
and installed-live-agent receipts, plus upgrade-rollback. Its SHA and fixture
platforms are in VALIDATION.json. Post-publication CI is a separate release asset.

Cancellation, live compaction and invalid-profile receipts describe earlier
acceptance runs of the same runtime features; they are not falsely attributed to
the final artifact hash. The invalid profile was restored immediately after its
fail-closed probe. The historical hosted MCP receipt retains failures that motivated
the stateless REST-backed scanner. Earlier successes did not erase those failures.

Cargo audit reports zero vulnerability entries, with an scc unsoundness warning
outside the Linux CLI normal/build graph. The scanner image has 20 unfixed
medium/low findings and zero critical/high findings. See RELEASE.md for limits.

Inference-key scopes were reduced to completions.write/logs.write. The final
20-case gateway contract also proves that key cannot initialize MCP. The scope
propagation receipt preserves an early observation of previous permission before
the data plane converged to denial; no instant-revocation guarantee is implied.
