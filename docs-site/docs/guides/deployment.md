---
title: Documentation and package deployment
---

Forgejo owns source review, CI and package release authorization. The GitHub
repository is a one-way source mirror. Documentation uses GitHub Pages at
**https://cdot65.github.io/prisma-airs-harness/**, with the same tagged publication
pattern as the SDK and CLI sites.

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

The GitHub repository enables only the owned documentation workflow; inherited
upstream and historical package workflows remain disabled. Pages uses the
`github-pages` environment, `pages: write` and OIDC deployment permissions. It
does not require npm credentials or a separate deployment token.

## Native package delivery

Documentation publication does not build or promote binaries. Harness package
delivery uses the owned `.forgejo/workflows/airs-harness-*` workflows and the
repository's release tooling. Source, tooling and packaging commit identities
are recorded independently. Existing package versions are immutable.

Apple Silicon candidates are Developer ID signed and notarized on the dedicated
Mac runner. Linux x64 and ARM64 packages require exact installed acceptance;
an ARM64 cross-build or QEMU version probe alone is not release acceptance.
Candidate and fresh registry checks cover native credential storage, bundled CLI,
upgrade/rollback and the relevant agent workflow before promotion.

Harness 0.1.3 is distributed through public npmjs.org and `npm.cdot.io`.
Current versions and platform boundaries are in [release channels](releases.md).
The repository's `RELEASE.md`, `RELEASE-TEST-PACKAGES.md`, release specifications
and retained receipts describe the operational procedure and actual acceptance.
