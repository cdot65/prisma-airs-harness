# Final publication and platform review

**PASS: `airs-harness@0.1.0-alpha.22.mcp.3` is published and its three native
platform distributions passed fresh anonymous registry acceptance.** This is an
owner-authorized test release. Stable promotion and attended real-account
acceptance are not claimed.

## Independently verified

- Original specification, runtime commit `7eba6bf33c13b669a6c28b53ec658b3c9c3be5c2`,
  candidate tooling `a4f85a3de4a3337407fecd1e2eeeb3182573e8cf`, packaging commit
  `94b45c17bd1182e789772f3f88685c5486cda7d8`, candidate receipts and staged plan
  remain identical to the prepublication baseline.
- All 136 original tooling files and all 137 registry-repair tooling files were
  checked against their declared committed git bytes and transfer hashes.
  Registry-only revision `0f625cd047a19e559f0500dd0d9f0b131c791e78` is explicit in
  the new receipts; it does not rewrite candidate provenance or published bytes.
- The publication receipt matches the exact staged plan and records Linux x64,
  Linux ARM64 and Apple Silicon natives before the launcher. Independent
  anonymous registry metadata reads confirm each immutable archive integrity and
  `mcp` selector. **Every non-`mcp` tag** matches the prepublication snapshot.
- Frozen tools reverified complete fresh registry evidence for all three native
  targets. Installed native hashes, source/version, managed CLI 7.0.0, SSO/key
  fixtures, terminals, MCP manager, doctor, command guidance, upgrade and Mac
  signing/notarization checks are bound to retained output hashes.
- Supplemental upgrades from onboarding.4 pass on all three targets, covering
  in-place npm upgrade, known manual-command preservation, and a legacy command
  earlier on PATH. Configuration and legacy targets are preserved; exact new
  native hashes match the release. The immutable main spec still names mcp.2 as
  its separate previous-version baseline.

| Native target | Installed regression suite | MCP suite | Doctor suite |
| --- | ---: | ---: | ---: |
| Linux x64 | 48 run, 1 platform skip | 4 run, 0 skipped | 4 run, 0 skipped |
| Linux ARM64 | 48 run, 1 platform skip | 4 run, 0 skipped | 4 run, 0 skipped |
| Apple Silicon | 48 run, 2 platform skips | 4 run, 0 skipped | 4 run, 2 platform skips |

Skipped cases are explicitly platform-specific: Mac Seatbelt on Linux, and
Linux credential-service/session-bus cases on macOS. They are retained as skips,
not counted as passed tests.

## Real Node 18 supplement

The official Node.js download and its official SHA-256 checksum were verified.
Actual **Node 18.20.8, bundled npm 10.8.2, on native Linux ARM64 Debian** then ran
the published launcher's unchanged entrypoints. Eight entrypoint cases passed
both plainly and with a separate observation preload: startup, native version,
managed CLI, Bash/Zsh completion, legacy alias, migration check and the direct
managed wrapper. All 16 runs returned the actionable Node requirement before
child dispatch, with no isolated harness state created. The preload observed
all public `child_process` dispatch methods; it did not change the runtime
version or application code. Entry hashes match the published staged launcher.

This does not claim an npm installation under Node 18, or the exact originally
reported Node 18.19.1/npm 9.2.0 combination. Main native registry acceptance used
supported Node environments. The official x64 Node 18 archive was also verified
but could not execute directly on the local Alpine/musl host; no x64 Node 18
execution is claimed. Both downloaded archives remain in scratch, not the repo.

## Honest recovery history

Initial registry acceptance failed because npm legitimately changes executable
bits on declared package `bin` files. The repaired validator first verifies all
file bytes, then permits npm's executable mode only for safe targets declared by
those verified package manifests. Non-bin mode changes, modified manifests,
missing/traversing targets and changed bin bytes still fail. A real npm fixture
captured RED/GREEN behavior, and the revised native matrices ran afresh. Failed
receipts remain separate; the diagnostic reuse of the failed Linux prefix is
not presented as fresh acceptance.

## Rubric and remaining gates

The bounded **publication/platform review scores 9.5/10**: completeness 3.0/3,
capability 2.8/3, best practices 1.9/2, optimization 1.8/2. Native distribution,
immutable publication, anonymous installation, supported workflow fixtures,
upgrade preservation and unsupported-runtime guidance have concrete evidence.
This is an evidence review, not sole approval of every validator implementation
this reviewer helped author; root and delivery performed the complementary code
reviews.

Full focus 5 completion remains pending the separate **live Pages check,
durable repository/vault handoff, and owned-fixture cleanup**. The owner Ubuntu
credential-session investigation and attended human SSO, real workspace-key and
ServiceNow acceptance remain explicitly deferred. The full Rust workspace is
not newly declared clean by these installed-package checks.

Primary receipts: `FINAL-PUBLICATION-REVIEW.json`, `STABLE-UPGRADE-REVIEW.json`,
and `node18/ACTUAL-NODE18-SUPPLEMENT.json`. No remote container work remains for
this reviewer; required Node evidence and the official archive have been copied
and verified locally before root's owned-container cleanup.
