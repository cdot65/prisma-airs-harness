# Independent focus 1 review — initial

**Current outcome after follow-up review: 9.5/10 for the bounded baseline focus.**
See the appended follow-up review. The initial 8.5/10 audit below is preserved.

Reviewed 2026-09-19 by the onboarding/review subagent. Scope is the bounded
baseline/reproduction deliverable, not runtime fixes or production acceptance.

**Initial score: 8.5/10. Do not open focus 2 implementation gate yet.** The main
remaining work is a complete, evidence-classified historical failure summary;
no new whole-workspace run is required to close this review.

## Evidence reviewed

- Vault PRD `Work/AIRS focus 1 - Reliability baseline and reproduction.md`.
- `BASELINE.json`, doctor and MCP logs; recalculated both log SHA256 values and
  confirmed they match the manifest. Each suite actually ran four passing cases.
- `doctor-backend-reproduction.json` and its fixture contract: native save/read
  with key syscalls denied succeeds, as does doctor. The suspected backend
  divergence is disproven for this controlled case, not generally for every host.
- `onboarding-baseline/RESULT.json`, rendered screens, and executable reproduction:
  selected `staging` receives HTTP403 while saved default remains `work`; displayed
  doctor recovery commands omit the selection. No owner state was touched.
- `auth-baseline/coverage.md`, `coverage.json`, and
  `primary-cleanup-masking.md`: source-inspected coverage is explicitly distinct
  from newly executed tests. Primary-error masking is a source finding with a
  meaningful proposed regression, not a claimed executable failure.
- `workspace-triage/skills-ancestry.json`: the identical retained skills binary
  fails with `/tmp` and passes with isolated home TMPDIR. This supports fixture
  ancestry contamination without deleting `/tmp/.git`.
- `workspace-triage/old-exec-server-dispatch.json` and completed
  `exec-server-focused.log`: old direct probe and fresh `just test` both reproduce
  `Unrecognized option: 'listen'`. The fresh run reports 0 passed, 1 failed,
  2 filtered/skipped, not an unfinished compile or successful suite.
- `workspace-triage/app-server-probes.json`: omitted-skills count remains
  reproduced (15 versus 7); the zsh test reports success **after skipping its
  actual behavior because dotslash/zsh could not be fetched**. Treat it as an
  unavailable-fixture scenario, not a repaired historical timeout.
- The runtime implementation worktree is still clean at
  `a826d22a63d9d6889157477a0109e242871796f5` at review time. Published runtime is
  separately identified as `19e5bcee52f6b21e22441a870db0a610fa8b3d4b` in BASELINE.

## Score

| Dimension | Score | Reason |
| --- | ---: | --- |
| Completeness | 2/3 | Auth/onboarding baseline and coverage inventory are useful; historical failure groups lack one consolidated current classification and sandbox representative evidence is not yet present. |
| Capability | 2.5/3 | Eight current executable fixtures, a successful disproval, a reproduced onboarding defect, and fresh exec-server reproduction; source-only auth findings still need failing regression before edits. |
| Best practices | 2/2 | Isolated state/secret service, truthful deferment and execution labels, preserved owner state, exact native/log hashes, no speculative backend rewrite. |
| Optimization | 2/2 | Cached targeted checks and representative comparisons; no need to rerun the entire historical workspace for this baseline. |

## Mandatory gates

1. **Pass:** no observed owner credential or session mutation; temporary fixture
   state only, no live gateway/ServiceNow assertion.
2. **Pass for reviewed baseline evidence:** source, published runtime version and
   native digest are separately recorded; doctor/MCP log hashes match.
3. **Pass with required wording:** source inspections, executable results,
   hypothesis disproof and deferred live checks are explicitly distinguished.
   The app-server zsh return code must not be summarized as actual coverage.
4. **Pass for planning, binding for the next focus:** every proposed correction
   must have an observable regression requirement. Runtime implementation may
   start only after its meaningful regression fails against the prior behavior.
   Preserve this condition in focus 2/3 handoff and final review.

## Close the remaining review gap

Publish one workspace triage table containing all historical groups and counts
(146 exec-server, 1 sandbox, 1 skills ancestry, 2 app-server), representative binary
hash/source, command/log link, current result, confidence and next action.

- Exec-server: record fresh reproduction as failed; don't classify all 146 tests
  as newly run or proven individually identical.
- Skills ancestry: record same-byte fail/pass comparison and unchanged `/tmp/.git`.
- App-server skills: record count mismatch as reproduced, not solved.
- App-server zsh: record fixture-unavailable skip and historical timeout still
  unresolved; source history can explain why this run cannot adjudicate it.
- Sandbox: add a bounded representative rerun or concrete read-only policy/helper
  evidence tied to the actual failing test; an explicit unresolved classification
  is acceptable for baseline completeness, but do not assert proven contamination
  solely from the old receipt.

Once this table is present with honest sandbox/zsh limits, baseline completeness
can reach 2.5–3/3 and the scoped baseline can pass >=9 without fixing unrelated
upstream fixtures. Store the consolidated review outcome and regression-before-
implementation requirement. Recheck only new evidence; do not repeat good tests.

## Required regression shapes for next focuses

- Primary + cleanup failure: fake read/save failure followed by delete failure
  preserves both bounded operation categories, pending metadata, prior binding,
  and typed recovery; backend/credential canaries never appear. First run must
  demonstrate current behavior fails the preservation expectation.
- Native-store guidance: an unavailable/unknown backend must not assert locking
  is the proven cause. Cover missing, locked and unavailable categories with
  accurate remediation while retaining no-plaintext guarantee.
- Environment recovery: two environments, default `work`, selected `staging`;
  denied/cancelled login displays exact staging retry/start commands and preserves
  default/credentials. Published-binary reproduction already establishes the
  bug; convert it to a maintained acceptance test before modifying behavior.
- Node runtime guard: first demonstrate an unsupported-version launcher fixture
  reaches child execution under old code (or fails for an unrelated dependency),
  then verify actionable early rejection and supported-boundary behavior. Do not
  claim actual Node18 execution unless that runtime is actually available.

## Follow-up review — completed baseline classification

Re-reviewed the completed `workspace-triage/README.md`, `SUMMARY.json`, all
`SHA256SUMS` entries, updated `BASELINE.json`, and installed-regression log on
2026-09-19. All 19 triage checksum entries passed; the three baseline fixture log
hashes match. Added installed acceptance is **46 passed, 1 macOS-only skip**,
alongside the previous 4 doctor and 4 MCP passes: **54 executed passing cases,
1 platform-specific skip**. These are distinct from the inventory of 54 existing
source tests; do not conflate the two counts.

The completed table accounts for all 150 historical failures with concrete
representative evidence and explicit limits:

- 146 exec-server failures share the reproduced embedded dispatch failure; a
  fresh focused test and retained direct probe fail, and a minimal constructor
  fixture demonstrates empty argv before main on this musl toolchain. This is
  strong mechanism evidence, not 146 newly executed cases.
- The skills test has exact-byte fail/pass ancestry evidence.
- The sandbox case still fails even with isolated TMPDIR because the compiled
  helper remains under denied `/tmp`. An outside-/tmp rebuilt helper rerun was
  not performed and is accurately disclosed.
- App-server skill omission mismatch is reproduced; bundled inventory explains
  the changed count at source level without removing product skills.
- The initial zsh self-skip is superseded by a real rerun with dotslash available.
  The selected existing ELF requests absent `/lib64/ld-linux-x86-64.so.2`;
  direct invocation fails ENOENT and the actual integration reproduces failure.
  The table no longer presents the self-skip as successful behavior.

This resolves the initial completeness blocker. The full historical suite remains
non-green, no old-source comparison is invented, and the owner-specific Ubuntu
and live production acceptance remain deferred. None blocks the scoped baseline
deliverable when reported honestly.

| Dimension | Final baseline score | Basis |
| --- | ---: | --- |
| Completeness | 3/3 | All requested baseline areas have executed results or explicit source/gap labels; every historical failure group is classified. |
| Capability | 2.5/3 | Reproducible onboarding defect, native-backend hypothesis disproof, 54 passing installed fixture cases, representative historical failures, and actionable next regressions. The new auth masking regression is not yet executed to completion. |
| Best practices | 2/2 | Isolated credentials/state, verified evidence hashes, honest provenance/deferred scope, no unsupported production or full-suite claim. |
| Optimization | 2/2 | Bounded targeted checks, reused caches, same-byte comparisons; no redundant whole-workspace rerun. |

**Decision: focus 1 meets its >=9/10 gate at 9.5/10.** This score is not contingent
on the pending auth RED run because the baseline PRD requires a concrete regression
requirement and truthful source classification, not a completed fix. It does not
authorize skipping that RED requirement in focus 2.

At follow-up, the only application-worktree diff is **47 added test lines** in
`airs_credential_transaction_tests.rs`; production code remains unchanged.
`auth-baseline/regression-red.log` is still compiling, so no failed/passed outcome
is credited. The next focus may prepare/review the regression and finish its run,
but **must not change authentication runtime behavior until this meaningful
regression fails against old production code for the intended reason**. Apply the
same rule to new native-store guidance and Node behavior. The already-executed
nondefault onboarding reproduction establishes that separate behavior defect.

One provenance clarification: `BASELINE.json`'s `source_clean:true` describes its
initial frozen capture. It must not be interpreted as claiming the worktree still
has zero changes after the test-only regression was added. Keep source and diff
status timestamped in subsequent focus receipts.
