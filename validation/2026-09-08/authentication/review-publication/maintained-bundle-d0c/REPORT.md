# Maintained private bundled installation acceptance

The maintained `--scoped --bundle-cli` packager and validator passed real Linux installations using Node 22.23.2 with npm 10.9.8 and 12.0.2. These are disposable, unpublished private alpha.9 packages. No native rebuild, publication, live management mutation, or production promotion occurred.

Packaging source is `d0c682c5f57bedfe60807744e604ab1d334e777f`; preserved native source is `154b4f0bcef7528844e85177d7e4dce611982044`. Both installed commands resolve native SHA256 `2fcb93d5b11e5c81f2851b94a200bf81da4adba0ecb9d60b20ab8b206e81e3ad`. Native integrity is verified before its first execution. Package-tooling file hashes were independently compared with the recorded Git commit.

Each installation used fresh npm configuration/cache and an isolated rejecting loopback registry/proxy. Ordinary scoped package-name installation made exactly four staged metadata/tarball requests, with zero unexpected requests and zero public dependency redirects. Inherited proxy and npm credential configuration was removed. This proves the maintained package path under those conditions, not real GitHub Packages credentials or teammate access.

Installed inventory checks passed for 70 exact-version packages, 3,091 retained files, 67 license files, and four Linux x64 Sharp payload packages covering glibc and musl. The target-derived Linux bundle intentionally excludes other operating systems. npm omitted four of five explicitly inventoried optional non-runtime source files; their staging evidence is retained. The one permitted shebang normalization preserves original and canonical hashes, reconstructs the original for verification, and changes only the declared executable's first CRLF. Every license remains byte-preserved.

Both installed launchers passed the real managed CLI 5.2.0 contract: configuration precedence and project-dotenv exclusion, credential-free doctor diagnostics without live probes, 20 capability command contracts, and PDF/PNG/JPEG/SVG/DOCX corpus generation using the native image dependency. These are generation tests, not live document detection.

The actual installed npm12 bundle was verified before two deliberate changes. Removing the exact optional Sharp package failed its manifest read; changing one actual license failed its exact hash. Both changes were restored, and complete inventory verification passed again. No native command was executed during these mutation checks.

The initial install correctly rejected npm update-notifier traffic; the second correctly rejected npm's shebang rewrite. Both failures are retained in `prior-failures.json`; they are not erased or counted as passing attempts. The separate [causal fixture correction](../npm-registry-fixture-correction/ADDENDUM.md) retracts earlier URL-specific attribution and distinguishes npm10 proxy denial from npm12 optional URL omission followed by exact installed-inventory rejection.

The full per-file inventory is packaged as `BUNDLE-INVENTORY.json`, SHA256 `974672dce73399147fa9d73d7906e95381a87511692eb9862e30a76849345485`. These receipts retain its binding, all dependency identities/archive hashes/license paths, and independent native/tarball hashes. Full staged packages and install logs remain at `/var/tmp/airs-maintained-scoped-bundle-fixed-1ttr2o9o`; earlier attempts remain at `/var/tmp/airs-maintained-scoped-bundle-ulutcon6`.

Full authentication release readiness remains **FAIL / not established**. This evidence does not close signing/notarization, owner Mac retest, Windows native/installed acceptance, real registry publication/access, or other mandatory authentication-plan gates. Hosted macOS artifact-only bundle acceptance is a separate running gate and is not credited here.
