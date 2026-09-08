# Prisma AIRS Harness

The `airs-harness` package launches the matching prebuilt native Rust agent.
Node.js 22 or later is required. No Rust compiler is needed on the endpoint.
Inference uses your configured Prisma AIRS AI Gateway; local files and tools
run on your machine. This package has no PAH server or OpenAI CLI dependency.

Install the approved release from your organization's Verdaccio registry:

```sh
npm install -g airs-harness@0.1.0-alpha.8 --registry https://npm.cdot.io
airs-harness --version
```

The registry requires the organization's LAN/VPN; downloads do not require npm
login. Alpha.8 includes Linux x64 and macOS Apple Silicon packages. Intel Macs are unsupported.
See the [Mac installation and Keycloak runbook](https://github.com/cdot65/airs-harness/blob/main/MACOS.md)
for prerequisites, sign-in, remote scanner setup and an end-to-end test. A copy
ships with the installed package, so GitHub access is not required to read it:

```sh
less "$(npm root -g)/airs-harness/MACOS.md"
```

Only platforms included in that release are installable. Do not disable optional
dependencies: they carry the platform binary. The launcher does not download or
compile code at startup and has no install scripts.

Start with `airs-harness setup --gateway-url https://your-gateway.example/v1`.
Use `airs-harness login --help` to choose Keycloak sign-in or a workspace key.
Guided first-run onboarding is a separate upcoming feature. Run `airs-harness`
in your project directory; `airs-harness resume` opens existing conversations.

macOS uses Keychain, Windows uses Credential Manager, and Linux requires an
unlocked Secret Service for native credential storage. Installing through npm
does not provision a Linux desktop/keyring session. Git, ripgrep and any project
tools remain runtime prerequisites; Linux also requires usable Bubblewrap.

Fresh state uses `~/.airs-harness`. Existing `~/.airs-terminal` state is reused
when the new directory is absent. `AIRS_HARNESS_HOME` overrides either default;
the legacy `AIRS_TERMINAL_HOME` override is still accepted. Existing credential
store identifiers remain stable so the rename does not discard authentication.
Do not delete an old executable that stored credential-helper paths still use.

Update by installing an administrator-approved version from the same registry.
Configuration and history are outside the npm package. License notices and
build provenance ship with each native platform package.
