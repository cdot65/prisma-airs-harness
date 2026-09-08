# Alpha.8 npm publication

Status: in progress. The Linux native package is published; the root launcher is
held until the Apple Silicon build and credential acceptance pass.

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
- [Artifact-only acceptance and packaging](https://github.com/cdot65/airs-harness/actions/runs/34183332079):
  reuses the compiled ARM executable without rebuilding; checks source revision,
  architecture, signature, native execution, Keychain and npm installation. Validation
  tooling revision: `f309144dae5555943925e02bcbc17a280809070c`.
- Linux native executable SHA256: `0cfc270b0a5560fba0180557ce6fd8a94c34b86625e7c632b0ccd88dd312e400`.
- Existing Linux optimized/installed live checks: seven turns and three real
  scanner calls each, recorded in [VALIDATION.json](VALIDATION.json).

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
remain follow-up work. The full upstream Rust suite has a pre-existing missing
V8 musl dependency archive; code mode remains disabled.

The owner ruled out future Intel Mac builds on September 8. The original
Intel job finished; it will not be rerun and its output will not be published. Active
release and Keychain workflows now target Apple Silicon only. Inherited upstream
workflows remain disabled. A source/dependency-keyed Cargo cache is configured for
future builds and saved immediately after successful compilation, so a later
acceptance failure retains compiled outputs. The successful ARM compilation cache and executable have now been saved.
A dedicated Apple Silicon runner is being considered, pending host access.

An independent agent's provisional review identified registry namespace isolation
and a weak local-tool assertion. Both are remediated before final publication.
A final evidence-based reassessment follows completed release gates.
