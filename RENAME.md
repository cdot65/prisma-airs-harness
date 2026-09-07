# Prisma AIRS Harness naming and upgrade contract

The standalone product is **Prisma AIRS Harness**, with executable and unscoped
Verdaccio package **`airs-harness`**. The rename is alpha.8, separate from the
already released alpha.7. This document does not announce registry publication.
The hosted PAH application remains a separate project and is not a dependency.
The independent repository is https://github.com/cdot65/airs-harness.

## Names used for new work

| Surface | Name |
| --- | --- |
| Product and terminal header | Prisma AIRS Harness |
| Command, help and resume instructions | `airs-harness` |
| npm launcher package | `airs-harness` |
| Native npm package | `airs-harness-<os>-<cpu>` |
| Fresh application home | `~/.airs-harness` |
| Explicit application-home override | `AIRS_HARNESS_HOME` |
| Project configuration | `.airs-harness/config.toml` |
| HTTP User-Agent | `airs-harness/<version>` |
| MCP client name/title | `airs-harness` / Prisma AIRS Harness |

The npm launcher has no install scripts, shell interpolation, source-build step,
or runtime downloader. The package builder adds exact-version optional native
dependencies only for the supplied verified release directories. A platform is
not supported for distribution merely because the launcher recognizes its name.

## Existing installations

The new home override takes precedence over the legacy `AIRS_TERMINAL_HOME`.
Without either override, an existing `~/.airs-terminal` is reused in place if
`~/.airs-harness` is absent. If both exist, the new directory wins; no histories
are merged. A legacy path that cannot be inspected is an error, not permission
to silently create a replacement home.

This preserves absolute model-catalog and credential-helper paths, environment
UUIDs, session binding revisions, encrypted refresh tokens and conversation
history. Historical conversations keep their original messages. New runtime
instructions and outgoing client identification use the current product name.
The generated greeting in legacy model catalogs is updated in memory for new
requests; the saved catalog hash, other instructions and past messages are retained.
The runtime replaces the product-owned inference User-Agent in memory after
validating the original gateway/auth configuration; it does not rewrite the
saved configuration or change the selected model.

Project configuration reads `.airs-harness` when present, otherwise the legacy
`.airs-terminal` directory. Both remain subject to project trust and gateway
binding restrictions. Use the new directory for new project settings.

Stored helpers may reference the previous executable by absolute path. Keep
that path available during an upgrade. On this Linux installation the operator
can retain `airs-terminal` as a compatibility symlink to the new `airs-harness`
binary. It is not a separately published product or npm command.

## Deliberately retained identifiers

| Identifier | Why it remains |
| --- | --- |
| `io.cdot.airs-terminal` | Stable native credential-store service; changing it would orphan encrypted credentials. |
| `airs-terminal-pilot`, `airs-terminal-mcp`, existing audiences and access groups | Deployed Keycloak authorization contracts. Renaming a CLI does not reprovision identities. |
| Existing scanner URLs, profile `Prisma AIRS Terminal`, backend `x-airs-terminal-mcp-key` | Deployed gateway/backend configuration; callers must still match the actual policy. |
| `AIRS_TERMINAL_POLICY` | Legacy backend environment setting; `AIRS_HARNESS_POLICY` takes precedence when supplied. |
| Historical `cdot65/prisma-airs-terminal` URLs and build paths | Archived source provenance; GitHub redirects the old repository URL to `cdot65/airs-harness`. |
| Existing container names, digests and infrastructure directories | References to deployed artifacts, not newly built package branding. |
| `validation/`, previous release records and administration evidence | Historical evidence must retain the names and hashes it actually verified. |
| Upstream `codex-*` crate names, license and notice attribution | Source lineage and internal engine API compatibility. |

The optional scanner source package identifies itself as AIRS Harness when next
built. This rename does not itself deploy new gateway, Keycloak or scanner images.

## Preparing the npm release

First produce and verify native release directories with build provenance and
license inventory. Then, from the repository root:

```sh
python3 scripts/package_airs_npm.py \
  --release-directory /absolute/path/to/verified-native-release \
  --output-directory /absolute/path/to/new-npm-staging-directory \
  --registry https://npm.cdot.io
```

Repeat `--release-directory` for additional validated targets. The builder emits
tarballs and `NPM-PACKAGES.json` with hashes and native-before-launcher publication
order. It never publishes. Validate installation in an isolated npm prefix and
run the executable/PTY fixtures through that npm command before publishing.
Do not claim Mac readiness until a complete native Mac package passes acceptance.

After publication, the endpoint install command will be:

```sh
npm install -g airs-harness --registry https://npm.cdot.io
airs-harness
```

Node.js/npm are endpoint prerequisites for this distribution. Guided first-run
setup and a graphical Mac installer are separate features, not part of the rename.
