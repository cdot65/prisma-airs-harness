# Post-0.1.1 reliability test release

Version **0.1.2-alpha.1.mcp.1** is published under `mcp` at
`https://npm.cdot.io`. All four packages retain `latest=0.1.1` and their other
protected tags. This is a test release, not stable promotion.

Runtime, packaging and acceptance tooling are frozen at
`44872ebf4c125f06fa69bee7a486f8e7ab26c6af`. The preceding stable binaries and
source are pinned by `PREVIOUS-SPEC.json`, copied from committed 0.1.1 evidence.

The release adds exact-version native-package repair guidance, a bounded
redacted diagnostic report, and a usable shell recovery command when credential
refresh prevents entering the TUI. It also adds actual stable upgrade/rollback
validation, an explicit product gate, and an audit of committed evidence.

## Executed acceptance

Exact candidates and fresh anonymous registry installations passed on Ubuntu
x64, native Linux ARM64, and signed/notarized Apple Silicon. Every platform
executed the diagnostic-report case and both consumed-refresh fault cases.
The stable 0.1.1 → candidate → exact 0.1.1 round trip preserved the isolated
configuration, native credentials and conversation history, with three real
MCP turns. Separate candidate renewal checks passed two inference and MCP
expiry cycles on every platform. Test-account cleanup is recorded.

`PRODUCT-CANDIDATE.json` and `PRODUCT-REGISTRY.json` bind these results to their
source, tooling, package and native-byte identities. The acceptance trees retain
the stage outputs, logs and dependency receipts needed to reproduce validation.
The source-object inventory verifies the frozen validator and packaging inputs.

## Full workspace result and limits

The current full GNU workspace run recorded **18,331 passed, 3 failed, 34
skipped**. It remains a failed run. Current reviews retain all three defects in
experimental remote shell snapshot V2, which is disabled by default. They do
not claim the feature is fixed or inherit the earlier stable-release waiver.
See `workspace-260/WORKSPACE.json` and its individual failure reviews.

Installed authentication and MCP checks use isolated synthetic HTTPS services
and native credential stores. They do not assert a new attended company SSO or
production ServiceNow acceptance. The owner's earlier real-account acceptance
remains a separate historical record.

The initial Ubuntu terminal check failed because zsh/fish were absent. That
failure is retained; unmodified distribution shell binaries were prepared in a
private task cache and the unchanged candidate acceptance resumed successfully.
Owner environments, system packages and compilation caches were preserved.

Documentation receipts retain the exported content, validation results and
actual deployment verification. `SHA256SUMS` covers this complete evidence set.
The immutable Git audit and final independent score are retained in a subsequent
handoff commit to avoid a self-referential evidence hash.

Raw logs retain their original bytes, including trailing whitespace in GitHub
Actions timestamp lines. Those evidence-only whitespace warnings are preserved
rather than altering a hash-bound deployment log. After removing redundant test
copies, the unchanged Mac free-space guard passes at 101.77 GiB; compiler caches
and the native ARM VM/container remain intact.

September 20 portability correction: the source-commit receipt is now named
`RUNTIME-SOURCE`, preserving its exact bytes. Its former name, `SOURCE`, collided
with the `source/` directory on case-insensitive Mac filesystems. `SHA256SUMS`
records the new path; the original committed release evidence remains available
at its historical Git revisions. No runtime or acceptance receipt was changed.
