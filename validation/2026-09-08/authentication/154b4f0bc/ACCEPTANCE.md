---
title: Authentication 154 Linux candidate acceptance ledger
date: 2026-09-08
status: linux-evidence-retained-platform-acceptance-pending
source_commit: 154b4f0bcef7528844e85177d7e4dce611982044
binary_sha256: 2fcb93d5b11e5c81f2851b94a200bf81da4adba0ecb9d60b20ab8b206e81e3ad
release_ready: false
source_stage_score: null
supported_platform_score: null
tags: [authentication, acceptance, evidence, private-candidate]
---

# Linux candidate evidence; platform acceptance pending

The final-source Linux CLI suite passed **359 of 359 tests**, with zero failures
or skips. The original JUnit hash was checked; the [sanitized JUnit](cli-junit-sanitized.xml)
retains every test identity, duration and outcome without process streams.
The [receipt](cli-tests.json) also records the parent's successful scoped fix
command: 25.33 seconds and no runtime changes. That command's result is attributed
to its executing agent; this recorder did not rerun it.

Source **154b4f0bc** adds bounded doctor capability-catalog reads. The previous
[3acf checkpoint](../3acf1f045/CHECKPOINT.md) remains failed: the new catalog
regression accepted an oversized file and timed out on a FIFO when executed
against the older immutable binary. Passing unit tests here do not convert that
older executable result to a pass. The regular 154 executable and its installed npm native binary now have the
independently verified SHA-256 above. The installed catalog regression passes;
the older failed result remains preserved. The candidate is unpublished and
non-publishable. See the [Linux artifact receipt](linux-binary/receipt.json).


## Observed Linux results

| Evidence | Actual result | Scope and limits |
| --- | --- | --- |
| Immutable executable and local npm installation | Original and installed native SHA-256 match; native archive and both npm tarball hashes independently verified | [Build and artifact receipt](linux-binary/receipt.json); unpublished private candidate, not a teammate registry-access trial |
| Installed executable tests | 38/38 passed, zero skips, 33.709 seconds | [Installed receipt](linux-binary/installed-tests.json); includes catalog size/FIFO rejection, managed skill, local tool loop, routing and cross-process logout against deterministic loopback |
| Managed Prisma AIRS CLI | Pinned 5.2.0; command contracts and corpus generation pass | [Managed CLI receipt](linux-binary/managed-cli.json); no live management API operation or document detection claimed |
| Live OIDC and MCP | 28/28 checks pass, including browser/device flows, identity separation, local filesystem work, actual allow scan IDs, model switch and continued interactive use after initial token expiry | [OIDC receipt](linux-binary/oidc-live.json); exact tested binary hash matches installed package, but invocation path is not recorded in this receipt; does not prove every two-expiry/concurrency/server-audit subcase |
| Workspace first attempt | Guided login and bounded gateway probe passed; measurement phase failed on fixture startup warning | [Failed receipt](workspace-live-first-attempt.json) and [diagnosis](workspace-fixture-diagnosis.json); native key read succeeded, fixture root corrected without relaxing secret checks; first failure remains preserved |
| Workspace rerun | 20/20 credential lifecycles, 20 disclosed and verified gateway probes; initial live file task and new-process resume pass | [Rerun receipt](linux-binary/workspace-live-rerun.json); same binary hash as first attempt and installed native; repeat cycles use a fixed 16-token-maximum connectivity probe, not a conversation or refresh test |
| Local performance | 30 paired launches: native credential-helper whole-process p95 **23.2635 ms**, status p95 **10.5001 ms**; <=500 ms target met | Secret output privately matched exactly and not retained; helper includes startup/configuration/native resolution; status is metadata-only; no isolated backend or desktop-wide timing claim |

The successful later scanner calls do not establish the cause of the preserved
older failure. No raw OIDC phase transcript or secret-bearing helper output is
committed. The build wall time was 560 seconds; it is not an isolated optimization
comparison or evidence of a speedup. The workspace rerun finished at **2026-09-08 19:48:22 UTC**. Its first/rerun/installed
binary hashes match. This isolated Secret Service session does not substitute
for GNOME/KDE desktop or selected headless onboarding acceptance.

## Score interpretation

| Assessment | Current evidence | Score/status |
| --- | --- | --- |
| Bounded source implementation | 359 passing Linux CLI tests; scoped fix passed; independent source review recorded separately | No numeric score assigned by this evidence recorder; source evidence only |
| Apple Silicon macOS delivery | No final 154 published/installed desktop and owner-incident acceptance in this ledger | FAIL / mandatory evidence missing |
| Windows11 x64 delivery | No final 154 signed package and installed desktop acceptance in this ledger | FAIL / mandatory evidence missing |
| Linux delivery | Private candidate installed 38/38 and live OIDC 28/28; workspace 20/20 and helper timing target met; GNOME/KDE and selected headless contract evidence pending | FAIL / mandatory evidence missing |
| Whole supported-platform delivery | Minimum platform result under the plan; unknown hard gates cannot be waived | FAIL; no defensible >=9 score |

The plan caps any platform with an unknown or failed hard gate at 8.9. **8.9 is a
ceiling, not an awarded score.** A weighted numeric score requires the frozen
subcase inventory and per-platform evidence. A passing source-stage review does
not supply those missing measurements. This document does not promote a package.

## Remaining acceptance work

Each entry requires the final immutable binary/package and its own evidence.
Earlier-source receipts provide history, not automatic passes for changed bytes.

| Required gates | Outstanding final-candidate evidence |
| --- | --- |
| P0 decisions and matrix | Freeze exact OS/session/terminal/Node builds, profile trust contract and selected headless persistence design; retain unresolved decisions. |
| A01, A18, A27, A28 | Fresh standard-user installs, pinned managed CLI, signed/notarized Apple Silicon and signed Windows artifacts, Linux provenance, real upgrade/downgrade, teammate read-only package access. |
| A02-A04, A19 | Both guided authentication flows, profile-driven browser/device behavior, protocol rejection/cancel coverage, trusted profile and destination-change protection. |
| A05-A07, A17 | Both credential modes across separate processes, OS-user separation, lock/denial/corruption/interruption cases, maximum native records; diagnose and resolve the actual unlocked-Mac incident with safe OS status and owner retest. |
| A08-A11, A25 | Both modes against live gateway, negative authorization and persisted audit, default/explicit routing, separate MCP authorization, real skill/file/test/scan effects and restart/resume on the same published bytes. |
| A12-A15 | Two access-token expiries, refresh concurrency/interruption, identity-safe history/resume, cross-process logout within 5 seconds, remote/offline revocation semantics. |
| A16, A21, A24 | Final leak inspection, typed safe diagnostics, bounded doctor/catalog regression, cancellation/deadlines in actual PTY/ConPTY sessions. |
| A20, A22 | Unfamiliar-user guide trial and five participants per OS family completing both methods unassisted; report timing separately from mandatory completion. |
| A23, A29 | Thirty warm credential resolutions per platform with p95 <= 500 ms; twenty installed credential lifecycles with zero unexplained failures and first attempts preserved. Whole-helper timing includes startup and is not isolated backend latency. |
| A26, A30 | Selected supported headless flow, GNOME/KDE prompts, Mac Terminal/Ghostty, Windows PowerShell/cmd, special paths and denied consent; no manual shell workaround credited as onboarding. |

A01-A21 and A24-A30 are hard gates, with A26 conditional only on the recorded D1
decision. A22's unassisted completion is mandatory; its timing and A23 are scored
quality targets. Missing evidence earns no points. Owner-incident resolution,
zero disclosure/bypass and unbroken gateway/MCP/local-tool integration remain
unconditional requirements.

## Next evidence and remediation order

1. Retain the completed Linux artifact/install/OIDC evidence and the red-before
   receipts. These exact bytes remain a non-publishable candidate.
2. Retain the completed workspace 20-cycle/30-measurement rerun and its earlier
   failed attempt. Both functional and measured p95 targets passed in this
   isolated Linux session; expand to the required platform/session matrix.
3. Diagnose the owner's Mac incident; complete signed native artifacts and the
   desktop/session, upgrade, teammate, repeatability and unfamiliar-user matrices.
4. Attach independent per-platform scorecards with frozen cases. Remediate every
   failed or unknown hard gate before considering promotion of the same bytes.

Implementation ownership stays with the parent/integration agent, platform
validation owners and independent reviewer. Owner-device and participant trials
require actual human evidence. Pending platform and full-contract outcomes
must replace pending fields through additive receipts; no measurements are
invented here. See [the governing plan](../../../../AUTHENTICATION-PLAN.md).
