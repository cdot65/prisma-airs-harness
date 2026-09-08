# Alpha.8 npm publication

Status: in progress. The Linux native package is published; the root launcher is
held until the Apple Silicon build and credential acceptance pass.

## Release identity

- Product: Prisma AIRS Harness; CLI and unscoped npm package: `airs-harness`.
- Version: `0.1.0-alpha.8`.
- Registry: `https://npm.cdot.io`, LAN/VPN, anonymous downloads.
- Native release provenance: `b3486afcd3da6bcc27daf1bb4b02c56ebadb5739`.
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

- [Native Mac build run](https://github.com/cdot65/airs-harness/actions/runs/34175596384).
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

The owner ruled out future Intel Mac builds on September 8. The already-running
Intel build is allowed to finish, but its artifact will not be published. Active
release and Keychain workflows now target Apple Silicon only. Inherited upstream
workflows remain disabled. A source/dependency-keyed Cargo cache is configured for
future builds and saved immediately after successful compilation, so a later
acceptance failure retains compiled outputs. The current cold build cannot acquire
that cache retroactively.
A dedicated Apple Silicon runner is being considered, pending host access.

An independent agent's provisional review identified registry namespace isolation
and a weak local-tool assertion. Both are remediated before final publication.
A final evidence-based reassessment follows completed release gates.
