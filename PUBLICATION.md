# Current Verdaccio release — alpha.12

`airs-harness@0.1.0-alpha.12` is published to `https://npm.cdot.io` as `latest`,
with Linux x64 and Apple Silicon native packages. Install or update with:

```sh
npm install -g airs-harness@latest --registry=https://npm.cdot.io
airs-harness --version
```

The Mac executable was compiled on Jadzia, Developer ID signed and notarized by
Apple. Forgejo package E2E run 79 passed. Fresh anonymous Verdaccio installs
verified both native hashes and the bundled CLI; the downloaded Mac signature
and online notarization ticket passed again. Published-package acceptance passed
43 Linux tests and 42 Mac tests with one Linux-only skip. CLI 5.2.0, SDK 0.28.0
and eight AIRS skills remain intact.

The owner explicitly authorized publication after disclosure of pending full
GNU Rust validation and independent/owner review. Publication is not a claim that
those gates passed. Only package manifests changed from the tested tarballs:
the private flag was removed and repository links now point to Forgejo. All other
archive members, including signed executable bytes, were verified unchanged.
[Publication receipt](validation/2026-09-13/verdaccio-alpha12/PUBLICATION.json)
and [promotion evidence](validation/2026-09-13/verdaccio-alpha12/PROMOTION.json)
retain the exact published archive hashes and authorization scope.

The GitHub Packages channel below remains at alpha.9. Its workflow instructions
are historical; GitHub Actions is disabled and GitHub is now a source mirror.

# GitHub Packages distribution

Version `0.1.0-alpha.9` is published privately under
[`@cdot65/prisma-airs-harness`](https://github.com/users/cdot65/packages/npm/package/prisma-airs-harness),
with Linux x64 and Apple Silicon packages. All three published downloads match
the staged archives. Fresh GitHub installs passed the executable suite on Linux
(27/27) and Apple Silicon (26 passed, one Linux-only check skipped).
[Acceptance run](https://github.com/cdot65/airs-harness/actions/runs/34228673532).
[Publication receipt](validation/2026-09-08/github-packages/PUBLICATION.json).
GitHub returned no repository association, so the packages are available on the
account Packages page; repository-sidebar linking remains a GitHub UI step below.

The owned `airs-harness-github-packages.yml` workflow publishes scoped copies of
an existing verified npm release to GitHub Packages. GitHub does not index
Verdaccio packages automatically. The three packages are `@cdot65/prisma-airs-harness`,
`@cdot65/prisma-airs-harness-linux-x64`, and `@cdot65/prisma-airs-harness-darwin-arm64`, with repository metadata pointing to
this repository. The executable command remains `airs-harness`.

Only the package manifests change: names are scoped, the destination registry is
GitHub, and native dependencies point to their immutable GitHub tarball URLs.
All original source archive checksums are checked against a committed publication
receipt before publication; published archives are downloaded and compared byte
for byte. Already published versions must match the staged integrity. The
workflow then installs from an empty cache and runs the executable acceptance
suite on Linux x64 and Apple Silicon. It does not compile Rust or build Intel Macs.

To publish a future validated release, attach its original npm tarballs to its
GitHub release, commit the publication receipt, then dispatch the workflow on
`main` with that release tag and receipt path. It uses the repository's temporary
`GITHUB_TOKEN` with `packages: write`; acceptance jobs use `packages: read`.
Packages are published privately. The receipt distinguishes declared repository
metadata from the repository association returned by GitHub. If GitHub returns
no association, the owner can select **Connect repository** on the package page
and choose `cdot65/airs-harness`. Then explicitly enable **Inherit access from
repository** under Package settings so repository readers can download it.
Connecting an already-published package alone does not enable inheritance.
Publication does not make the repository public.

## Install from GitHub Packages

GitHub requires a classic personal access token with `read:packages` and access
to the package. Authenticate interactively (use the token as the password):

```bash
npm login --auth-type=legacy --registry=https://npm.pkg.github.com
```

Resolve only the harness from GitHub, then install using npmjs for its public
CLI/SDK dependencies. Do not redirect the whole `@cdot65` scope to GitHub: those
public dependencies share that scope.

```bash
airs_harness_url="$(npm view @cdot65/prisma-airs-harness@0.1.0-alpha.9 dist.tarball --registry=https://npm.pkg.github.com)"
npm install -g "$airs_harness_url" --registry=https://registry.npmjs.org
airs-harness --version
airs-harness airs doctor --output json
```

If the previous unscoped package is installed globally in the same npm prefix,
run `npm uninstall -g airs-harness` before installing the scoped package; both
packages provide the same command. This removes the old npm launcher, while
environments and credentials remain in their existing harness configuration and
OS credential store. Authenticate and resolve the new download URL first.

The native dependencies still download from GitHub with the same authentication.
Verdaccio's existing anonymous LAN/VPN installation remains supported. For npm
registry authentication and repository linking, see
[GitHub's npm registry documentation](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-npm-registry).

# Alpha.9 managed CLI release

Published `airs-harness@0.1.0-alpha.9` and its Linux x64/Apple Silicon packages
to `https://npm.cdot.io`. Anonymous downloads match staged SHA256/SHA512
integrity. A fresh install with an empty cache and no registry credentials passed
managed CLI/version, document generation and a live built-in skill workflow.
Native/npm acceptance and the additional npm-installed Mac CLI Keychain lifecycle
check passed. The local `airs-harness` command now runs the published package;
existing configuration/binding files and the global CLI 3.3.0 were preserved.
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
