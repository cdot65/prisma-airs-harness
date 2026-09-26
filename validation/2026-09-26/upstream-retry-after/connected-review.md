# Retry-After connected feature review — accepted source integration

This is an adversarial self-review backed by executable checks, not independent certification. The four scores apply only to this bounded feature. Terminal rendering, fullscreen and signed Apple Silicon distribution remain required parts of the overall goal.

| Dimension | Score | Evidence and limit |
| --- | --- | --- |
| Implementation | 9/10 | One monotonic deadline survives body reads, error mapping, stream notifications, exhausted HTTP attempts and WebSocket-to-HTTP fallback. Expired advice remains explicit zero. Cancellation, logout and policy revocation have executable regressions. |
| Code quality | 9/10 | Shared parser with explicit API/error propagation; ten-package lint is clean on Linux and Mac, formatting passed, Cargo/Bazel consistency was checked natively, and all 7,179 tracked Rust-workspace files match the Mac. Failed attempts and fixture corrections are retained. |
| Design | 9/10 | Retry eligibility remains separate from advice. Authentication and explicit gateway denials stay terminal; rotating-token exchanges remain one-shot. Existing guard covers request backoff. Local backoff timing retains its original semantics. No new authentication path, provider activation or daemon continuation is introduced. |
| Feature completeness | 9/10 | Full GNU runtime-source validation: 18,596 passed, three exact inherited failures, 34 skipped; no retry-pass flakes. Native 4,972 executed cases have all 16 initial failures closed through the 221/221 final closure. Seven revised fixture cases pass locally and in three additional no-retry native runs. Distribution/owner acceptance is a separate later gate. |

## Evidence

Runtime commit: `0da5d68cf1`. Full workspace run: [Actions 310](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/310), database run 3852, exact matching source/tooling inputs. The comparison checks every failure assertion against the retained stable baseline, normalizing only timestamps, ANSI, indentation, process IDs and temporary paths. The suite is **not all-green**; the three inherited exec-server failures are explicitly recorded in `stage-05/WORKSPACE-BASELINE-COMPARISON.json`.

The only Rust changes after that workspace source are three test-fixture files in `42fb30eb11`, validated on Linux and Apple Silicon. They separate optional-MCP cold startup from catalog grace, expose actual startup failures, and prove mixed-tool concurrency with a barrier rather than total turn duration. Clean native lint/source parity is recorded in `5fe54f7e33`.

Parser prerequisites passed 120 local and 122 native tests. The connected selection passed 806/806; the final complete API package passed 206/206 locally. Required GNU and native regressions were checked individually as executed, including backpressure, transport fallback, turn cancellation, terminal-denial precedence, token refresh and logout while waiting. Native nextest had 24 skipped cases outside its 4,972 executed count; those are not counted as passes.

## Review findings resolved

- Logout monitoring originally covered individual requests but not the retry wait. The existing guard now wraps the complete endpoint retry operation, preserving post-auth policy checks.
- Mapping and notification work could restart a relative delay. The original deadline now crosses both boundaries, including fallback after retries are exhausted.
- Subtracting elapsed time from a newly created local backoff changed millisecond telemetry. Only server advice uses remaining time; ordinary backoff keeps its prior duration.
- Wrapped WebSocket envelopes could combine an explicit hook denial with a retryable code. The field-based denial classifier now wins without exposing private diagnostics or searching model text.
- Streamed numeric advice could panic on oversized floating-point durations. Checked conversion rejects overflow and preserves fractional milliseconds.
- Native validation exposed host-profile, Python-launcher and timing-fixture assumptions. The initial failures, targeted isolation, stronger causal assertion and successful closures are documented in `stage-04/CLOSURE-REVIEW.md`. No sandbox policy or owner profile was relaxed to obtain a pass.

## Boundaries

AIRS AI Gateway routing, CAS/OAuth and workspace-key ownership remain intact. Retry-After cannot grant replay permission for 401, 403, 446, explicit HTTP-200 hook denial, local-policy denial or rejected/ambiguous token exchange. Existing retry limits and eligibility are retained. Dependency reuse did not activate Bedrock or hosted-account behavior.

Scores remain 9 rather than 10 because this is a source-integration gate with recorded inherited suite failures and platform skips, not a claim of universal production behavior. No new signed package has been produced by this gate. Continue the accepted terminal/fullscreen work, then build, sign, notarize, publish and validate the actual Apple Silicon preview.
