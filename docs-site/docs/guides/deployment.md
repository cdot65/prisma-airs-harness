---
title: Documentation and package deployment
---

This page covers how this site and the native packages are published. The first
part explains how the pipeline is set up and why. The second part is the
procedure for publishing the documentation.

## How publishing is set up

Forgejo owns source review, CI and package release authorization. The GitHub
repository is a one-way source mirror. Documentation uses GitHub Pages at
https://cdot65.github.io/prisma-airs-harness/, with the same tagged publication
pattern as the SDK and CLI sites.

**Why a tag names the exact commit.** The live site is built from a tag of the form
`airs-docs-<full commit SHA>`. The GitHub workflow verifies that the tag and the
commit match before it builds anything, so the published site can always be traced
back to reviewed source. The deployed `source.json` records that commit, which is
how you check it afterward.

**Why only one workflow is enabled.** The GitHub repository enables only the owned
documentation workflow. Inherited upstream and historical package workflows remain
disabled. Pages deploys through the `github-pages` environment with `pages: write`
and OIDC deployment permissions, so it needs no npm credentials and no separate
deployment token.

**Documentation and packages are separate pipelines.** Publishing documentation
does not build or promote binaries. Package delivery uses the owned
`.forgejo/workflows/airs-harness-*` workflows and the repository's release
tooling. Source, tooling and packaging commit identities are recorded
independently, and an existing package version is immutable, so a published
package cannot be quietly replaced.

**Why acceptance is per platform.** Apple Silicon candidates are Developer ID signed
and notarized on the dedicated Mac runner. Linux x64 and ARM64 packages require
exact installed acceptance, because an ARM64 cross-build or QEMU version probe
alone does not exercise native credential storage or the sandbox. Candidate and
fresh registry checks cover native credential storage, the bundled CLI,
upgrade and rollback, and the relevant agent workflow before promotion.

## Publish documentation

1. Run the documentation checks and merge the reviewed change into Forgejo `main`.
2. Confirm the Forgejo **AIRS Harness documentation** check passes for that commit.
3. Create and push the lightweight tag `airs-docs-<full commit SHA>` to Forgejo.
4. Wait for the push mirror to copy the commit and tag to GitHub.
5. The owned `airs-harness-docs.yml` workflow verifies the exact tag/commit match,
   builds and tests the site, uploads the Pages artifact and deploys it.
6. Check the deployed `source.json` commit and browse the published routes.

For an already validated main commit:

```sh
git fetch origin main
source_commit=$(git rev-parse origin/main)
git tag "airs-docs-${source_commit}" "$source_commit"
git push origin "refs/tags/airs-docs-${source_commit}"
```

## Publish a package

Package publication follows the repository's release procedure, not this page.
The repository's `RELEASE-TEST-PACKAGES.md`, `MACOS-BUILDS.md`, the
`RELEASE-0.1.x.md` records, release specifications and retained receipts describe the operational procedure and the actual
acceptance. Current versions and platform boundaries are in
[release channels](releases.md).
