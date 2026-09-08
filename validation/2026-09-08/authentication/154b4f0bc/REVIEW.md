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


## Addendum: completed Linux two-expiry and hosted Mac 15/26 evidence

Reviewed 2026-09-08 after Mac15 acceptance completed at **21:04:13 UTC** and
Mac26 at **21:13:13 UTC**. This supersedes the earlier pending status while preserving that historical
snapshot. **Overall FAIL and formal score NOT ESTABLISHED remain unchanged.**

- Linux's new first-attempt installed npm run passed **29/29** between
  20:57:38 and 21:02:20 UTC. The exact wrapper and native hashes were independently
  checked, as were the receipt and all six fixture hashes at source 67976. One TUI
  continued across **two recorded inference-and-MCP expiry boundaries**; each
  subsequent turn performed two successful local commands, verified its unique
  file, and returned a distinct real allow scan ID. The earlier 28-check receipt
  remains byte-identical. This closes the previously missing invocation-path and
  two-expiry-continuation evidence for this Linux fixture, not full A12. Private
  helper snapshots may themselves rotate near-expiry credentials before each
  wait; none run during the wait or following task. Concurrent/crash-safe refresh,
  exact server rotation count and required desktop coverage remain open.
- Mac15 run **34272843528** passed on hosted macOS **15.7.9 ARM64**. Native
  suite: **37 passed, 2 explicit skips / 39 methods**. Installed suite:
  **37 passed, 1 Linux-only skip / 38 methods**, including the managed CLI skill.
  The raw-only npm-managed skip is covered by the installed suite; Linux bus
  recovery is not a Mac scenario. Seven separate native-store phases passed,
  including the 16 KiB workspace and legacy/v2 cases. Both native and installed
  CLI Keychain receipts pass secure login, exact private helper readback,
  new-process access, local tool loop, plaintext-state absence and logout denial.
  Managed CLI 5.2.0 real corpus generation passed. The independently rehashed
  native is `f9ca78fccf5e16a18055b05b655e2fb1c5523362bf353d99c0de0daa2c19850b`;
  fixture and three downloaded artifact ZIP hashes also match canonical receipts.
  These are **ad-hoc signed, unpublished private bytes and loopback inference**,
  not Developer ID/notarization, real gateway/MCP, owner-Mac, upgrade or novice
  trials. Combined build step 47m42s is measured; no optimization speedup is inferred.
- The cancelled historical c10 Windows job did complete **26/26 identity/keyring
  contract tests, zero skips** at 18:01:09 UTC. The original log hash, nextest UUID,
  summary and unchanged identity/keyring crate tree IDs between c10 and 154 were
  independently verified. Mock storage contracts and typed-error tests do not
  establish live Credential Manager, a source154 Windows executable or ConPTY.
  Cancellation and the absent executable/cache outcomes remain unchanged.

Mac26 run **34278615096** subsequently passed on hosted **macOS 26.6.2 ARM64**
in **7m13s**, using the same source/archive/CLI/fixture identity as Mac15 without
Rust compilation. Actual logs again reconcile to **37 passed + 2 skips / 39 native
methods** and **37 passed + 1 Linux-only skip / 38 installed methods**. Seven store
phases, native and installed CLI Keychain lifecycles, and managed CLI corpus
contracts passed. The evidence ZIP SHA256 was independently verified; original
compilation provenance and newer validation tooling remain separately recorded.
This establishes hosted cross-version native/installed acceptance and the working
artifact-only retry path. It does not close signing, owner-device, real Mac gateway,
full desktop/user trials, Windows or publication gates. See
[the new evidence index](review-addendum-evidence.json),
[Linux two-expiry evidence](linux-binary/oidc-two-expiry-evidence.json),
[Mac15 receipt](macos-15/RUN-RECEIPT.json), [Mac26 receipt](macos-26/RUN-RECEIPT.json) and
[historical Windows contracts](../c10e3f993/windows-cancelled/native-contracts.json).

## Addendum: scoped CLI bundle and immutable Mac acceptance checkpoint

Reviewed 2026-09-08 by `/root/guided_login` after the final npm10 baseline run completed successfully at 22:38:22 UTC.
**This packaging/CI checkpoint passes its recorded cases. Full authentication
release readiness remains FAIL; the formal weighted score remains NOT
ESTABLISHED.** Existing scoring denominators and mandatory platform/owner gates
are unchanged. This review does not award the user's >=9/10 delivery threshold.

The maintained packaging implementation was independently reviewed across scoped
native resolution, archive extraction, exact dependency pins, full installed-tree
verification and npm's permitted first-shebang normalization. It now stages the
real Prisma AIRS CLI 5.2.0 and SDK 0.28.0 with locked archive integrity, preserved
licenses and target-specific image dependencies. Added files, omitted required
optional packages, Intel-Mac payloads and license tampering have explicit
rejection evidence. Declared optional source files which npm legitimately omits
remain separately inventoried. The source review and recorded tests do not prove
absence of other defects.

The earlier npm12 negative URL fixture was a false-positive attribution: unrelated
npm update-notifier traffic supplied the denied request. The retained
[correction](../review-publication/npm-registry-fixture-correction/ADDENDUM.md)
withdraws that claim. Corrected npm10 cases prove specific forbidden request
rejection. Corrected npm12 URL cases make no external request and instead fail
on the exact omitted package's byte-bound installed inventory. Both controls
matter; they establish different behavior. Real maintained Linux npm10/npm12
installs subsequently passed with four staged requests and zero unexpected/public
dependency redirects; their actual native image generation also passed.

The hosted Mac matrix reuses source154's immutable Apple Silicon binary
`f9ca78fccf5e16a18055b05b655e2fb1c5523362bf353d99c0de0daa2c19850b` and
artifact `10076349913`. No Rust compilation occurred in these acceptance runs:

| Host | npm | Run | Result |
| --- | --- | --- | --- |
| macOS 15.7.9 ARM64 | 12.0.2 | 34284909851 | Passed |
| macOS 26.6.2 ARM64 | 12.0.2 | 34284911571 | Passed |
| macOS 26.6.2 ARM64 | 10.9.8 | 34285996453 | Passed |

Each passing run verifies the scoped-name install and exact CLI pin; native
integrity is checked before the first native invocation. Mac bundles contain 68 exact-version packages, 66 licenses
and only the two Darwin ARM64 Sharp native payload packages. Successful install
receipts record exactly four staged registry requests, zero unexpected requests
and zero public dependency redirects. Node is 22.23.2. The full native suite
records 37 passes and two explicit skips from 39 methods; installed acceptance
records 37 passes and one Linux-only skip from 38 methods. Seven separate-process
store phases, native and installed CLI Keychain lifecycles, 20 managed CLI command
contracts and real PDF/PNG/JPEG/SVG/DOCX generation pass. These are deterministic
loopback authentication and offline corpus generation, not live Mac gateway/MCP
or document scanning.

The first bundled Mac attempts failed in cheap checks on a symlinked temporary
path mock. Their [original failure evidence](../review-publication/bundled-mac-cebe-preflight/README.md)
and independent Linux alias reproduction remain retained. Correcting the fixture
and adding its regression required neither native source changes nor rebuilding
the artifact. The split build/acceptance workflow and explicit artifact IDs kept
the previous compile usable through all retries. This establishes retry behavior;
it does not show faster Rust compilation. Newer workflow collection retains npm
failure diagnostics without modifying candidate bytes.

Original evidence ZIPs were independently checked against GitHub digests. All
13 packaging source-file hashes were compared with their recorded Git commits,
and native hashes match the independently retained original artifact. The larger
private candidate ZIPs are referenced by GitHub metadata; this review did not
independently download them. See [Mac15 npm12](../review-publication/bundled-mac-719/macos-15/RUN-RECEIPT.json),
[Mac26 npm12](../review-publication/bundled-mac-719/macos-26/RUN-RECEIPT.json),
[Mac26 npm10](../review-publication/bundled-mac-719/macos-26-npm10/RUN-RECEIPT.json)
and [maintained Linux bundle acceptance](../review-publication/maintained-bundle-d0c/REPORT.md).
The reviewer authored earlier Mac workflow splitting and some evidence tooling;
root reviewed those changes separately. This is not an independent review of
all self-authored implementation.

The next delivery gates remain actual authenticated GitHub Packages installation
with a read-only teammate account, a new publishable version with upgrade/session
and credential migration checks, required signing/notarization, and the affected
owner's unassisted Mac workflow. Windows native/installed authentication and
ConPTY acceptance remain open; installed Windows npm shim verification currently
fails closed and earns no bundled-Windows pass. Trusted profiles, frozen platform
subcases, required desktop sessions, complete live auth/policy/MCP negatives and
user trials retain their earlier dispositions. Private alpha.9 candidates were
not published or promoted by these tests. Hosted Macs, synthetic credentials and
loopback registry authorization cannot substitute for those outstanding gates.
