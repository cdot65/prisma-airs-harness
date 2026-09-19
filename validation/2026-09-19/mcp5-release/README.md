# mcp.5 release evidence — September 19, 2026

`airs-harness@0.1.0-alpha.22.mcp.5` is published under `mcp` at `https://npm.cdot.io`. Fresh anonymous registry acceptance passed on Linux x64, native Linux ARM64 and signed/notarized Apple Silicon. All existing non-mcp tags were preserved. The release includes the corrected Ubuntu preparation helper; the native runtime changes only its displayed version from mcp.4.

## Identities and preserved receipts

Runtime, tooling and packaging source are `9e4f1d2211afcfe81054bc00b8fccc546bf25b26`. The canonical release specification digest is `4d38763d27c5075599a1a830ab86f7718b2703c01374626f0eaf0e129aee6c12`.

| Target | Native SHA-256 |
| --- | --- |
| Linux x64 | `4a82d38a399e0d78fe3facfb4065418cc0d2c437468ffdbfe1f39768a5b45d31` |
| Linux ARM64 | `20c143820cdfe6b516b8dd410297f3c325a2364b40182e31dab1ef36cb621b48` |
| Apple Silicon | `28526eac904aed939a0c7364c25aa88d897819bc291ff75ac532e9be11998586` |

The frozen tooling manifest binds 154 files to Git. The shell helper is bound separately to source, candidate archive, published archive and installed bytes: SHA-256 `0f8619659b7a80f392d8bca638b401155829ca512d40b6c05dbbdc9048a1dcab`.

`RECEIPT-MANIFEST.json` maps each retained receipt to its scratch-relative source, size and SHA-256. Receipts are copied byte-for-byte; canonical JSON digests and file-byte hashes are distinct. `BUNDLE-CHECK.json` records collector checks, and `SHA256SUMS` covers retained files except itself. The collector omits raw logs, terminal transcripts, observation streams, credentials, installed trees and package archives. References inside original receipts describe the original run layout, not a standalone rerun bundle. This private evidence ledger must not be exported wholesale to the public site.

## Checks completed

- Affected source checks: 8 home-dir tests and 13 packaging tests passed. Formatting and diff checks passed. No core/common/protocol logic changed; no new full workspace pass is claimed.
- Owned native build workflows passed, including Mac Developer ID signing, notarization and installed signature checks. Linux ARM acceptance ran on native ARM hardware, not a QEMU version probe.
- Candidate and fresh anonymous registry matrices each passed on all three targets. They cover installation, onboarding, terminal behavior, installed regressions, native MCP management, diagnostics, bundled product CLI, upgrade and command output. Platform-specific skips are retained in each original receipt; the native-store positive persistence/reuse/logout test ran on every target.
- Each exact candidate native executable passed the approximately 178-second lifecycle profile: two renewal cycles each for inference and MCP, three verified tool turns and native credential cleanup. These synthetic gateway/identity fixtures do not establish production issuer behavior.
- Upgrades from mcp.4 and onboarding.4 passed three cases per platform, preserving checked configuration and the legacy target without force or uninstall.
- The exact candidate helper passed six real Ubuntu fixture checks with an explicit mcp.4 package override before publication. After publication, a fresh anonymous mcp.5 install passed the same checks using the exact installed helper and its unmodified mcp.5 default. Checks cover encrypted keyring creation, native write/read/delete, unlocked reuse and no-prompt checking, shell-quoted rerun paths containing spaces, wrong-password/locked-state preservation and correct-password recovery.
- Independent prepublication review scored 9.6/10. It correctly left registry verification pending; later registry and published-helper receipts close those gates. Independent publication review compared every candidate/staged archive member, verified that only permitted validation metadata changed, and checked registry integrity and protected tags.

The helper correction is included in mcp.5. The immutable mcp.4 package predates it; retain [its historical limitation](../native-mcp-release/README.md). The mcp.5 helper was delivered to a new owner-host filename, with older scripts and the working owner installation preserved. Fixture tests used private HOME, npm prefix, D-Bus and XDG storage, never the owner's credential values.

## Remaining scope and operational notes

The owner reported successful workspace-key inference and ServiceNow use on Ubuntu before this release handoff. That report is not an independently observed service transcript or a company SSO/device-authorization acceptance result. Existing inference device authorization requires issuer support; it remains separate from gateway MCP authorization. No new real-account login or tool call was performed during this release.

The historical 60-minute active and 35-minute idle observations remain bound to their recorded development executable; no such duration is claimed for mcp.5. The historical full Rust workspace suite is not green and is not represented as passing by these narrower release checks.

Mac capacity exceeded the 105 GiB operational threshold. A verified mcp.4 backup was retained, but this run did not delete its original directories. Other historical directories disappeared during concurrent external activity before this run performed any deletion; their actor and backup status are not asserted here. Owner VMs, the native ARM container and successful build caches were preserved.

The next bounded feature is workspace API-key replacement from `/doctor` inside AIRS. It is not implemented in mcp.5.

Check retained bytes with `sha256sum -c SHA256SUMS` from this directory.
