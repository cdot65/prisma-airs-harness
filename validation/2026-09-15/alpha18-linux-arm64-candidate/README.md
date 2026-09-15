# Alpha.18 Linux ARM64 candidate — September 15, 2026

The owner reported that an unversioned install from `https://npm.cdot.io` fails on
an Omarchy Linux ARM64 VM hosted on Apple Silicon. The registry default was
alpha.14; both alpha.14 and alpha.17 declare only Linux x64 and macOS ARM64 native
packages. Enabling optional dependencies cannot install an undeclared target.

The alpha.18 candidate includes all three native platforms and a launcher that
distinguishes an unsupported target from a missing installed optional dependency.
Runtime source is frozen at `a7d03cb60275d91ffd46a67c3a1630e2d804db99`.
The only Rust difference from the released alpha.17 source is the version
constant; the authentication recovery implementation and 30-minute idle policy
are unchanged. Docusaurus is not part of this change.

## Validation

Launcher tests: 16 passed. Packaging tests: 13 passed. Home-directory tests:
8 passed. Formatting and the Forgejo package-contract workflow passed.
Linux x64 native acceptance ran 44 tests: 42 passed and two expected skips for
macOS-only and bundled-CLI checks on the unbundled Linux binary.

Linux x64 npm installation passed. Linux x64 and Mac npm upgrades passed from
both alpha.14 and alpha.17, including configuration preservation and the existing
legacy-command migration cases.

The ARM64 musl binary was cross-compiled with Zig and passed a QEMU version
probe. This is not installed ARM64 acceptance. The Mac binary is signed with
hardened runtime and notarized. Attached receipts bind the native bytes to their
source commit; the npm receipt records all four candidate tarballs.

## Outstanding host acceptance

The target VM is `10.0.3.148`. SSH access has not succeeded, so its existing global
installation has not been changed. The exact ARM64 npm candidate must pass native
installed acceptance before publication under the repository release policy.
All candidate packages remain private/unpublishable. No registry publication or
tag promotion is recorded here.

Prepared host checks reproduce the original alpha.14 launcher failure in an
isolated npm prefix, upgrade that prefix to alpha.18 without uninstalling or
forcing installation, verify configuration preservation, and exercise the
installed native package. No new production browser consent or long-running
expiry test is required for this packaging repair. Full production OAuth
lifecycle acceptance remains incomplete and is not claimed.
