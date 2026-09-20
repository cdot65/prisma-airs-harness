# Focus 4 early integration review

Status: read-only review of existing release contracts and approved evidence design. No product edits, Rust commands or numeric release score. Authors are implementing Git audit, upgrade/rollback and product-gate slices separately.

## Preserve existing authority

`verify_acceptance_set` already enforces all three native targets, observed target equality, package-set identity, source/tooling/packaging identity, receipt chains, retained output hashes, native keyring onboarding and signed/notarized installed Mac bytes. New product gate should call it for each requested phase and add stricter changed-feature/round-trip requirements; a cached aggregate passed flag is insufficient.

Candidate and registry phases remain distinct, including explicit verification_tooling_commit rules. Do not make candidate evidence substitute for anonymous registry evidence or silently use a revised validator without the existing identity field.

## Smallest required product-gate checks

1. Require exact newly delivered tests in each platform's verified test results: diagnostic report preview/copy/save and both lost-response/rejected-refresh cases. Every required ID must be present, executed, absent from skipped, and in a suite with zero failures/errors. Existing aggregate rules only reject an entirely skipped suite and therefore cannot establish that a particular changed feature actually ran.
2. Require native credential/history round-trip evidence per target. Existing config-only UPGRADE.json must not satisfy that row. Bind candidate version/hash, previous0.1.1 version/hash, restored rollback hash and actual exercised preservation fields.
3. Prefer retained stable0.1.1 SPEC/manifest as the authority for previous native hashes. Checking downloaded BUILD-INFO against downloaded bytes establishes consistency but not identity with the published baseline. The immutable stable evidence already supplies known all-three hashes; carry its identity rather than fabricate a new baseline.
4. Require every expected target once; reject wrong phase, stale source/tooling/candidate identity, missing rollback, skipped required case and failed native cleanup.
5. Preserve full GNU diagnostic scope/status/counts/source/tooling/raw artifact separately. Historical stable253 is not current-source coverage. If no current full run executes, explicitly report not-run with reason, keeping historical evidence in a separate labeled reference. Focused success cannot fill the full-workspace slot. Current unknown/product-related failures require review and block readiness; never import the0.1.1 six-case waiver.
6. No new stable authorization: product success cannot alter `latest` or claim production SSO/ServiceNow/attended refresh acceptance.

## Minimal meaningful product-gate tests

- Valid three-target acceptance plus actually executed required IDs and native round-trip passes.
- One real existing-contract receipt integration/tamper path, not only a mocked verify_acceptance_set return; a removed authority call must not silently make tests green.
- Missing and skipped required ID even while the surrounding suite passes.
- Wrong round-trip candidate hash, wrong previous/rollback identity, config-only/native-credential-absent evidence, wrong phase and missing target.
- Full failed GNU remains visibly failed; focused substitution, mismatched count/name set, stale-source current claim and unresolved product blocker reject. Not-run state is explicit, never converted to green.

## Git audit integration

The design correctly reads pinned object bytes independent of worktree/index and guards replacement/env redirection. Audit must reject symlink root/ancestors/output, gitlinks, unsafe paths, duplicate/omitted manifest members, stale receipt digest/size and missing ACCEPTANCE/dependency closure. A valid local file must not rescue absent committed bytes; dirty worktree after valid commit must not change the pinned audit.

Use a no-checkout clone test. Bound blob/member/aggregate reads before allocation. No network/fetch/index mutation. The audit establishes retrievability/integrity, not secret safety or product acceptance.

Avoid a receipt self-reference cycle: commit evidenceE, auditE, then retain that audit in later handoffE2 with explicit audited commit. Manifest cannot include its own digest; later handoff commit is not automatically the audited evidence commit.

## Upgrade/rollback integration

Stable previous-version support is a real prerequisite gap: spec accepts0.1.1 while old npm-upgrade parser only accepts alpha syntax. Retain supported historical command selection while accepting stable versions using current airs/env create. Reject tags/ranges/malformed input before mutation.

Round-trip must use isolated prefixes and unique native accounts, preserve actual credentials/conversation across candidate use and restored0.1.1, and verify known old native bytes after downgrade. Avoid force/uninstall and automatic config rewrites. New candidate-created state inspection is distinct from preserving old state. Do not assert downgrade support for future fields that were not exercised.

Numeric score waits executed required native/product/registry/evidence gates. Code completeness alone cannot finish PRD4 release delivery.
