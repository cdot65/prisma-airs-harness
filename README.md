# Prisma AIRS Harness

A local terminal agent built for **Prisma AIRS AI Gateway**. Work with project
files, run approved commands, use bundled Prisma AIRS skills, and call remote MCP
tools through the gateway.

[Documentation](https://cdot65.github.io/prisma-airs-harness/) ·
[Getting started](GETTING-STARTED.md) ·
[Release 0.1.3](RELEASE-0.1.3.md) ·
[Canonical source](https://git.cdot.io/cdot/prisma-airs-harness)

## Install

Use Node `^22.13.0 || >=23.5.0` on Apple Silicon, Linux x64 or Linux ARM64.

```sh
npm install -g airs-harness@0.1.3 --registry=https://registry.npmjs.org
airs --version
airs cli --version
airs env create work
airs
```

The same version is distributed through `https://npm.cdot.io`. The npm package
is `airs-harness`; the command is `airs`. CLI 7.2.0 and SDK 0.34.0 are bundled.
A separate product CLI installation is unnecessary. Windows and Intel Mac
packages are not provided. Linux requires a usable Bubblewrap sandbox and an
unlocked Secret Service session; macOS uses the user's Keychain.

If npm configuration omits optional dependencies, reinstall the same version
with `--include=optional`. If another package owns `airs`, inspect `type -a airs
 airs-cli airs-harness` and follow the [migration guide](RENAME.md); do not force
an overwrite. Upgrades preserve environments, credentials and history. Restart
running sessions after upgrading.

## Connect and verify

Your administrator supplies the gateway URL and either public OIDC settings
or a user workspace API key. Inference supports Keycloak company SSO, including
Entra federation through Keycloak, or workspace keys entered in a hidden prompt.

```sh
airs env create work --gateway-url https://gateway.example.com/v1
airs --environment work login
airs --environment work doctor --verify-access
airs --environment work
```

Skip creation when the profile exists. Doctor makes a small inference request
and can consume quota. It does not verify MCP. In the terminal, use `/mcp` to
add the gateway integration and complete its separate organizational login.
After tool discovery, start a new conversation and verify an actual read-only
tool result.

The gateway owns upstream MCP OAuth. Both model inference and remote MCP go
through AI Gateway. Local files, shell execution, sandboxing, approvals and
history remain in the local runtime; selected contents and tool results can
become model context.

## Documentation paths

- [Architecture and deployment](https://cdot65.github.io/prisma-airs-harness/platform/deployment/)
- [Keycloak configuration](https://cdot65.github.io/prisma-airs-harness/configuration/keycloak/)
- [Entra federation](https://cdot65.github.io/prisma-airs-harness/configuration/entra/)
- [Workspace keys and routing](https://cdot65.github.io/prisma-airs-harness/configuration/gateway/)
- [MCP authorization](https://cdot65.github.io/prisma-airs-harness/configuration/mcp/)
- [End-to-end validation](https://cdot65.github.io/prisma-airs-harness/validation/acceptance/)
- [Command cheat sheet](https://cdot65.github.io/prisma-airs-harness/operations/cheat-sheet/)
- [Bundled CLI and skills](PRISMA-AIRS-CLI.md)
- [Ubuntu host preparation](UBUNTU-TEST-HOST.md)

Product administration uses a separate CLI tenant (`airs cli tenant create`).
It does not inherit inference SSO. The optional Jev judge also has a separate
TypeSafe credential and an explicit live-command approval boundary.

## Source and releases

Forgejo is authoritative for review, CI and releases. GitHub is a public source
mirror and hosts the dedicated Docusaurus site. Documentation publishes from
exact `airs-docs-<commit SHA>` tags. Inherited upstream workflows stay disabled.

See [development](https://cdot65.github.io/prisma-airs-harness/guides/development/)
and [release deployment](https://cdot65.github.io/prisma-airs-harness/guides/deployment/)
for build commands, platform checks, provenance and publication procedures.
Apple Silicon artifacts are Developer ID signed and notarized. Linux ARM64
requires native installed acceptance; a cross-build or QEMU probe alone is
insufficient. Release notes distinguish fixtures from live-account acceptance.

Derived from Codex; see [UPSTREAM.md](UPSTREAM.md), [LICENSE](LICENSE) and
[NOTICE](NOTICE) for upstream provenance and licensing.
