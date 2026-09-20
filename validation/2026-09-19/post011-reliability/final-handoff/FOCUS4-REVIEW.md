# PRD4 final independent review — 9.5/10

Decision: the bounded release-reliability deliverable meets the 9/10 threshold. Published test version **0.1.2-alpha.1.mcp.1** is ready for owner testing on Linux x64, native Linux ARM64 and Apple Silicon. **latest remains stable 0.1.1**. This approval covers the executed test-release scope; it does not promote stable or claim a fresh attended company SSO/production ServiceNow acceptance.

## Identities and independently reproduced evidence

- Frozen runtime, packaging and validator source: `44872ebf4c125f06fa69bee7a486f8e7ab26c6af`.
- Closed evidence commit: `0b171dd54fd724634ec696c5a6ea60a9d83ef054`, root `validation/2026-09-19/post011-reliability/focus4-release`.
- Documentation source: `0235219441a75f9aa28c67312e47e6be1037ca90`; GitHub Pages run `35482389835`.
- No runtime, npm or script changes occurred between source freeze and the evidence commit; independently checked with Git.

Re-executed the frozen product gate separately for candidate and registry using the closed committed evidence layout. Both passed and each reproduced the corresponding retained product receipt exactly. This checks actual stage outputs, transitive receipt hashes, required test IDs, package/native identities, target observations, installed Mac signature/notarization and real stable upgrade/rollback evidence. It does not trust only an earlier aggregate passed flag.

All three native platforms executed the diagnostic report preview/copy/save case and both refresh fault cases; none was skipped. Registry installation records show exact-version anonymous installs with fresh caches and isolated npm configuration. Platform-specific and optional expiry skips remain disclosed. On every target, both candidate and registry acceptance exercised stable 0.1.1 → test version → exact stable 0.1.1 in one disposable prefix, preserving configuration, native inference/MCP credentials and a real resumed conversation while completing three synthetic MCP turns. No force/uninstall was used; native cleanup completed.

The independent quick lifecycle review also validated actual observation events, hashes and complete frozen tooling inventories on all three targets: each completed two inference and two MCP post-expiry cycles in the same installed process, with three completed turns and cleanup. This is bounded synthetic renewal coverage, not an implied new long-duration or production result.

Recomputed staged archive inventories and permitted metadata changes against accepted originals. Publication identity matches canonical frozen spec plus recomputed plan. Fresh anonymous registry metadata reads confirmed all four package integrities, `mcp=0.1.2-alpha.1.mcp.1`, `latest=0.1.1`, and every other protected tag unchanged. Native runtime payloads remain the accepted exact bytes.

## Current GNU diagnostics remain failed

Downloaded the actual completed full GNU run 260/internal 3724 artifact and checked source, tooling and full-workspace scope. Raw log SHA256 is `24ad97b0f4aa1fdca0cb39c0e898234cddb188998a51b6e88c2c45bfb4b2793e`.

Result: **18,334 run; 18,331 passed, three failed, 34 skipped**. The failures are remote shell snapshot-v2 pipe recovery, TTY recovery and the read-only sandbox profile-capture case. Current-source reviews document their actual assertions, default-disabled UnderDevelopment feature and guarded normal execution path. Related local/core/non-sandbox tests pass; all six V8 tests pass. The three experimental defects remain unresolved and excluded from default product readiness. Full workspace status stays failure. No historical 0.1.1 waiver was reused, expanded or relabeled.

## Documentation and retrievable proof

Independently reviewed installation/rollback, report privacy, startup restoration and capability statements. Verified all 17 retained documentation source-file hashes against the exact docs Git commit and all four retained log hashes. Local and CI logs each record 22 browser checks passing. An independent live GitHub API request confirms Pages run 35482389835 completed successfully at the documented commit. Fresh HTTP retrieval of the homepage and all 15 lessons matched the deployment receipt's exact artifact/live hashes: **16/16 pages**. Synthetic, attended and production acceptance remain distinct in the published guide.

Reproduced the new Git-object audit at the exact evidence commit: **349 manifest members, 56 stage receipts, 5,072,435 bytes**. Then created a new local **bare, shallow clone** with `--no-local`, one branch and no object alternates; no working tree or validation files existed. The frozen auditor passed again from that independent object database and produced the identical audit/inventory digest:

`ec866eb5d569128db225ae882eb3a4de5cac54d73d3339254116016f0fb35058`

This proves committed inventory and receipt closure independent of the original checkout. It does not substitute for the separate product, publication or secret-review checks. The follow-up handoff must retain this review/audit with the explicit audited commit to avoid a self-reference cycle.

The committed cleanup evidence records removal of only the approved task paths, preservation of compiler caches/VM/owner profiles, and restoration of the unchanged 100 GiB Mac guard at 101.7707 GiB. Private publication configuration removal is recorded. Raw CI logs retain their original hash-bound bytes, including documented timestamp whitespace; no source whitespace correction is needed.

## Rubric: 3 + 3 + 2 + 2

| Dimension | Score | Evidence and remaining limitation |
| --- | ---: | --- |
| Completeness | 2.9/3 | Stable parser, real all-target rollback, explicit gates, registry publication, deployed guide and clean-clone evidence audit are complete. Release coordination still has several manually assembled handoff steps. |
| Capability | 2.9/3 | Exact native and anonymous registry behavior passed on all three targets, including credential/history preservation and cleanup. Claims remain limited to the exercised synthetic scenarios and supported platforms. |
| Best practices | 1.9/2 | Immutable source/package/proof identities, native signing, private disposable credentials, protected tags and honest failed-workspace reporting are preserved. Experimental remote snapshot-v2 defects remain an explicit follow-up outside the approved path. |
| Optimization | 1.8/2 | Existing validators, caches and isolated fixtures were reused; source froze once for release, checks are bounded, and only owned task artifacts were cleaned. Cross-host orchestration and evidence assembly can still be simplified. |
| **Total** | **9.5/10** | **Approved for the delivered test-release scope.** |

No unresolved mandatory product, registry, documentation or committed-evidence gate remains. Retaining this final review and independent verification receipts in the follow-up handoff commit is the remaining bookkeeping action owned by root.

## Independent receipts retained for the handoff

- `FINAL-GIT-EVIDENCE-AUDIT-INDEPENDENT.json`
- `CLEAN-CLONE.json` and `CLEAN-CLONE.log`
- `FINAL-GIT-EVIDENCE-CLEAN-CLONE.json`
- `FINAL-CLOSED-CANDIDATE-INDEPENDENT.json`
- `FINAL-CLOSED-REGISTRY-INDEPENDENT.json`
- `FINAL-DOCS-INDEPENDENT.json`

Earlier candidate, registry/publication and quick-renewal independent checks are already in the closed evidence commit. This final review makes no new runtime or publication mutation.
