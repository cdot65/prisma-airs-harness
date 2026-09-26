# Retry-After connected feature review — validation in progress

This is an adversarial self-review backed by executable checks, not independent certification. All four completed-feature scores are withheld until the remaining gates close. The accepted scope remains monotonic retry advice for eligible requests, with AIRS authentication, authorization and routing boundaries preserved.

| Requirement | Current evidence | Remaining gate |
| --- | --- | --- |
| Delta/date/zero/expired/invalid/overflow advice | Shared parser: 120 local and 122 Apple Silicon package tests passed | Connected native source validation |
| Capture before error-body processing | Buffered/streamed, bounded/unbounded slow-body fixture passed | Full workspace/native |
| Bounded HTTP retries and expired advice | Attempt counts, preserved exhausted deadline and cancelled wait passed | Full workspace/native |
| Login/logout and network policy | Existing guard now wraps whole retry operation; logout during hour-long advice and policy revocation each observe only one request | Connected native |
| Rotating OAuth tokens | Real token-request fixture with Retry-After:0 retains exactly one request and rejection/ambiguous-response distinction | Full workspace/native |
| Session notification backpressure | Ten-second deadline still ends near ten seconds after six seconds of queue backpressure; restarted sixteen-second wait is rejected | Connected native |
| Transport fallback | Wiremock records at least the advised second between final WebSocket attempt and HTTP fallback | Full workspace/native |
| Turn cancellation | Real core turn interrupts an hour-long stream retry wait and observes one inference request | Full workspace/native |
| Terminal denials | 401/403/446 precedence, 200 HTTP hook rejection and terminal capacity classification passed with advice | All 206 API tests pass, including wrapped denial conflict; native/workspace pending |
| Local backoff compatibility | Ordinary backoff keeps its original duration; only server advice is reduced by elapsed time | Full workspace/native |
| Error/API compatibility | Existing CodexErr retry eligibility preserved; no RPC/schema change, daemon continuation or provider activation | Ten-package CLI/Guardian-inclusive lint passed; full workspace pending |
| Dependency consistency | Native Bazel regeneration and strict drift check passed; existing httpdate dependency reused, no package upgrade | Commit/audit final source |

The corrected stage-3 selection passed 806/806 with retries disabled. The initial 771/773 result remains in the artifacts. One failure exposed millisecond truncation from calculating a fresh remaining local-backoff duration; local timing now retains its original semantics. The other test polled at an exact paused-clock boundary before the timer wheel delivered readiness; it now awaits completion and allows at most 10 ms rounding, still rejecting a restarted delay by six seconds.

A subsequent review found that a wrapped WebSocket envelope could combine an explicit hooks_failed/soft-denial field with a retryable code. The existing gateway denial classifier now runs first, without searching model text or exposing the upstream diagnostic. Its first 206-test API run passed 205 tests and failed only the new assertion: the expected text incorrectly added an `invalid request:` prefix, whereas the established `CodexErr` display returns the policy message directly. The expectation is corrected; no production behavior changed. Corrected local and native validation remain required.

No release, complete feature score or signed binary is implied by these intermediate receipts. Fullscreen, terminal polish and signed Apple Silicon delivery remain separate parts of the overall goal.

The corrected API package passed 206/206 with retries disabled. Ten-package lint and formatting passed. The native connected run started before the one assertion correction; its source manifest is retained and the corrected test must be synchronized and validated afterward. Full workspace validation will use the committed corrected source. Scores remain withheld.

The first full native affected-package run completed 4,972 tests: 4,956 passed and 16 failed, with retries disabled. The original-deadline, logout, interruption and transport-fallback regressions passed. Failures include the known corrected assertion, host shell-profile contamination (reproduced independently with the same snapshot script: host profile fails validation, isolated profile passes), Apple Python launcher cache writes denied by Seatbelt, and MCP-startup/timing cases still under investigation. Follow-up tests retain every assertion, isolate only test-process profiles, use the installed Python interpreter directly, and include every failed test plus the complete API package. No environment failure is waived merely because it appears unrelated.

Native failures are now closed: the complete API package plus every original failed case passed 221/221, and all seven revised fixture cases passed locally and in three additional no-retry native repetitions. See `stage-04/CLOSURE-REVIEW.md` for causes and retained failed attempts. Ten-package local lint and formatting passed. Native lint is being completed after its disk-exhaustion failure; the full workspace still runs against exact production source 0da5d68cf1. Subsequent source edits are restricted to the three reviewed test-fixture files. No completed-feature score is assigned yet.

Corrected ten-package native lint passed with zero warnings. A full source parity check matched all 7,179 tracked Rust-workspace files. The initial native run also had 24 nextest skips outside its 4,972 executed cases; the required new retry regressions were all explicitly found and passed. The full GNU result is the only remaining connected feature gate.
