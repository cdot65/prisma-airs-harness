# Retry-After integration

Adapt upstream `9d8de196748b57d7f463a7757eaa447b6483a331` while retaining AIRS error eligibility, authentication and policy boundaries. This is a prerequisite-by-prerequisite implementation within one feature; no feature receives scores until its connected behavior is validated.

1. Shared parser: monotonic deadlines, unsigned delta seconds and HTTP dates, explicit zero/expired advice, malformed/overflow rejection. Direct httpdate dependency, Cargo lock and actual Bazel regeneration/drift check.
2. HTTP plumbing: capture advice before buffered/error-body reads; preserve deadline on errors through bounded request retries and WebSocket HTTP errors. Mechanical constructor changes must not change semantic error handling. Test maximum attempts, disabled retry policy, exhausted error metadata, original deadlines and cancellation.
3. Stream/core integration: carry the same deadline through API and protocol mapping, notification queues, fallback transport and final exhausted errors. Preserve this fork's `is_retryable` decisions, including terminal overloaded/usage errors. Do not import upstream daemon continuation or its newer retry classifier solely to resolve patch conflicts. The existing core retry loop is the integration point.
4. Adversarial gate: 401/403/446, HTTP-200 hook rejection, local policy change and ambiguous rotating-token exchanges remain terminal. Cancellation interrupts a long wait; credentials and policy are revalidated before new traffic. Compare request counts as well as error messages.
5. Run affected package suites, full GNU workspace and connected native checks, then scoped lint and formatting. Record initial failures and exact closure, and separately score implementation, quality, design and completeness. Signed delivery remains a later integrated gate.

## Stage 1 review

The parser implementation and its two paused-clock tests match the frozen upstream commit. It only records timing; it grants no permission to retry and does not introduce a new retry consumer. An explicit expired deadline remains `Some(0)` rather than becoming absent advice. Wall time is sampled before monotonic time for HTTP dates, making a scheduling pause conservative. Checked instant addition rejects unrepresentable delays. The added dependency already exists in the workspace lock; no package version changes.

Initial local invocation omitted the existing test runner that removes inherited CA overrides: six native-TLS fixture assertions failed, while both new parser tests passed. The complete package passed 120/120 with that runner, retries disabled. This is an execution-environment correction, not a changed TLS assertion. Retain the initial JUnit artifact.

Bazel cannot execute its GNU binary directly on this Alpine host; actual regeneration runs on the existing Apple Silicon builder. The native default lock-check wrapper selected system Python 3.9 and failed before invoking Bazel; run the same helper explicitly with Python 3.13. Keep both failure and successful check logs.

Stage 1 native result: 122/122 passed with retries disabled; scoped native lint passed. All six changed-source/lock files match local bytes. Bazel regeneration and strict check passed with no MODULE lock delta. This prerequisite has no connected feature score yet.

## Connected review questions still open

The request layer rebuilds unary requests and clones the prepared **unauthenticated** streaming request (`codex-api/src/endpoint/session.rs`). Both paths then apply authentication and check the AIRS generation guard on every attempt. Review found the guard previously watched attempts but not backoff. Stage 2 moves the existing guard around the complete retry operation while retaining the post-auth check. A one-hour Retry-After is interrupted by logout within the existing 250 ms guard cadence on both paths. A separate policy-revocation fixture returns a terminal local-policy error with only one observed gateway request. Both regressions passed in the 428-test connected run.

Still open for stage 3: wrapped WebSocket errors whose retryable code conflicts with a terminal authentication/policy status, and stream/core notification delays. No completed-feature score until those checks, full workspace and native validation close.
