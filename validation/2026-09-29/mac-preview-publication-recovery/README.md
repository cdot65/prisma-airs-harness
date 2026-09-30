# Mac preview publication recovery — September 29, 2026

Published `airs-harness` and `airs-harness-darwin-arm64` **0.1.4-alpha.1.mcp.1**
to `https://npm.cdot.io` under **mac-preview**. Every other tag is unchanged,
including **latest = 0.1.3**. No public npm publication occurred.

Runtime and packaging source: `11f676685f34289036b2885bea714439c0cd03b1`.
The executable SHA-256 is
`eda210ca422b810861e6b4f2d35598a7f955781d451d6304c9fc56c8d2b0b05a`.
Apple Silicon only; ad-hoc signed, without Developer ID signing or notarization.
Production SSO and owner-account acceptance are not claimed.

## Root causes and repair

[Run 4189](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/413)
failed with `ENEEDAUTH`: the added workflow used Jadzia's default npm configuration,
while earlier releases used a separate private publishing configuration. The
existing npm.cdot.io credential was verified and installed in the runner account's
configuration, preserving a backup. No credential values are retained here.

[Run 4192](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/416)
passed compilation and native/npm acceptance, then failed with `EPRIVATE`:
the new publish step attempted to publish unvalidated candidate manifests directly.
These remain truthful failed CI runs; final publication used their retained,
accepted bytes with the repaired staging path.

Fix `c1eb7185cac43ecc3f391d4107e9e4cc23a92e2f` in
[PR 56](https://git.cdot.io/cdot/prisma-airs-harness/pulls/56) checks authentication
before compilation and stages accepted candidates before publication. It reuses
the existing archive inspector/rewriter, binds CI acceptance to the exact source
and executable, restricts this path to npm.cdot.io/mac-preview, retains original
candidate provenance, and changes only release metadata. Both launcher and native
latest tags are checked. The launcher explicitly rejects non-Apple-Silicon hosts.
The notarized release path is unchanged.

## Verification

- Native CI suite: 61 tests, five skips, no failures; native Keychain and refresh
  fixtures passed. npm candidate suite: 43 tests, one skip, no failures.
- Six staging regression tests and two workflow boundary tests passed; Python
  lint, workflow YAML parsing, repository formatting and diff checks passed.
  Unrelated repository formatter edits were discarded.
- Fresh anonymous npm registry installation on Jadzia used an isolated prefix
  and empty cache. `airs --version` reports `0.1.4-alpha.1.mcp.1`.
- Registry tarball integrity and every installed package file match staged bytes;
  the archive rewriter checked unchanged runtime bytes against accepted candidates.
- All 12 installed feature checks passed: stdin key replacement, exactly one test
  request with the new key, Keychain readback, both update-auth entrypoints,
  replacement/SSO-switch confirmation cancellation, test-current behavior,
  fullscreen default, inline settings override, and refusal of global `model`.
- Separate installed native Keychain and managed CLI 7.2.0 acceptance passed.
  Synthetic state and keys were cleaned up; the temporary desktop test job was removed.

`STAGING-FINAL.json` binds original candidate hashes, staged tarballs and CI
receipts. `publication/PUBLICATION.json` records native-first publication.
`REGISTRY-VERIFIED.json` records final tags and exact installed-file checks.
`INSTALLED-SMOKE.json` and the registry Keychain/CLI receipts record post-install
results. The owner's saved application login and normal global installation were not changed.

The repaired workflow preflight passed as
[run 4195](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/419), with
`preflight_only=true` and no publication tag. Source checks and documentation
checks also passed on the repair commit (runs 4193 and 4194). The recovered release
was published from retained accepted artifacts; no second Rust rebuild was needed.
