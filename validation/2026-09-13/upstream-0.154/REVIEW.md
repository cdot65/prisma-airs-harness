# Codex 0.154 integration review

Implementation is complete in `integrate/codex-0.154-candidate`; release acceptance is **NO-GO, currently 89/100**. This is the implementer's evidence assessment, not an independent review. The accepted contract requires at least 90/100 **and every mandatory gate**, independent review and owner acceptance. No package was published, and original main remains at `1cdc152631d4c1e960435f1630373fef29d34a64`.

## Result

The candidate merges stable Codex `rust-v0.154.0` (`6b9826e3aa83b1a5947db50f4332cb9c65f1b340`) and preserves the two separate upstream backports for macOS terminal-input injection denial and hook-stdin timeout handling. All 21 merge conflicts were reconciled; the 47 overlapping paths are inventoried for review in [OVERLAP.json](OVERLAP.json).

AIRS retains ownership of the gateway, optional default model, qualified explicit routes, reasoning capabilities, serial tool requests, credentials, per-environment state, independently authenticated MCP and the required Prisma CLI. Runtime feature constraints protect existing homes as well as fresh setup. Realtime rejects an AIRS provider before network initialization. Guardian V2, hosted services, Code Mode, voice and remote model discovery remain deferred. Worktrees and inline questions are opt-in, with an owner-controlled capability catalog required for inline questions.

CLI **5.2.0**, SDK **0.28.0**, and all **eight Prisma skills** are preserved. The eight skill files are byte-identical to alpha.11; see [SKILLS.json](SKILLS.json). The upstream skill-budget test now counts these eight retained skills rather than removing them to match an upstream-only expectation.

## Frozen artifacts

Both native artifacts use runtime source `19ce13981e92079ec29889896836d58d070cf106`. Later commits change tests, validation tooling, owned workflows and review records; [RUNTIME-EQUIVALENCE.json](RUNTIME-EQUIVALENCE.json) records the comparison.

| Target | Native SHA-256 | Status |
| --- | --- | --- |
| Linux x64 musl | `41301da84a0e9917077809e2fb48062f841c376fe58043545866b58bf7d00c89` | Private native and installed bundled npm acceptance passed |
| Apple Silicon | `63500ffa8d3c7d47833242dc7dee908e626d85df2ebe88eec7d74f617a94138d` | Same frozen artifact passed native/npm/Keychain revalidation; ad hoc signed only |

The Linux build uses release optimization with debug=0, LTO disabled and 16 codegen units. Mac additionally uses CLI opt-level=1, as recorded in its immutable build receipt. These are separately tested platform artifacts, not a claim of identical executable bytes.

Native/package records are under [receipts](receipts). Local private artifacts are retained in `/var/tmp/airs-alpha12-candidate`; Apple Silicon acceptance and download provenance are in [owned run 34740660670](https://github.com/cdot65/airs-harness/actions/runs/34740660670), using the immutable executable from [build 34739344591](https://github.com/cdot65/airs-harness/actions/runs/34739344591).

## Validation

- Affected Rust suite: **10,418 passed**, 24 skipped, two passed on retry.
- Frozen Linux native: **42 passed**, no skips. Installed Linux: **42 passed**, plus the subsequent **one-test concurrent-writer acceptance**. The latter rejects a second writer without inference, then resumes after the original exits.
- Apple Silicon native: **45 run, 43 passed, two skips**. Installed npm: **43 run, 42 passed, one Linux-only skip**. The native run includes the actual terminal-input injection control/denial regression. No skipped case is credited as Mac behavior.
- All 29 full native OIDC/MCP checks pass, including browser/device authentication, two resource audiences, two real expiry cycles, cross-user denial, real scans, logout and same-user reauthentication. Disposable users were removed; shared client availability and scanner policy were unchanged.
- Fresh gateway enforcement passes all 20 cases, requiring applicable completed scan/profile identifiers. Persisted input/output scans are verified for the disposable native-only signed subject. The doctor's `x-client-request-id` is not the gateway's `SessionID`; the failed direct-ID correlation attempt and the explicit subject-based discovery provenance are retained.
- Installed live default → explicit → default workflow passes seven turns, independent local tests and three completed MCP scans. Root-agent inline question/answer/draft preservation passes. An earlier model-delegation failure is retained; no subagent asynchronous-question support is claimed.
- Installed managed-CLI contract and live embedded-skill workflow pass. Empty trusted scanner/management credentials are reported truthfully; no real management mutation or unperformed red-team/DLP/model-scan campaign is claimed.
- Alpha.11 native state upgrade, executable relocation and rollback pass with exact binary hashes. Rollback runs the old binary with both newer fixture executables absent, retaining identity and history. Installed candidate launchers resolve to the same verified native bytes; this is not a published package upgrade or owner-device acceptance.
- Linux, Apple Silicon and Windows seven-process native-store checks pass. Windows also passes 26 identity/keyring tests; no Windows distribution was added.
- Configuration and stable/experimental app-server schemas were regenerated. The authentic owned GNU-host Bazel refresh matches Cargo.lock. Owned npm platform checks pass. No inherited workflow file was activated; the preexisting platform-managed Dependabot entry is distinguished in [OWNED-WORKFLOWS.json](OWNED-WORKFLOWS.json).

The initial **complete musl workspace run** finished with **17,989 passed, 156 failed, 34 skipped**. All failures remain recorded in [RUNS.json](RUNS.json): 146 execution-server cases depend on constructor-based test dispatch, one bundled zsh fixture needs GNU shared libraries, six TLS cases inherited CA settings from test-runner startup, one ancestry case encountered an existing `/tmp/.git`, one V8 test needed its feature aligned with the pinned sandbox-enabled archive, and one skill-budget expectation omitted the eight AIRS skills. The last nine cases are resolved: all 92 HTTP-client tests pass with immediate child-environment isolation, and the ancestry/V8/skill-budget cases pass. The execution-server and GNU-zsh cases still require the **owned GNU complete workspace run**, which is now running on Forgejo. A passing affected suite is not substituted for that missing full-suite result.

The original GitHub GNU retry was blocked by the Actions budget. The owner then
moved canonical source and CI to Forgejo; GitHub is a private mirror with Actions
disabled. GNU run 3201 executed 18,180 cases but failed (17,772 passed, 407
failed, one timeout). The runner lacked bubblewrap. Run 3244 includes that
prerequisite and is executing; helper-process failures also exposed inherited
Cargo test-runner configuration, now corrected for follow-up run 3269. No
complete GNU suite pass is claimed.

## Evaluation and remaining gates

| Area | Maximum | Current credit |
| --- | ---: | ---: |
| Inference and scans | 30 | 30 |
| Identity and authority | 20 | 20 |
| MCP | 15 | 15 |
| Managed CLI and skills | 15 | 15 |
| Sessions and inline questions | 10 | 9 |
| Upstream maintenance | 5 | 0 pending full suite |
| Release surface | 5 | 0 pending required signing |
| **Total** | **100** | **89** |

G1–G7 currently have implementer evidence; G8 remains blocked. Maintenance credit is withheld while full GNU validation is blocked. One inline-question point is withheld for model delegation friction. After full-suite closure, one maintenance point remains reserved for independent review of the existing RSA advisory's applicability.

The current RustSec audit is **not clean**: RSA 0.9.10 remains in the unchanged OIDC dependency chain. The AIRS client uses public-key signature verification; no application RSA private-key signing or decryption was found. The scc 2.4.0 workspace warning is outside the Linux CLI normal/build closure. Both findings and their scope are retained in [DEPENDENCY-SCOPE.json](DEPENDENCY-SCOPE.json); this assessment does not suppress the audit result.

Mac Developer ID signing and Apple notarization are now available and verified
on Jadzia. The fresh Forgejo build and signed-package receipts supplement the
historical frozen-artifact ledger below. That ledger still binds the original
unsigned Mac hash; its score must not be promoted by silently substituting newly
compiled or signed bytes into historical evidence. Full GNU validation,
independent review and owner hands-on acceptance remain pending.

Reproduce the current gate decision from the repository root:

```sh
python3 scripts/evaluate_airs_upstream.py validation/2026-09-13/upstream-0.154/LEDGER.json
```

The expected current result is nonzero exit status and `NO-GO`. [LEDGER.json](LEDGER.json) binds evidence references and artifact hashes; [EVALUATION.json](EVALUATION.json) records the computed result. Receipt integrity is not a substitute for independently reviewing whether the evidence proves each claim.

Owner infrastructure correction: candidate packages are now staged together for Verdaccio (`https://npm.cdot.io`) under the unscoped `airs-harness` name. The isolated registry installation and all 43 Linux native acceptance checks pass. Both native payload hashes remain unchanged; Apple Silicon acceptance of this newly combined npm staging has not been rerun. No publication occurred. See the adjacent `receipts/verdaccio-candidate-*` evidence.

## Forgejo cutover

The owner moved source and CI to [cdot/prisma-airs-harness](https://git.cdot.io/cdot/prisma-airs-harness). GitHub is now a private push mirror with Actions disabled. All 13 branch heads and five tags matched before mirror setup; dedicated SSH mirroring has succeeded. Forgejo package contracts pass on main and the candidate. The full GNU suite is running on Forgejo; Jadzia runner 74 is online and its frozen native/Keychain/Verdaccio acceptance passed (run 30). Fresh Mac compilation and native acceptance passed on run 52; Developer ID signing and notarization passed on run 50. The prepared Windows identity workflow still needs a Windows runner. Earlier budget and namespace blockers above are historical. See [the cutover receipt](receipts/forgejo-cutover.json) and [runner operations](../../../../FORGEJO.md). No new runtime score or release acceptance is claimed.

## Fresh Forgejo signed-package acceptance

Forgejo [build 52](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/52)
compiled the same alpha.12 runtime source on Jadzia and passed native acceptance.
[Package E2E run 79](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/79)
then signed that compilation with Developer ID team `G5QLZ5A8TA`, received Apple
notarization acceptance, built bundled npm tarballs and passed fresh installed
acceptance on both platforms. Mac passed 42 of 43 tests with one Linux-only skip;
Linux passed all 43. Managed Prisma CLI acceptance passed on both platforms;
installed Mac signature/online notarization and Keychain lifecycle passed.

The signed Mac binary is
`397f8a02f73e7512d19537f27fd73baa7bc9c8c9c95a13064a411c19347eec8d`.
Linux reuses the verified source-matched musl artifact above. No new live gateway
campaign or registry publication was performed by this package workflow.
[The final receipt](receipts/forgejo-package-e2e.json) binds build/signing provenance,
all three tarball hashes and both platform logs. This closes the package signing
and installation task; the complete GNU suite and independent/owner review gates
remain open. The historical ledger is not rebound to new bytes without reviewing
its artifact-specific evidence.
