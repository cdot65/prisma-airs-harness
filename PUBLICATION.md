# Alpha.9 managed CLI release

Native/npm acceptance passed for Linux x64 and Apple Silicon. Publication and
an additional npm-installed Mac CLI Keychain lifecycle check are pending.
CLI 5.2.0 / SDK 0.28.0 are pinned. See
[managed CLI evidence](validation/2026-09-08/managed-cli/REVIEW.md).

Runtime source: `fae43287c8ddc11c37b6af60d9cd26ec48eb5a22`.
Linux SHA256: `a2126c876b79491ad4fb054de172bc3fbe012d1b6612dd9495a57c7bdeacd608`.
Apple Silicon SHA256: `5968c0b9f84c24eb8632216060955392bfbdfa1bdb68ab7f8f2c450303522ab1`.
The platform build receipts are snapshots taken before npm/live acceptance;
the consolidated root validation records the current status.

Historical alpha.8 publication evidence follows.

# Alpha.8 npm publication

Status: published. `airs-harness`, `airs-harness-linux-x64` and
`airs-harness-darwin-arm64` version `0.1.0-alpha.8` are available from Verdaccio.
Anonymous downloads match the staged SHA256 and SHA512 integrity values. A fresh
production-registry install with empty npm configuration/cache passed.
[Publication receipt](validation/2026-09-08/publication/PUBLICATION.json),
[installation receipt](validation/2026-09-08/publication/production-install.json).

## Release identity

- Product: Prisma AIRS Harness; CLI and unscoped npm package: `airs-harness`.
- Version: `0.1.0-alpha.8`.
- Registry: `https://npm.cdot.io`, LAN/VPN, anonymous downloads.
- Native package source revision: `b3486afcd3da6bcc27daf1bb4b02c56ebadb5739`.
- Linux executable compilation receipt: `17624ca2761a0ca4217f54944b69a6436d7ec375`;
  it was repackaged without changing the binary. Both revisions have the exact
  `codex-rs` tree `bc8f5ee8041d5a64111d8473b34d7a9b89999bc7`. Changes between those
  revisions are documentation, packaging, fixtures and validation records.
- Apple Silicon compilation uses `b3486afcd` directly. Packaging source and
  original compilation receipts are distinct evidence, not a claim that the
  existing Linux executable was recompiled during publication.
- npm launcher/tooling revision: `bf024215bf6e0ea4cbedcb400641e310f7319461`.
- Targets: Linux x64 musl and macOS Apple Silicon. Intel Macs are excluded by owner policy.
- [Mac user runbook](MACOS.md).

## Acceptance gates

The native Mac release workflow compiles optimized executables on native Apple Silicon
macOS 15, checks ad-hoc signatures and dynamic-library dependencies,
runs local execution and sandbox tests, exercises Keychain, packages native
archives with licenses and provenance, and installs/tests through npm.

A second workflow downloads those exact archives and verifies login, new-process
credential retrieval, an authenticated local tool loop, absence of a plaintext
key in application configuration/history, and logout through the installed npm CLI.
Its receipt is paired with archive-integrity evidence.

After native packages pass, they are published before the launcher. Published
metadata and anonymous downloads must match the staged SHA/integrity values.
A fresh install from production Verdaccio then exercises the actual npm command.

- [Initial native Mac build run](https://github.com/cdot65/airs-harness/actions/runs/34175596384):
  compilation/signature checks passed; three Mac fixture checks failed. Temporary
  paths used `/var` while the CLI used `/private/var`, and the PTY driver submitted
  before session initialization. Assertions and sandbox permissions are unchanged.
- [Apple Silicon retry](https://github.com/cdot65/airs-harness/actions/runs/34178828451):
  runtime source remains `b3486afcd`; validation tooling is
  `78233a21861175a14369e03d06eb3c3be014b314`. The three corrected fixtures passed on
  Linux before this retry. Compilation succeeded and its executable/cache were preserved.
  The follow-up diagnostics explained an extra default-route request as background
  session-title generation, not incorrect explicit-model routing. Fixture checks
  now classify every request by prompt and account for title generation explicitly.
  Native archives retain source-era fixture copies;
  current acceptance uses the separately identified workflow tooling. Published
  npm packages contain the executable, licenses and provenance, not those fixtures.
- [Successful artifact acceptance and packaging](https://github.com/cdot65/airs-harness/actions/runs/34183881120):
  reused the compiled ARM executable without rebuilding. Native tests: 26 pass,
  one Linux-only skip. Through npm: 25 pass, one Linux-only skip. Six launcher
  checks pass. The complete job took **4m45s**. [Mac evidence](validation/2026-09-08/publication/macos/VALIDATION.json).
- [Installed CLI Keychain lifecycle](https://github.com/cdot65/airs-harness/actions/runs/34184222859):
  secure login, new-process credential retrieval, authenticated local tool effects,
  absence of plaintext credentials in application state, and logout all pass.
  [Receipt](validation/2026-09-08/publication/macos/cli-keychain/cli-keychain.json).
- Earlier npm acceptance exposed a teardown timeout after successful interaction.
  The PTY fixture now drains output during bounded shutdown and handles EOF/reaping
  correctly. Independent review also found a launcher bug where a handled interrupt
  suppressed later signals; exit-state checks and a SIGINT→SIGTERM regression test
  corrected it. The successful run above includes both fixes.
- Linux native executable SHA256: `0cfc270b0a5560fba0180557ce6fd8a94c34b86625e7c632b0ccd88dd312e400`.
- Existing Linux optimized/installed live checks: seven turns and three real
  scanner calls each, recorded in [VALIDATION.json](VALIDATION.json).

## Fresh production install and live workflow

The published npm command passed all 26 Linux executable checks, then a seven-turn
live gateway/scanner workflow. It edited the calculator, passed four project tests
and independent assertions, switched default → explicit → default, and completed
three real `pan_inline_scan` calls with `allow` actions.
[Executable log](validation/2026-09-08/publication/production-executable-tests.log),
[live receipt](validation/2026-09-08/publication/production-live.json).

The live receipt distinguishes the npm JavaScript entrypoint hash from the native
executable hash. Replies correctly describe remote inference and configured MCP;
some free-form wording still overgeneralizes on-premises deployment and source
availability. [Prose review](validation/2026-09-08/publication/self-description-review.json)
records those limits; actual tool results establish the integration behavior.

## Registry controls

Verdaccio's upload limit is 256 MiB to accommodate the Linux package's base64
attachment. Ordered rules reserve `airs-harness` and `airs-harness-*` without
public npm proxying. Authenticated publishing and anonymous reading are separate.
GitOps configuration revision: `9af3ef88a246a7b3afaec95cc303705ce2ce6fa2` in
`cdot65/talos-cluster`; verified Synced and Healthy after rollout.

Never overwrite a published version. Compare an existing version's integrity
and skip it if identical; publish a new version if package content must change.
Preserve registry storage because it now contains authoritative first-party artifacts.

## Scope and limits

This is an internal alpha release, not a managed-device production certification.
macOS binaries are ad-hoc signed, without Developer ID signing or notarization.
Mac enterprise Keycloak/gateway/scanner hands-on acceptance must run on the LAN/VPN;
public hosted build runners cannot reach those private endpoints. Native macOS
15 local execution and Keychain fixtures are recorded separately from Linux live
checks. Windows npm distribution, guided onboarding, and managed-device signing
remain follow-up work. Conjur integration is prepared but not activated. The full upstream Rust suite has a pre-existing missing
V8 musl dependency archive; code mode remains disabled.

The owner ruled out future Intel Mac builds on September 8. The original
Intel job finished; it will not be rerun and its output will not be published. Active
release and Keychain workflows now target Apple Silicon only. Inherited upstream
workflows remain disabled. A source/dependency-keyed Cargo cache is configured for
future builds and saved immediately after successful compilation, so a later
acceptance failure retains compiled outputs. The successful ARM compilation cache and executable have now been saved.
A dedicated Apple Silicon runner is being considered, pending host access.

The original MVP verification matrix also requires live installed checks across
both operating systems, authentication modes and routing choices, plus owner
acceptance. This internal-alpha distribution release does not claim that broader
matrix is complete.

Independent review scored this **9/10 for the internal alpha distribution release**,
with no remaining release blocker. The reviewer independently fetched production
metadata, downloaded and verified the launcher, checked its packaged behavior,
reran six launcher tests and reviewed the platform/live receipts.
[Assessment](validation/2026-09-08/publication/independent-review.json).
This score excludes the broader MVP and owner acceptance listed above.
