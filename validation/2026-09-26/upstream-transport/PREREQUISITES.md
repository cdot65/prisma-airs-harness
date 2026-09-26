# Transport enforcement — in progress

Stage 1 adapts upstream 973ec2942c (PR 45503): destination policy, revocable request permits, stale publication rejection, account-bound handles and factory identity. Four new policy regressions plus the existing HTTP suite pass: 96 tests, zero skips. Scoped lint/fix, format and source diff checks pass. Mac socket-fixture blocking/read-timeout corrections are included as upstream supplied.

Review: configured-but-unavailable state denies acquisition; policy load failure revokes outstanding work without changing account identity; account invalidation permanently rejects retained old clients; secure exact-host matching excludes subdomains/lookalikes and insecure schemes; unsupported SDK permits cannot bypass restrictions. Factory equality includes shared policy identity so policy owners are not conflated. Existing unmanaged behavior is retained until explicit policy ownership is wired.

This is a tested prerequisite, not completed request enforcement. Feature scores and release readiness remain pending HTTP/body/redirect/WebSocket integration, application ownership/reload propagation, AIRS gateway and CAS coverage, terminal denial/no-replay checks, full tests and signed delivery. The schema already exists in config while current core discards application requirements. AIRS inference explicitly refuses redirects in core/src/client.rs::build_api_transport; preserve that stronger boundary. No new proxy fallback is enabled by this stage.

## Stage 2: opt-in response bounds

Adapt ced02c5c38 (PR 45822), keeping the existing guardian sampler layout rather than importing its execution refactor. Per-request limits count observed bytes and reject oversized declared lengths; exact limits, zero/unbounded requests, chunked streams, missing length, interrupted bodies and error-response decoding are tested. An oversized response is non-retryable and reports only its limit. Ordinary requests remain unbounded unless a caller opts in. ModelsClient raw-byte API carries the limit through auth retries without leaking it to later requests; no new hosted catalog configuration is activated.

Affected HTTP/API/client/reviewer/debug-context suites: 403 passed, zero skips. Standard scoped lint succeeds with one large_enum_variant warning in the unchanged guardian Connection declaration (384-byte HTTP versus 152-byte WebSocket variant). The warning's baseline provenance is not established; assess it after the connected transport API changes and resolve or document it before final feature scoring. Do not label it a new functional failure or silently claim warning-free lint. Formatter/source diff checks pass. Native and full connected-feature acceptance remain pending; no transport feature scores yet.

## Stage 3: request execution and factory helpers

Apply the behavior-preserving request execution extraction in `3c6f32ca82`. Import only HTTP-client helpers from `3bb0a530d1` and `888be42a20`: route changes retain the factory and policy, redirect observation remains opt-in, request diagnostics can be disabled without rebuilding a differently routed client, and transport errors remove URLs. Proxy fallback still defaults off; no config feature defaults, login retry behavior or alternate model-catalog URLs are activated. These interfaces are prerequisites for the connected request/policy implementation.

HTTP suite: 105 passed, zero skips, including real custom-CA TLS and intercepted-proxy fixtures, error-response limits and redirect timeouts. Scoped lint/format and source diff checks pass. No final transport feature score yet.

## Stage 4: shared request construction

Import HTTP-client portions of `6ea62c4396`: draft construction preserves reqwest URL authentication and ordered header precedence once, then reuses the built request for redirects and TLS fallback. Route-aware transports resolve each redirect independently; fixed transports retain their existing route. No web-search consumer or default-client behavior is imported. HTTP tests: 111 passed, zero skipped. Scoped lint, formatting and source diff checks pass. Connected enforcement and native validation remain pending; no feature score yet.

## AIRS refresh transaction constraint for ownership integration

`airs_oidc::credential` deliberately writes a secret-free RefreshPending marker before sending a rotating CAS refresh grant. Discovery happens before that marker. Policy refusal before any refresh transmission must preserve the active credential, but revocation after transmission can have an unknown token-consumption outcome and must not resurrect/replay the predecessor. Do not mechanically transplant upstream refresh handling that assumes reusable refresh tokens. Distinguish preflight denial from in-flight revocation in tests and recovery copy; retain the existing transaction boundary. Private CAS requests must remain no-redirect, issuer/resource bound and response-size limited.

## Stage 5: connected HTTP and WebSocket enforcement

Request/body/redirect/WebSocket permits are wired, SSE and realtime preserve terminal denials, and caller body errors propagate. Realtime uses the shared connector without importing ChatGPT WebSocket cookies or voice defaults. Review and evidence are in `stage-05-review.md` and `stage-05.json`. Mac transport (334), retry (1), SSE (2), and caller (1,380) runs pass; Linux transport (432), proxy (1), retry (1), and corrected guardian (99) runs pass. The broader musl caller run has 146 documented platform/fixture failures; final GNU workspace validation remains required. Scoped lint/format pass after enum sizing correction. Application ownership and private AIRS propagation remain pending, so this is not a completed feature or release.
