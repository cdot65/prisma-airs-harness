# Bundled Prisma AIRS CLI and skills

This guide covers harness **0.1.0-alpha.22.mcp.5**. Install this exact version
once available in your registry. It retains the mcp.3 bundle: `@cdot65/prisma-airs-cli@7.0.0` and SDK
`0.33.0`. One npm harness installation includes the pinned CLI and eight embedded
product skills. `airs` starts the harness; **`airs cli ...`** runs its bundled CLI.
An independently installed product CLI uses **`airs-cli ...`**.

This version makes native MCP credential storage the default for new harness
environments. Existing environment modes and tokens stay unchanged; this does
not change CLI tenants or product API authentication. See [Getting started](GETTING-STARTED.md#4-open-airs-and-check-mcp-storage-for-existing-environments)
for optional MCP storage migration, including signing out in the original mode
before changing configuration and signing in again.

```sh
airs --version
airs cli --version
airs cli --help
airs cli tenant list
airs cli doctor --output json
```

CLI commands work before harness environment setup or company SSO. Put `cli`
immediately after `airs`; harness `--environment` does not select a product tenant.
`airs doctor` checks the harness; `airs cli doctor` checks product API readiness.
The managed runner verifies the exact bundled CLI and never substitutes a global
installation. Node 22.13+ in the 22.x line, or 23.5+, is required; keep npm optional
dependencies enabled for native executables and image/document generation.

## Upgrade command ownership

If an old global product CLI owns `airs`, upgrade it first:

```sh
npm install -g @cdot65/prisma-airs-cli@7.0.1 --registry=https://registry.npmjs.org
airs-cli --version
npm install -g airs-harness@0.1.0-alpha.22.mcp.5 --registry=https://npm.cdot.io
airs --version
airs cli --version
```

The `mcp` tag selects the currently published test build; inspect `airs --version` after installation. Normal installs include optional dependencies, so `--include=optional` is only a repair option when npm configuration omitted them.

Fresh harness users need only the second installation. The harness does not
install a global `airs-cli`. Open a fresh shell and inspect `type -a airs airs-cli airs-harness` if an alias, manual file or another package manager still resolves
an old executable. Resolve ownership through its original installer; do not force
npm to overwrite unknown files. The `airs-harness` launcher and its old
`airs-harness airs ...` forwarding remain compatibility entrypoints in alpha.22
and are scheduled for removal in alpha.23. Environments, history, saved logins,
MCP registrations and storage paths remain unchanged.

For a read-only check before global installation:

```sh
npm exec --yes --registry=https://npm.cdot.io --package=airs-harness@mcp -- airs --migration-check
```

This reports PATH entries and recognized package owners without executing those
commands or changing any installation. Inspect shell aliases/functions separately.

## Select product credentials

CLI 7 reads the selected tenant's JSON configuration. Credential environment
variables, `PRISMA_AIRS_CONFIG_PATH` and a project `.env` are ignored. Existing
CLI 6 tenant registrations work unchanged. To register a protected legacy file:

```sh
airs cli tenant create development --config /absolute/path/to/config.json
airs cli tenant switch development
airs cli doctor --output json
```

Or run `airs cli tenant create development` in a terminal for guided TSG ID,
OAuth client ID and hidden secret entry. Use `tenant set NAME KEY` for supported
edits; approved automation can supply secrets through stdin. Never paste secrets
into chat or put them in command arguments.

Scanner credentials are `airsApiKey` or `airsApiToken`. Management uses
`mgmtClientId`, `mgmtClientSecret` and `mgmtTsgId` with the tenant's token endpoint.
Company SSO and MCP grants do not substitute for these product credentials.
`airs env use` and `airs cli tenant switch` have independent selections. A trusted
`PRISMA_AIRS_TENANTS_PATH` can isolate the CLI registry; the harness does not
implicitly bind a CLI tenant to an environment.

## Skills and validation

Ask inside the harness:

> Use $prisma-airs-cli to check the bundled version and selected tenant, then
> diagnose missing settings without revealing credentials or modifying configuration.

The embedded skills cover diagnosis, Runtime Security, Guardrail Generation,
Red Teaming, AI Gateway, Model Security, DLP Testing and DLP Management. Agent
shell tools invoke the absolute `AIRS_MANAGED_CLI` path, so login-shell PATH changes
cannot select another product version. User-created skills are preserved.
AgentGuard is available through CLI help but has no dedicated embedded skill yet.

Doctor distinguishes pass/warn/fail/skip and performs network probes when
credentials are present. It is not proof of every operation or permission.
Skills require complete scan evidence before treating an Allow summary or
aggregate evaluation metric as success. Guardrail writes and rollback can
partially fail; preserve baseline policy and verify actual state. DLP corpus
generation is not document scanning, and ZIP generation is unsupported. Model
Security's separate Python scanner requires its own provisioning when requested.

The single-install bundle applies to npm distributions for Linux x64, Linux
ARM64 and Apple Silicon. Bare native archives do not include Node or the managed
CLI; use npm for `airs cli` and product skills. Windows launcher contracts remain
tested separately; this release does not add a Windows native package.

For future upgrades, pin the CLI and SDK exactly, review command/credential
contracts, refresh affected embedded skills and the lockfile, rebuild native
assets, and validate clean installs and upgrades on every released platform.
Retain bundle hashes, signing evidence and published-package acceptance receipts.
