# AIRS command migration — alpha.22

Published September 17, 2026, under the owner's instruction to release both products for remote end-to-end testing.

## Released commands

- `airs-harness@0.1.0-alpha.22` owns `airs` and bundles Prisma AIRS CLI **7.0.0** / SDK **0.33.0**. Use `airs cli ...`; no standalone installation is required.
- `@cdot65/prisma-airs-cli@7.0.1` owns only `airs-cli`. Public npm `latest` points to this stable release; `next` retains 7.0.0.
- All four harness packages on `https://npm.cdot.io` have `latest`, `alpha`, and `migration` at alpha.22. The historical `gateway-validation` tag remains alpha.21.
- The deprecated `airs-harness` launcher remains for alpha.22 and is scheduled for removal in alpha.23. The standalone CLI has no `airs` alias.

CLI 7.0.0 was published first under `next` and frozen into the harness. npm's trusted publisher rejected promotion of the existing tag with HTTP 401. The standalone 7.0.1 patch was therefore published through the successful release workflow as `latest`. Its command implementation, tenant schema, and SDK pin match 7.0.0. The harness's already-tested 7.0.0 bundle was not repacked.

Upgrade an existing standalone CLI first, then install the harness. Never use npm `--force` to resolve command ownership. Inspect `type -a airs airs-cli airs-harness` and `airs --migration-check` if another installation shadows npm's commands. Preserve existing harness environments, credential bindings, histories and product tenant files.

```sh
# Only needed when upgrading a machine with the old standalone CLI:
npm install -g @cdot65/prisma-airs-cli@7.0.1 --registry=https://registry.npmjs.org

# The sole installation needed on a fresh machine:
npm install -g airs-harness@0.1.0-alpha.22 --include=optional --registry=https://npm.cdot.io
airs --version
airs cli --version
```

## Provenance

Native runtime source: `2a787c03d1673b28fe85803c4082dd1a5e85a36e`.
Final launcher/packaging source: `a39575ff8e706da6a9d86518d7f3fa366fc72434`.
Standalone stable CLI source: `48794e5b37db42130538610a5f37ea2f0f0b5b04`.
Bundled CLI 7.0.0 source: `ad4f4109d591cbfe0079760ca7b830a5669f60d3`.

`NPM-PACKAGES.json` records exact archive hashes, npm integrity, native source and packaging tooling. `BUNDLE-INVENTORY.json` records dependency pins, licenses and native image dependencies. Public dependency archive URLs in the lock were verified against matching public npm integrity before packaging; the external dependency allowlist was not widened.

| Installed platform | Native binary SHA-256 |
| --- | --- |
| Linux x64 musl | `60c1fda9ece4b83cf7fed3415196d460e0b822e0e8334c155b2d2b1638e10275` |
| Linux ARM64 musl | `f161ab0a38dea39b3c7602222b528b52d7b0d3a3d9b935e0241e4d415ccfedef` |
| Apple Silicon | `d0163d8915574cc82874a56d16002ecac3949497114402c7b048c2dee8cd6846` |

Linux ARM64 was built in Forgejo run 3591 and executed natively in the ARM64 Linux VM on Jadzia before publication. Apple Silicon was built in run 3592, then signed, notarized and checked in run 3593. Notarization submission `8e32a410-f20b-47e7-9682-f472029c1870` was accepted. No Intel Mac package was built or published.

## Acceptance

- CLI: 1,920 unit/integration tests, 14 release checks, typecheck, lint, formatting, dependency audit, build and packed-consumer checks passed. Lint retains seven preexisting warnings.
- Harness: 925 scoped Rust CLI tests, 51 skills tests, clippy, packaging/launcher/upgrade/completion checks passed.
- Every platform: installed candidate and fresh anonymous published-registry installs passed 47 executable checks (46 passed, one platform-inapplicable skip), environment lifecycle, and exact native hash verification.
- Every platform: CLI 6.1.1 + harness alpha.21 → CLI 7.0.0 first → harness alpha.22; independent tenant/environment selection; poisoned global CLI; independent uninstall/reinstall; rollback to the original pair. Existing state was preserved and `force_used` is false. Alpha.20 upgrades also passed on Linux x64 and Apple Silicon.
- Every platform: managed CLI contract checks include 20 help surfaces, tenant-only credentials, doctor behavior and real PDF/PNG/JPEG/SVG/DOCX generation with native image dependencies. Agent execution used the bundled `prisma-airs-cli` skill and absolute managed helper.
- Every platform: default standalone 7.0.1 installs coexist with published harness alpha.22 / bundled 7.0.0; uninstalling standalone preserves the harness and bundle.
- Actual bash/zsh/fish completion and public `npm exec` ownership preflight passed. macOS signing and native Keychain fixtures passed.

Per-platform subdirectories hold candidate, registry, migration, installed CLI, default-install and standalone coexistence receipts. Candidate receipts accurately retain `published: false`; `PUBLICATION.json` and subsequent registry/default receipts establish publication. Preliminary Mac artifacts also retain pre-publication status. The initial Mac run's stale `airs-harness env create` assertion is preserved; the validator was corrected to `airs env create`, and acceptance was rerun against the same signed runtime bytes.

## Documentation and remaining user acceptance

The canonical vault walkthrough and both Docusaurus sites document the installation order, new commands, tenant setup, environment lifecycle and company SSO → gateway ServiceNow MCP → same company SSO → read-only incident call. `DOCS-PUBLICATION.json` records final deployments and browser verification.

These checks establish distribution and migration readiness. They do not claim a fresh human production SSO session, a fresh live ServiceNow call, or hourly frontend renewal. The user will perform the attended remote workflow with their own identity. Inference and MCP remain routed through AI Gateway; shared company identity does not merge their tokens or supply product management API credentials.
