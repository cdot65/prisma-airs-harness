# Independent authentication release-readiness review

**Decision: FAIL — the requested >=9/10 threshold is not met. Formal weighted
score: NOT ESTABLISHED.** This is a point-in-time evidence audit on 2026-09-08;
Mac status below was observed at 20:42:44 UTC. It is not release approval.

Reviewer: `/root/release_review`. Runtime assessed:
`154b4f0bcef7528844e85177d7e4dce611982044`. The final Linux binary and installed
native both independently rehashed to
`2fcb93d5b11e5c81f2851b94a200bf81da4adba0ecb9d60b20ab8b206e81e3ad` during this
review. They remain private, non-publishable alpha.9 candidates. This audit did
not rerun tests, build Rust, dispatch CI or modify runtime code.

## Scoring and mandatory gates

The plan defines weights, equal-weight frozen subcases within each dimension,
and the minimum platform score as the delivery score. The repository contains a
scorecard template but no frozen, populated per-platform denominator. Therefore
neither a precise score nor a verified/frozen pass ratio can honestly be awarded.
**8.9 is only the plan's maximum with missing hard gates, not a current grade.**
This reporting gap is itself an actionable P0 defect; it is not permission to
replace missing measurements with an impression of progress.

| Plan dimension | Weight | Current full-scope disposition |
| --- | ---: | --- |
| Native persistence and incident resolution | 20 | Partial lower-layer evidence; owner Mac incident and required desktop cases unresolved |
| Guided onboarding/usability | 15 | Custom guided flows have Linux evidence; trusted profiles and unassisted trials incomplete |
| Gateway/policy/MCP | 15 | Real Linux positives and selected negatives pass; full platform/auth/audit matrix incomplete |
| Refresh/resume/logout/isolation | 15 | Linux subsets pass; two-expiry, fault and desktop matrix not complete |
| Secret handling/profile trust | 10 | Reviewed secret/diagnostic regressions pass; full profile trust contract and platform coverage incomplete |
| Installation/signatures/upgrades | 10 | Private Linux install passes; signed published new version, upgrades and teammate access incomplete |
| Diagnostics/cancellation | 5 | Linux fault/PTY subsets pass; native desktop/ConPTY matrix incomplete |
| Performance/repeatability | 5 | Measured isolated Linux targets pass; other required sessions unmeasured |
| Session/terminal compatibility | 5 | GNOME/KDE, Mac Terminal/Ghostty, Windows PowerShell/cmd and D1 scope unresolved |

Unknown or failed mandatory cases count as **not passed**, with no N/A assumed.
A01-A21 and A24-A30 are mandatory, with A26 conditional only after the recorded
D1 decision; A22 unassisted completion is also mandatory. A23 and A22 timing are
quality targets. Owner incident resolution, no credential disclosure/bypass and
unbroken integration are unconditional. All platform delivery verdicts are FAIL.

## What the evidence actually establishes

- **Linux:** final CLI units 359/359; private npm-installed executable 38/38;
  managed CLI 5.2.0; live OIDC/MCP 28/28 with actual allow scan IDs; workspace
  20/20 lifecycle runs and 20 disclosed minimal probes. Thirty credential-helper
  launches measured p95 23.2635 ms. Twenty Ctrl-C cases (40 cancellations) and
  twenty SIGTERM cases restored terminal state, with whole-case maxima below
  0.504 seconds. These are useful implementation results, not GNOME/KDE or
  selected headless onboarding evidence. The OIDC receipt identifies exact bytes
  but not its invocation path; it proves continuation after initial expiry, not
  every required two-expiry/concurrency/server-audit subcase. The workspace first
  failed fixture attempt remains preserved.
- **Apple Silicon:** run 34272843528 requests final source154. At the observed
  cutoff, preflight and ARM/cache steps passed; native compilation remained in
  progress from 20:08:07 UTC, with no executable/native/installed acceptance result.
  Offline preflight 21/21 is not Mac native acceptance. Earlier f964 hosted
  Keychain success is useful history, not final154 or affected-owner evidence.
- **Windows:** old c10 run 34259921603 was cancelled before executable completion;
  no raw executable or compilation cache survived. GitHub attributes cancellation
  to account cdot65, not an identified person/UI/API action. No timeout/OOM was
  established. New source-independent checkpoint workflow is locally validated
  but has not run natively. Its preparation earns no Windows runtime acceptance.
- **Distribution:** f94aba8d0 hardens only the legacy copy publisher: explicit
  review tags, early private/unvalidated rejection, immutable checks and failing
  pipeline propagation. Five mocked tests pass. It neither publishes current
  candidates nor solves normal scoped npm installation, signing or P5 migration.
  The later local npm10/npm12 feasibility controls support bundled CLI/SDK with
  actual scoped native names and exact versions; they supersede the shrinkwrap
  proposal. They used synthetic packages. Real Sharp OS/CPU/libc payloads, actual
  GitHub installs and teammate authorization remain unproven.

The source has meaningful verified repairs: credential transaction guards,
bounded/safe doctor reads, disclosed bounded access checks, lifecycle invalidation
and hidden-input handling. Earlier c10 namespace and pre-154 catalog failures
remain historical failures. No new source defect was established by this evidence
audit. That is not a comprehensive source-quality grade or proof of absent bugs.
The reviewer authored some fixtures and CI guard/checkpoint changes; root's
separate code review is required for those authored portions.

## Prioritized remediation

| Priority / cases | Concrete next action and regression evidence | Owner / dependency |
| --- | --- | --- |
| P0 / scoring, A19, A26, A30 | Freeze exact OS/session/auth subcases, profile trust contract and D1-D5 decisions; instantiate scorecards before calculating points. Record unknown applicability explicitly. | Parent can prepare matrix/contracts autonomously; product/headless and signing availability decisions require evidence/owner input where unresolved. |
| P0 / A07, A05-A06 | Collect safe backend operation/category/OS code on the affected Mac through app-owned diagnostics, reproduce and repair the actual failure, then rerun both auth methods and resume. Do not prescribe shell/keychain workarounds or assume signing caused it. | Owner device or faithful reproduction access required; source diagnosis/repair can proceed once evidence exists. |
| P0 / A01, A18, A27-A28 | Implement coherent new-version review packaging, correct ordinary scoped install and required signing; verify exact extracted bytes, CLI pin, credential/session upgrades and read-only teammate access. Never overwrite alpha.9 or strip private flags. | Packaging/install work autonomous; Developer ID/notarization, Windows signing and teammate access need actual credentials/access evidence. |
| P1 / Windows and Mac native gates | Finish the existing Mac run and retain all results. Validate the Windows library checkpoint and full candidate after cancellation clarification; preserve immutable artifacts before acceptance. | Platform agents; Windows relaunch currently pending clarification. No duplicate/cancelled builds to manufacture green status. |
| P1 / A03-A04, A19 | Complete profile/custom wizard and profile provenance/destination-change tests, preserving one realm/JWKS and local approval boundaries. | Parent implementation; reviewed administrator profile/trust inputs needed. |
| P1 / A08-A17, A25 | Expand final-byte tests to both auth modes, negative claims/routes/MCP, persisted audit, two access expiries, refresh interruption/concurrency, live-process logout and actual file/test/scan/resume effects. | Autonomous integration work with available gateway/IdP; required desktop sessions remain external evidence. |
| P1 / A02, A05-A06, A17, A23-A24, A29-A30 | Run real desktop/native denial, special-path, multi-user/restart, maximum-record, timing and repeatability cases on each frozen session. | Requires corresponding native desktops; hosted/synthetic results cannot substitute. |
| P2 / A20, A22, P6-P7 | Conduct guide-only trials with five unfamiliar participants per OS family and both auth methods; retain assistance and first failures. Recalculate independent scores, remediate, then promote identical signed bytes only after all gates pass. | Actual participants/owner evidence; agents may prepare fixtures/docs but cannot impersonate trials. |

The detailed [acceptance ledger](ACCEPTANCE.md), [Linux artifact receipt](linux-binary/receipt.json),
[installed tests](linux-binary/installed-tests.json), [OIDC receipt](linux-binary/oidc-live.json),
[workspace rerun](linux-binary/workspace-live-rerun.json), [cancellation trials](linux-binary/cancellation-trials.json),
[Windows cancellation/checkpoint review](../c10e3f993/windows-cancelled/REVIEW.md),
and [P5 assessment](../review-publication/ASSESSMENT.md) and [install feasibility](../review-publication/install-feasibility/REPORT.md) are the supporting record.
The companion evidence index freezes reviewed receipt hashes. Later Mac evidence
must be appended as a dated revision; no pending result is counted as passed here.
