---
title: Interim authentication release readiness review
status: not-release-ready
reviewed: 2026-09-08
plan_version: "0.1"
source_commit: c10e3f99393f7d79dbe48fa228a9a18e548953e1
reviewer: release_review
evidence_scope: repository-only
tags: [authentication, acceptance, independent-review, private-candidate]
---

# Interim verdict: release not ready

The authentication release does **not** meet the hard gates in
[AUTHENTICATION-PLAN.md](../../../../AUTHENTICATION-PLAN.md). Do not promote the
candidate or describe the owner's Mac incident as resolved. This assessment is
an interim evidence snapshot, not a prediction of currently running builds.

The bounded source assessment for `c10e3f993` is now **8/10, remediation required**.
This supersedes the earlier 9/10 assessment: root and the credential reviewer
confirmed a shared native-namespace defect in transaction cleanup and persistence
that the per-kind fake store did not model. Existing passing tests do not prove
identity preservation under this malformed-state case. This assessment concerns
source behavior, not full release readiness or installed usability.
This reviewer authored parts of the earlier prompt, cleanup, Mac
deletion, and workflow work; those contributions still need another reviewer's
approval. The independent source assessment here concerns the lifecycle and
typed OIDC integration authored by other agents.

**Confirmed source blocker:** `WorkspaceKeyringV1` and `OidcIdentityV1` use the
same native `SERVICE + UUID` address, despite having different logical store
kinds. An inconsistent same-UUID binding and pending raw-cleanup record can
therefore cause recovery to delete an active OIDC manifest. Persistence can also
overwrite that shared address before recovery protects it. The fake keyed by
`(StoreKind, UUID)` incorrectly represented these entries as isolated and hid the
failure. The plan explicitly requires preserving identity and usable credentials
when metadata is malformed; this is not an acceptable corruption exception.

The proposed guard and realistic shared-namespace regression fixture were
prepared but **not applied to this reviewed runtime** at this amendment's cutoff.
Require rejection before any destructive save/delete, preservation of the
existing native record and binding, and retained actionable cleanup state. Review
and rerun affected transaction/recovery regressions against the corrected commit
before restoring any source score of 9 or above. Candidate acceptance for
`c10e3f993` cannot clear this known defect.

No defensible weighted **release score** can yet be calculated: P0 has not
recorded a frozen platform/subcase denominator, and the scorecard is still a
template. Mac, Linux, and Windows each have unknown mandatory gates and therefore
each have a **FAIL / incomplete** release verdict. The plan's 8.9 ceiling is a
cap, not an awarded score. Unexecuted cases receive no credit. At the full-row
level, none of A01–A30 is established in its entirety for this candidate across
all required sessions; this does not erase the substantial partial evidence
below. A26 remains unresolved, not implicitly N/A.

## Evidence inspected and its limits

| Evidence | Verified result | What it establishes |
| --- | --- | --- |
| [CLI receipt](cli-tests/receipt.json) and [JUnit](cli-tests/nextest-junit.xml) | 339 passed, zero failures/skips; JUnit SHA-256 matches receipt | CLI unit/fault coverage attributed to this runtime; not installed CLI acceptance. Recorded scoped lint completed without runtime source fixes. |
| [Library receipt](scoped-tests/receipt.json) and [JUnit](scoped-tests/nextest-junit.xml) | 363 passed; JUnit SHA-256 matches receipt | Localhost inference/MCP request boundaries and epoch behavior. The excluded child-process entry is documented and invoked by its passing parent. Linux keyring-store selection has no executed unit cases in this JUnit. |
| [Mac native receipt](native/airs-harness-native-identity-macos-15.json) | Seven fixture processes passed | Native persistence, legacy/V2 workspace formats, 16,384-byte workspace key, and cleanup. Not the owner's Mac, browser login, terminal UX, or installed package. |
| [Linux native receipt](native/airs-harness-native-identity-ubuntu-24.04.json) | Seven fixture processes passed | Same bounded storage fixture on Linux; not GNOME/KDE/SSH acceptance. |
| [Windows native receipt](native/airs-harness-native-identity-windows-2025.json) | Seven fixture processes passed | Same bounded storage fixture on a hosted Windows runner; not Windows 11 PowerShell/cmd or ConPTY prompt acceptance. |

Native receipts identify workflow run `34259890077` and artifact IDs. They do not
include final product binary digests, complete OS/session inventories, or a full
native JUnit inventory. Do not infer additional test counts from workflow success.
The two JUnit receipts explicitly explain working-tree-to-commit attribution;
they are not claims that every file in the commit was tested.

Mac candidate run `34259924957`, Windows candidate run `34259921603`, and the
final Linux binary were pending at this review's evidence cutoff. No credit is
awarded for them. Earlier Linux workspace/Keycloak/MCP checks reportedly used
pre-P4 `4ad…` binary hashes; those are regression context, not acceptance of the
new request-revocation runtime. Existing alpha.9 publication evidence likewise
cannot establish candidate acceptance.

## All acceptance rows

“Partial” means useful implementation or lower-layer proof exists, but the
complete required row is not passed. “Missing” includes evidence not present in
the inspected candidate records. No row below grants credit for an intention.

| Gate | Candidate assessment | Remaining pass evidence or implementation |
| --- | --- | --- |
| A01 — standard-user install | Missing | Final packages installed on each supported platform, correct native selection and CLI pin, no Rust/manual shim steps. |
| A02 — hidden input | Partial | Prompt/fault source coverage exists. Real desktop paste/cancel/interruption and Windows Ctrl+Enter/ConPTY delivery remain required. |
| A03 — profile/browser login | Partial | Saved public IdP settings and browser flow exist; profile import/trust wizard is absent. Complete installed/live browser and rejection cases. |
| A04 — device flow | Partial | Existing device implementation is not new-candidate live proof. Retain expiry/denial/unsupported/cancel evidence for the selected sessions. |
| A05 — persistent lifecycle | Partial | Seven-process storage fixtures pass on three OS families. Both real auth modes, installed restart/logout/reboot cycles, and cross-user isolation remain. |
| A06 — native failure classes | Partial; known preservation failure | Typed diagnostics exist, but the shared-namespace defect can destroy prior good state. Fix it and complete actual locked/denied/cancelled/absent/wrong-session/corrupt cases. |
| A07 — owner Mac incident | Missing; unconditional blocker | Capture the actual OS status and demonstrate repair on the affected Mac or faithful reproduction; retain owner retest separately. A deletion-status repair is not a diagnosis of login failure. |
| A08 — authenticated gateway | Missing for final bytes | Both modes, denied/expired credentials, outage classification. Login currently saves credentials without a bounded authenticated gateway verification step. |
| A09 — server authorization/audit | Missing for final bytes | Negative issuer/audience/client/role/scope/signature/expiry and spoofing tests, with persisted server-side user attribution. |
| A10 — model routing | Partial | Preserve existing omission/explicit-route behavior; rerun actual candidate wire/live switching, unsupported-parameter normalization, and denied routes. |
| A11 — MCP authorization | Missing for final bytes | Real allowed tool result and wrong identity/audience/scope rejection; optional denial must leave inference usable. |
| A12 — refresh | Partial | Durable pending and serialized storage mechanisms exist. Two live token expiries, concurrency, and fault behavior through the installed TUI remain. |
| A13 — resume/isolation | Partial | Existing history boundary and new epoch code have source tests. Run installed restart/resume and identity/gateway/environment rejection matrix. |
| A14 — logout across processes | Partial | Localhost cached-client/new-request and inference stream cancellation tests pass. Actual installed two-process tests are pending; already-open MCP response bodies remain outside the recorded bounded stage. |
| A15 — revocation/offline | Partial | Local-first invalidation and explicit remote uncertainty are implemented. Final live/offline receipts and measured server token validity remain. |
| A16 — secret disclosure | Partial | Redacted error and transaction fixtures exist. Complete final installed canary scans across arguments, logs, history, model context, and child environments on each platform. |
| A17 — bounds/crash safety | Partial; confirmed source blocker | Shared-namespace raw cleanup/persistence can damage an active OIDC record; existing per-kind fakes miss this. Fix before claiming malformed-state preservation, then complete backend exhaustion, corrupt chunks, interruption, and maximum OIDC bundle matrix. |
| A18 — upgrade/downgrade | Missing | Alpha.9 → candidate → next build, actual prior credential access, path/Node-manager relocation, stable signing, and safe incompatible downgrade. |
| A19 — profile trust | Missing implementation | Freeze profile contract/trust model, implement validation/import/update behavior, and test adversarial inputs. Manual IdP prompts are not a profile wizard. |
| A20 — unfamiliar-user guide trial | Partial | Candidate guide avoids credential shell recipes. Actual unfamiliar-user install/auth/task/resume trial remains. |
| A21 — diagnostics | Partial | Safe operation/category/native code mapping exists. Complete native fault classes and redacted-bundle review; an app-owned bundle flow is not implemented. |
| A22 — novice completion and timing | Missing | Five unfamiliar participants per OS family, both methods, no intervention; preserve total/phase timings and first failures. Completion is mandatory. |
| A23 — warm resolution performance | Missing quality evidence | Thirty installed warm launches per required platform/session; measure p95 overhead and confirm no write/delete probe during ordinary startup. |
| A24 — bounded waits/cancellation | Partial | Bounded state lock, prompt restoration and request polling exist. Prove all noninteractive deadlines and 20 ≤2-second cancellation trials in real terminal sessions. |
| A25 — complete production task | Missing | Same final published bytes, both modes per platform: skill/read/edit/tests, authorized model change, real MCP scan, restart and resume. |
| A26 — headless | Unresolved conditional gate | Record D1/D5 before declaring applicability; implement and test the selected persistence/unlock contract if included. No manual bus/secret recipes. |
| A27 — signing/provenance | Missing | Developer ID/notarization and Windows signatures verified on final extracted bytes; Linux checksums/provenance. Ad-hoc/private candidates do not pass this gate. |
| A28 — teammate package access | Missing | Fresh read-only teammate registry access to every final native package, without owner/admin credentials; verify intended inheritance. |
| A29 — repeatability | Missing | Twenty consecutive installed lifecycle runs per platform with zero unexplained failures and retained first-attempt/rerun records. |
| A30 — desktop/terminal matrix | Missing | GNOME/KDE, Windows PowerShell/cmd, Mac Terminal/Ghostty, special paths, and denied native consent. Hosted storage success is insufficient. |

## Prioritized remediation

| Priority / owner | Concrete work | Dependency and regression evidence |
| --- | --- | --- |
| P0 — credential/runtime owners | Reject inconsistent same-UUID logical formats sharing one physical native address before save/delete; preserve the active record and cleanup metadata. Replace the misleading per-kind fake with realistic aliasing coverage. | Guard fix is not in reviewed `c10e3f993`. Test raw↔OIDC pending cleanup and persistence collisions, unchanged active bytes/binding, safe retry, and non-collision behavior. Independent review and affected shared regressions required; source assessment stays below 9 until verified. |
| P0 — release owner | Freeze D1–D5 and per-platform subcases, including terminals, OS builds, signing access, profile trust, and headless applicability. | Owner decisions and accessible environments; instantiate scorecards without changing weights or removing failed cases. |
| P0 — incident owner + platform engineer | Diagnose the reported Mac with bounded native error metadata and reproduce the failing access path. | Affected-device access or faithful reproduction; A07 plus both login modes and subsequent restart. Never request secrets in reports. |
| P0 — runtime/test owners | Finish exact-binary installed logout, helper, inference and MCP boundaries, including behavior of already-open MCP streams. | Final candidates; A12–A16 and actual two-process tests. Retest after any runtime fix. |
| P1 — onboarding owner | Complete reviewed profile wizard/trust rejection and honest saved-versus-authorized status with bounded authenticated access validation. | Frozen D2/profile schema; A03, A08, A19; network outage, denied route, and optional MCP failure tests. |
| P1 — platform owner | Complete native consent UX, explicit/noninteractive probe boundaries, Windows prompt behavior, and desktop failure recovery. Review `status` native access against the no-surprise-prompt contract. | Native desktops/terminals; A02, A05–A06, A21, A24, A30. |
| P1 — release engineering | Produce final signed packages, finish upgrade/relocation/downgrade and teammate-access trials. | Signing identity/protected credentials and ordinary teammate account; A01, A18, A27–A28. No production promotion before gates pass. |
| P1 — integration owner | Repeat live gateway policy, user audit, routing, refresh, MCP and full coding task on final bytes in both modes. | Authorized live environment and bounded fixture inputs; A08–A15, A25 with request/scan IDs and hashes. |
| P2 — QA/user research | Run terminal/session coverage, 20 lifecycle repeats, 30-launch performance samples and five novice participants per OS. | Candidate installation access and real participants; A20, A22–A24, A29–A30. Preserve failures and assistance. |

Implementation work can proceed independently of external access: profile and
diagnostic flows, bounded verification, failure fixtures, and test orchestration
are not blocked merely because signing or the owner's Mac is unavailable.
Conversely, source changes and hosted tests cannot replace signing authority,
actual affected-device evidence, teammate access, or human task trials.

## Next review boundary

Append final candidate receipts without rewriting this cutoff's pending results.
Include exact package/source/binary identity, target/session, fixture revision,
first attempt and reruns. The namespace repair requires a new runtime identity;
do not attach its success to the defective `c10e3f993` source. Recalculate only
after freezing the denominator; treat
unknown hard gates as failures. Obtain independent review of reviewer-authored
changes and the final published-byte evidence before promotion. The candidate
can become useful for private testing before it becomes release-ready, but these
are separate milestones.
