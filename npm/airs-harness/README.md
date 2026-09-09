# Prisma AIRS Harness

The `@cdot65/prisma-airs-harness` package launches the matching prebuilt native Rust agent.
Node.js 22.13+ in the 22.x line, or 23.5+ is required. No Rust compiler is needed on the endpoint.
Inference uses your configured Prisma AIRS AI Gateway; local files and tools
run on your machine. This package has no PAH server or OpenAI CLI dependency.

Install the private review release from GitHub Packages. Your GitHub account needs
package access. At the password prompt, use a classic personal access token with
`read:packages` permission:

```sh
npm login --auth-type=legacy --registry=https://npm.pkg.github.com
npm install -g @cdot65/prisma-airs-harness@auth-review --include=optional --registry=https://npm.pkg.github.com
airs-harness --version
```

The `auth-review` tag selects the published prerelease for hands-on review.
Linux x64 and macOS Apple Silicon packages are provided; Intel Macs and native
Windows packages are not. See the [Mac installation guide](https://github.com/cdot65/airs-harness/blob/main/MACOS.md)
for the complete first-time setup. A copy also ships inside this package.

Only platforms included in that release are installable. Do not disable optional
dependencies: they carry the platform binary. The launcher does not download or
compile code at startup and has no install scripts.

Replace the gateway URL with the address supplied by your organization:

```sh
airs-harness setup --environment work --gateway-url https://your-gateway.example/v1
airs-harness --environment work login
airs-harness --environment work
```

Choose company sign-in or enter a workspace API key at the hidden-input prompt.
Your administrator supplies the company issuer URL, public client ID, and gateway
audience. `airs-harness resume` opens existing conversations.

macOS uses Keychain, and Linux requires an unlocked Secret Service for native
credential storage. Installing through npm does not provision a Linux keyring
session. Git, ripgrep, and your project's tools remain separate prerequisites;
Linux also requires usable Bubblewrap.

Fresh state uses `~/.airs-harness`. Existing `~/.airs-terminal` state is reused
when the new directory is absent. `AIRS_HARNESS_HOME` overrides either default;
the legacy `AIRS_TERMINAL_HOME` override is still accepted. Existing credential
store identifiers remain stable so the rename does not discard authentication.
Do not delete an old executable that stored credential-helper paths still use.

Update by installing an administrator-approved version from the same registry.
Configuration and history are outside the npm package. License notices and
build provenance ship with each native platform package.

## Managed Prisma AIRS CLI

This release pins `@cdot65/prisma-airs-cli@5.2.0` and includes eight
built-in Prisma AIRS skills. Run `airs-harness airs doctor --output json` before
Prisma AIRS operations. Existing protected CLI configuration and explicit
`PANW_AI_SEC_API_KEY`, `PANW_MGMT_CLIENT_ID`, `PANW_MGMT_CLIENT_SECRET`, and
`PANW_MGMT_TSG_ID` environment settings are supported. Keycloak sign-in does not
supply management credentials. See the bundled `PRISMA-AIRS-CLI.md` for setup,
capabilities, known CLI limits and the compatibility policy.
