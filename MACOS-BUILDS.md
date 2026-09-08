# Apple Silicon build operations

Mac builds target `aarch64-apple-darwin` only. Intel Mac builds and publication are
prohibited by project policy. Inherited upstream workflows remain disabled.

The first hosted `macos-15` build compiled the release executable and Keychain
fixture in **3,021 seconds (50m 21s)**. It passed compilation and signing checks,
then exposed three fixture assumptions. See [PUBLICATION.md](PUBLICATION.md).
This is a measured cold-build baseline, not a warm-cache performance estimate.

## Current pipeline

The owned `airs-harness-macos-release.yml` workflow uses native Apple Silicon,
Rust 1.95, two compiler jobs, release optimization, no LTO, 16 codegen units and
no debug information. Build provenance records these choices.

The workflow checks out validation tooling separately from its runtime source.
The optional `source_ref` input allows corrected acceptance tooling to validate
the same immutable runtime revision. Normal future builds can leave it empty.
Compilation and native package provenance use the runtime checkout; fixture
execution uses the workflow checkout. Record both revisions in release evidence.

Cargo dependency and build-output caches are restored before compilation and
saved immediately after successful compilation. An architecture-matching fallback
allows reuse after validation-only workflow edits; Cargo still validates the
lockfile, compiler options and source fingerprints. Cache restoration never skips
compilation checks or acceptance. Only manual trusted release workflows use this
cache. Warm-build duration remains to be measured.

The executable and Keychain fixture are also uploaded as explicitly **unvalidated**
diagnostic artifacts before acceptance. They are not release packages. Successful
native and npm checks produce the separate validated archive. Later credential
checks download that archive and require no Rust rebuild.

`airs-harness-macos-revalidate.yml` can promote the preserved executable only after
checking its runtime source revision, Mach-O architecture, ad-hoc signature and
hash, then rerunning native, Keychain and npm acceptance. It regenerates locked
license metadata without invoking a Rust build. Receipts identify both the
compilation run and validation tooling revision. The first artifact-only diagnostic
run completed its 27 native checks in 58.5 seconds; this is test time, not the
whole packaging pipeline.

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
