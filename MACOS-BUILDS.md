# Apple Silicon build operations

Mac builds target `aarch64-apple-darwin` only. Intel Mac builds and publication are
prohibited by project policy. Inherited upstream workflows remain disabled.

The first hosted `macos-15` build compiled the release executable and Keychain
fixture in **3,021 seconds (50m 21s)**. It passed compilation and signing checks,
then exposed three fixture assumptions. See [PUBLICATION.md](PUBLICATION.md).
This is a measured cold-build baseline, not a warm-cache performance estimate.

## Current pipeline

The owned `airs-harness-macos-release.yml` workflow separates three jobs:

1. **Preflight:** Python syntax, packaging contracts, immutable-artifact checks,
   and Node launcher tests run on Linux before allocating a compiler runner.
2. **Build:** Apple Silicon compiles the CLI and native Keychain fixture, signs
   them ad hoc, and uploads their immutable artifact. Candidate CLI optimization
   is level 1; dependencies retain release settings, with `LTO=false`, 16 codegen
   units and no debug information. Provenance records the actual settings.
3. **Acceptance:** A fresh Apple Silicon job downloads that exact artifact ID,
   verifies runtime source, archive and both executable hashes, architecture and
   signatures, then runs native, Keychain, packaging and npm acceptance. It does
   not invoke a Rust build or test command.

Set `preflight_only` to true when checking workflow or fixture changes first.
This runs only the cheap job; its receipt explicitly excludes Rust compilation
and native acceptance. Leave it false for a full build and acceptance run.

The workflow checks out validation tooling separately from its runtime source.
The optional `source_ref` input resolves once during preflight; the build checks
out that exact commit. Normal future builds can leave it empty.
Compilation and native package provenance use the runtime checkout; fixture
execution uses the workflow checkout. Record both revisions in release evidence.

Cargo dependency and build-output caches are restored before compilation and
saved after the mandatory immutable-artifact upload. Cache saves are best effort;
an unavailable cache must not turn successful compilation into a failed build.
An architecture-matching fallback
allows reuse after validation-only workflow edits; Cargo still validates the
lockfile, compiler options and source fingerprints. Cache restoration never skips
compilation checks or acceptance. Only manual trusted release workflows use this
cache. The revised source154 build and earlier cache-restored builds are recorded below.

Acceptance restores a separate dependency-source cache for license inventory,
without downloading the large Rust build-output tree. A miss can fetch locked
dependency metadata without compiling.

For a transient acceptance failure, choose **Re-run failed jobs** in GitHub.
The successful build job stays complete and acceptance reuses its immutable
artifact. Artifact and evidence names include the run attempt; selection receipts
record the artifact ID, compilation attempt and acceptance attempt. Artifacts are
retained for 30 days. [GitHub rerun behavior](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/re-run-workflows-and-jobs)

If fixture code changes, use `airs-harness-macos-revalidate.yml` from the corrected
tooling revision. Supply the original build run, exact artifact ID, runtime commit,
and independently verified archive/CLI/fixture SHA-256 values from build evidence.
This workflow runs the full acceptance scope on preserved bytes without compiling.
Choose `macos-15` (the default) or `macos-26` for the hosted Apple Silicon runner.
The same verified executables can be exercised on both OS versions; every run
records its selected runner and actual OS version. Both choices retain ARM64
checks. Hosted macOS 26 acceptance does not replace the owner's Mac review.
Rerunning an old workflow uses its original tooling revision; it does not pick up
a fixture fix. Runtime changes still require a new build.

For npm packaging changes, artifact-only acceptance also selects the exact npm
consumer (`10.9.8` or `12.0.2`) and package layout. `legacy` preserves the existing
unscoped layout; `scoped-bundled` stages the scoped harness and native packages
with the locked managed CLI included. The receipt records the npm version and
layout separately from runtime provenance. These options do not publish packages
or establish real GitHub package access by themselves.

Both workflows create **private, non-publishable candidates**, even when native
acceptance passes. They do not promote ad-hoc signatures to production signing or
claim owner-device acceptance. Older artifacts with unrecorded compiler settings
retain that uncertainty. Cheap checks reduce avoidable failures but do not replace
native acceptance or prove runtime/fixture compatibility on every platform.

## Measured cache behavior

The failed f964 run restored a 2.14 GB build cache in 67 seconds, then rebuilt
131 workspace packages in 42m 16s. The preceding c10 run also rebuilt those
packages after a successful cache restore and took 47m 16s. Third-party compiled
dependencies were reused; this was not a total cache miss. Both compilations
succeeded and failed later in acceptance.

Fresh-checkout timestamps can invalidate workspace artifacts, but the logs did
not retain Cargo fingerprint reasons, so that explanation remains unconfirmed.
The local Linux 154 build took 9m 20s while rebuilding essentially the CLI only,
with different optimization settings and hardware. It is not a comparable Mac
benchmark. See the [timing and cache review](validation/2026-09-08/authentication/ci-build-performance/REPORT.md).

The next planned build will retain Cargo fingerprint diagnostics and separate
CLI/fixture timing reports. Its workflow enables the compiler fingerprint info
log to identify why Cargo invalidates cached crates. Do not trigger another long
build solely to collect diagnostics. Artifact-only acceptance retries already avoid
compilation; persistent-workspace or compiler-cache changes need separate
measurements before claiming a speed improvement.

The source154 split run completed successfully: CLI compilation took **47m31s**
after a 48-second restore, and both build/source cache saves succeeded. Native
and installed-package acceptance passed on macOS 15.7.9. The same immutable
executable then passed macOS 26.6.2 acceptance in **7m13s without compiling**.
See the [Mac15 build and acceptance receipt](validation/2026-09-08/authentication/154b4f0bc/macos-15/RUN-RECEIPT.json)
and [Mac26 artifact-only evidence](validation/2026-09-08/authentication/154b4f0bc/macos-26/README.md).
This verifies build/acceptance separation; it establishes no compiler speedup or
owner-device/signing approval.

The reported `unused_mut` in app-server and two unused cloud-tasks imports occur
because their uses are behind `cfg(debug_assertions)` while release builds omit
those branches. They were warnings, not the acceptance failure. Their cleanup is
deferred to the next runtime revision so these verified executable bytes remain
unchanged during fixture and packaging validation.

## Dedicated Mac requirements

A dedicated Apple Silicon Mac with a persistent workspace can preserve local
compiler outputs between iterations. Confirm actual hardware and benchmark it;
do not promise a speedup based only on the machine's name.

Before moving a workflow, confirm:

- Native Apple Silicon architecture and a supported macOS version; current
  acceptance uses macOS 15.
- SSH access or an existing repository-scoped GitHub runner, under a dedicated
  non-admin build account with adequate free disk for the checkout, dependencies,
  build outputs and package staging.
- Xcode Command Line Tools and the workflow's Rust, Python, Node, CMake, pkgconf,
  Git and ripgrep prerequisites.
- A signed-in user session with an unlocked login Keychain for native credential
  acceptance. Do not store the Mac login password in workflow YAML or disable
  Keychain security to make a background daemon pass.
- An up-to-date Actions runner. The pinned cache action requires runner 2.327.1
  or later; see the [official cache action requirements](https://github.com/actions/cache/blob/668228422ae6a00e4ad889ee87cd7109ec5666a7/README.md).
- A repository-specific runner label. Verify it is online before changing the
  release and Keychain workflows to that label. Keep the hosted ARM fallback
  available until native execution, Keychain and package checks pass there.

Keep npm publishing credentials off the build runner. Builds need read access to
the source repository and return artifacts; authenticated publication runs from
the authorized operator environment. Do not enable arbitrary fork workflows on
a persistent runner.

Measure a cold build, an unchanged-source build and one small source edit before
raising compiler concurrency. Record elapsed time, peak resource use where
available, cache hit status and acceptance results. Preserve the same sandbox,
credential and package-integrity checks when moving runners.

No dedicated host has been selected yet; host access is pending and does not
block the current hosted Apple Silicon release.

## Forgejo runner disk space

The Forgejo `airs-macos-arm64` runner is the owner's Mac, so the build refuses to
start below **40 GiB** free and warns below 60 GiB. The job log reports the actual
free space. A release build peaks near 30 GiB: about 5–7 GiB of `target/`, the
cache archive and its extracted copy, 2.5 GiB of Cargo sources, up to 10 GiB of
sccache (`SCCACHE_CACHE_SIZE`) and package staging.

The build cache is keyed by `Cargo.lock` and the toolchain, not by commit, so the
runner keeps one compiled `target/` per dependency set instead of one per build.
Cargo rebuilds only the crates that changed.

`scripts/airs_mac_runner_cleanup.sh` reclaims space without touching release
evidence. By default it only reports; `--apply` removes. It skips everything while
a runner job is in progress, then removes:

- bulky files from `~/.cache/airs-*` scratch folders older than 14 days, keeping
  `.json`, `.md`, `.log` and `.txt` evidence, and archives older than 14 days;
- runner job work directories older than 2 days;
- action cache blobs unused for 21 days (a pruned entry restores as a cache miss).

Install it once on the runner to run daily at 04:30:

```sh
scripts/airs_mac_runner_cleanup.sh            # preview
scripts/airs_mac_runner_cleanup.sh --install  # launchd job io.cdot.airs-runner-cleanup
```

It logs to `~/Library/Logs/airs-runner-cleanup.log`. `KEEP_DAYS`, `CACHE_DAYS` and
`RUNNER_HOME` override the defaults.
