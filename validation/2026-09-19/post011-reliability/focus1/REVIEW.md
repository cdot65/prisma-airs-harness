# Independent PRD1 review — first-use reliability

Reviewer: post011_diagnostics_plan (did not implement PRD1).
Reviewed: product diff in airs-post011-reliability-20260919, approved PRD1, launcher test log, before/repair receipt and executable reproduction script, omission investigation, packaged Ubuntu helper receipt/log and runner script. Read-only review; no code edits or test runs by reviewer.

## Decision

**Pass for the bounded PRD1 implementation: 9.4/10.** No functional/privacy blocker found. This is approval to complete PRD1 and proceed to PRD2 after the root finishes mandatory formatting and restores unrelated formatter churn. It is not a package publication or all-platform candidate acceptance decision.

At review time `just fmt` was still in progress and19 unrelated baseline formatting files were visible. These must be restored before committing; only launcher.js, launcher.test.js and GETTING-STARTED.md belong to this implementation. Root already identified this cleanup. `git diff --check` passed at review time.

## Verified findings

- The defect is reproduced against an anonymously installed published0.1.1 local project with optional dependencies omitted: native package absent, exit1, misleading instruction to install a different supported release. Global omission did not reproduce; OMIT-INVESTIGATION preserves the contrary evidence and explains the observed npm behavior instead of claiming universal omission.
- Updated source changes only the missing-dependency message. It preserves supported-platform validation, resolution ordering, bounded failure/no child execution, version matching and existing bundled CLI dispatch. It names the installed exact package/version and `--include=optional`, while preserving registry and global/local scope.
- Source launcher SHA256 `f19e258f731ab2bf89f4d07890d018945720819caff784ff7959e195bee99f73` matches REPAIR.json. That receipt accurately labels the candidate as a source launcher copied into a published0.1.1 installation; it does not pretend this updated launcher has been published.
- The reproduction script installs the same pinned0.1.1 package into the same project using the same registry. Both user/global npm config hashes remain unchanged. After repair, `airs --version` reports0.1.1 and bundled CLI reports7.0.0. No owner account or keyring is used.
- The changed behavioral test checks actual process exit/stdout and meaningful recovery guidance. All31 launcher tests pass with zero skips/failures, including runtime bounds and portable manifest boundaries. Node18 remains a metadata-injection test, not a claimed fresh real Node18 runtime.
- Exact packaged Ubuntu helper SHA256 `7879c3d869093236fd818ac7bb5aa45a9feeca716ca44122f9fa14384db3c44a` passed six isolated cases without a version override. Log includes new encrypted store, reuse, no-prompt check, quoted spaced path, wrong-password preservation, and subsequent successful unlock. It explicitly uses disposable home/prefix/D-Bus/keyring state.
- GETTING-STARTED now explains exact-version repair, `--include=optional`, and correct global/local scope. Existing ordinary-install guidance does not recommend unnecessary optional flags for normal installs.

## Rubric

| Dimension | Score | Evidence / limits |
|---|---:|---|
| Completeness |2.8/3|Ranked defect, real before/after, precise docs and packaged helper gap closed. Candidate publication/site synchronization is correctly reserved for integrated release work.|
| Capability |2.8/3|Actual local installed failure/repair passes; supported and unsupported paths remain distinct. No fresh Mac/native rebuild is claimed for a JS copy-only change; prior stable acceptance is reused.|
| Best practices |1.9/2|Minimal AIRS-owned diff, no automatic fetch/global mutation/credential use, honest source-vs-published receipts. Restore unrelated formatter churn before commit.|
| Optimization |1.9/2|Existing tests/fixtures and native bytes reused; no redundant Rust build or speculative auth rewrite.|
| Total |**9.4/10**|All chosen-scope behavioral/privacy checks passed; final formatting cleanup is the remaining mechanical completion gate.|

## Non-blocking handoff notes

1. Carry source-vs-published distinction into final report. PRD1 is source-ready, not remotely installable as a new release yet.
2. Preserve the global npm omission observation; public text must not imply every global --omit invocation loses its native dependency.
3. Copy only the bounded receipts/logs and reproduction sources into tracked release evidence later; do not copy npm caches or private npmrc files.
4. Update the PRD1 progress paragraph after helper completion; it still says those cases are undergoing execution.
