---
title: Corrected authentication source review
status: source-reviewed-release-not-ready
reviewed: 2026-09-08
source_commit: f964e5246dec3b8d98ab8ba6920c060a2aeb176c
reviewer: release_review
source_scope_score: 9
release_ready: false
tags: [authentication, source-review, regression, private-candidate]
---

# Bounded source assessment: 9/10

The corrected namespace protection, login/logout lifecycle integration, and
disclosed authenticated access probe meet a **9/10 source-stage assessment**.
The previously pending CLI test conditions are now cleared by the final
352/352 run. This is not a release score: **the authentication release remains
FAIL / not ready** against its mandatory acceptance plan.

The [c10 interim assessment](../c10e3f993/INDEPENDENT-READINESS.md) remains
historically correct at 8/10 for that defective runtime. Do not transfer this
review or new test results to c10 packages. This review covers the bounded fixes
authored by other agents; this reviewer's earlier prompt, cleanup, Mac deletion,
and workflow contributions still require a separate reviewer's approval.

## Evidence verified

The [CLI receipt](cli-tests.json) reports 352 tests passed, zero skipped, and
records the first attempt's 351 passes and one missing help snapshot. The final
run includes the reviewed snapshot and doctor credential-read correction.
I independently calculated the [JUnit](cli-junit.xml) SHA-256 and matched
`0f2ba0efec6d940a5e9526c7f99c427cd878c350c85043adb396d1bb6dae9d7e`.
Its run UUID is `903f22ba-414a-4140-949e-64bd3fd37dfb`, timestamp
`2026-09-08T18:47:11.775+00:00`; failures and errors are both zero.

The JUnit contains passing results for all three realistic namespace regressions,
nine access-probe tests, and the lifecycle revocation/failed-install tests.
The receipt also records scoped lint success without runtime changes and a
successful unchanged Bazel lock update. These remain unit/fault/localhost
results, not evidence of a native desktop or published executable.

## Reviewed corrections

- **Shared native namespace:** V1 raw workspace credentials and OIDC manifests
  address the same service/account root. The new guard rejects conflicting
  same-UUID formats before recovery deletes or persistence saves anything.
  Binding and pending metadata remain intact. The fake now models that alias,
  and tests assert unchanged bytes and zero mutations in both directions.
  Independent V2 cleanup still works.
- **Lifecycle:** login captures an epoch before waiting for the configuration
  lock. Commit checks it under the short state lock, writes a revoked epoch
  before installing files, and activates last. Failed installation remains
  fail-closed. Logout invalidates before waiting for long credential work and
  repeats invalidation after acquiring the configuration lock. No lock-order
  cycle was found. Both inference and MCP credential helpers recheck before
  emitting credentials.
- **Access verification:** the application discloses one fixed Responses request
  with a 16-token output limit, no tools/files, and `store:false`. Default routing
  omits `model`; explicit routing uses an allowed catalog route. Credential
  helper stdout and response bodies are bounded, helper stderr is suppressed,
  redirects are disabled, sensitive headers are marked, and response bodies are
  not rendered in error diagnostics. The probe distinguishes denial, transport
  failure, timeout, and invalid responses without changing saved credentials.
- **Probe wiring:** guided sign-in reports saved-versus-verified outcomes and
  provides `doctor --verify-access` for retry. That explicit doctor mode now
  skips the earlier synchronous native credential read; the bounded helper owns
  resolution. Ordinary doctor behavior is a separate unfinished contract.

No additional blocker was found in this bounded source review. The nine probe
tests cover request shape/routing, unsafe destinations, denial/soft failure,
redirect non-forwarding, oversized responses, revocation before send, helper
output/termination, messages, and token size. They do not establish every
possible live deadline or cancellation scenario.

## Conditions that remain open

Final Linux executable, installed package, live probe, Mac and Windows candidate
acceptance are not credited here. Doctor's total runtime also includes local
checks and an unauthenticated health probe before the 30-second access operation;
the entire command must not be described as having a 30-second deadline.

Mac run `34259924957` tested c10 and failed its original strict logout-message
assertion: 31 passed, one failed, two expected skips out of 34. Its transcript
shows an earlier credential-helper rejection; subsequent post-login assertions
and later Keychain/npm stages were not reached. This remains a retained failed
attempt, not a passed A14 gate. The executable's verified SHA-256 is
`fd88021ce5775066b49401cd60def9eabaf58bd95e753775b744a8702e2ace0a`;
it cannot validate the new namespace or probe implementation.

All other unresolved hard gates in the
[authentication plan](../../../../AUTHENTICATION-PLAN.md) remain open, including
the owner's Mac incident, desktop/terminal matrix, signatures and upgrades,
profile trust, final live authorization and refresh, teammate installation, and
unfamiliar-user trials. A source-stage 9 does not waive any of them.
