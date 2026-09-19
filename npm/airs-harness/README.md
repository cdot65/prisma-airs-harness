# Prisma AIRS Harness

`airs` starts the native Prisma AIRS terminal agent. Inference and remote MCP
both use AI Gateway. The built-in MCP client connects to the gateway integration;
the gateway owns upstream OAuth. The npm package remains `airs-harness`.

## Install or update

Use Node.js **22.13.0 or newer in the 22.x line**, or **23.5.0 or newer**
(`^22.13.0 || >=23.5.0`). Check both tools in the terminal you will use:

```sh
node --version
npm --version
```

Installing npm alone does not upgrade Node.js. If Ubuntu supplies Node 18,
install a supported Node version using your organization's usual method, reopen
the terminal and check again. npm may otherwise finish with `EBADENGINE`; the
mcp.3 launcher rejects unsupported Node before starting AIRS or its bundled CLI.

The **mcp test channel** contains the in-session `/mcp` manager and `/doctor`
dashboard. This guide covers **0.1.0-alpha.22.mcp.3**, an owner-authorized test
release. Real-account SSO, workspace-key and ServiceNow acceptance remain separate
from automated distribution checks. Stable `latest`,
`alpha` and `onboarding` remain **0.1.0-alpha.22.onboarding.4**, which lacks these
dashboards. Install the currently published test build on the organization's
LAN/VPN and inspect its version:

```sh
npm install -g airs-harness@mcp --registry=https://npm.cdot.io
airs --version
airs cli --version
```

Git, ripgrep and your project's build tools remain prerequisites; Linux also
requires a usable Bubblewrap sandbox.

Linux x64, Linux ARM64 and Apple Silicon have native packages; Intel Mac and
Windows are outside this distribution. Downloads are anonymous. Normal installs
include optional dependencies and do not require `--include=optional`. If your
npm configuration omits them, re-enable them for installation. The launcher does
not compile or download code at startup. See the [Mac guide](MACOS.md).

The harness includes Prisma AIRS CLI **7.0.0** and eight product skills as
`airs cli`. If an older standalone CLI already owns `airs`, upgrade it first:

```sh
npm install -g @cdot65/prisma-airs-cli@7.0.1 --registry=https://registry.npmjs.org
airs-cli --version
```

A separate CLI install is otherwise unnecessary. Do not force npm to overwrite
another package's command. Use `type -a airs airs-cli airs-harness` and
`airs --migration-check` to inspect command ownership. `airs-harness` remains a
temporary compatibility alias; new commands use `airs`.

Ordinary upgrades preserve environments, credentials and history. Restart running
AIRS processes after upgrading. Fresh state uses `~/.airs-harness`; existing
`~/.airs-terminal` state is reused when the new directory is absent.
`AIRS_HARNESS_HOME` overrides the default, with `AIRS_TERMINAL_HOME` retained as a
compatibility fallback. Keep targets referenced by existing credential bindings.

## First session: inference, then gateway MCP

Follow [Getting started](GETTING-STARTED.md) for the complete SSO-to-ServiceNow
journey, including workspace API keys as an alternative for inference. A local
environment name does **not** bind to a gateway workspace name. Separate
environments when you need separate connections, credentials or history.

For guided creation and a choice of authentication methods:

```sh
airs env create work
```

Explicit creation saves settings without opening sign-in:

```sh
airs env create work --gateway-url https://gateway.example.com/v1
airs --environment work login
```

Reuse existing profiles instead of recreating them. Company SSO uses your
administrator's issuer, public client ID and inference audience. Alternatively,
create a user workspace API key in AI Gateway with `completions.write` and an
approved default saved config/model route, then enter it only at the hidden
prompt from `airs --environment work login --with-api-key`. Never put a key in a
command argument or conversation. A saved credential is not verified inference
access; `airs --environment work doctor --verify-access` sends a small quota-using
probe. HTTP status and request/gateway trace IDs help diagnose route or policy
rejection without exposing the credential.

Before MCP login, find the selected environment's `state_directory` with
`airs env show work`. Set top-level `mcp_oauth_credentials_store = "keyring"` in
its `config.toml`, before table headers, updating any existing value. The manager
respects this setting but does not enforce native-only storage automatically.
Linux needs an available Secret Service session; macOS uses Keychain. Installing
npm does not provision a Linux credential service.

Start `airs --environment work`, then enter `/mcp` and choose **Add gateway MCP
server**. Supply the gateway's integration URL, complete company SSO and any
gateway-managed upstream consent, then choose **Start new conversation**. Stay
inside AIRS; previous history and the unsent draft remain saved and nothing is
replayed. Use the same intended company account for inference SSO and MCP. An
inference workspace key does not create that SSO identity or grant MCP access.

`/mcp` also provides **Sign in**, **Sign out**, **Reconnect and verify** and
**Remove connection**. A connected inventory does not prove a tool call; the
getting-started guide ends with a read-only ServiceNow incident request. Over
SSH, the MCP dialog supports hidden manual callback input. A shell fallback is
`airs --environment work mcp login service-now --no-browser`.

## Inspect and recover

Open `/doctor` in the current environment. **Refresh diagnostics** does not send
an inference request. **Verify gateway access** asks for confirmation before
**Send connectivity check** makes a small request that can consume quota and
appear in gateway logs. It sends no files, conversation content or tools. MCP
verification remains separate through **Reconnect and verify**.

For inference SSO, **Restore company sign-in** or `/signin` preserves the
verified identity, conversation and draft. Workspace-key replacement currently
requires leaving the session and using the selected environment's shell login.
MCP changes require **Start new conversation**; the opaque gateway token does not
establish identity continuity with the previous conversation.

An unavailable credential service does not establish that the store is locked.
Check service access and any OS authorization prompt in the same user session.
If cleanup is pending, restore service access and retry login or logout for that
environment. The harness retains both the original and cleanup errors. A
pre-exchange identity-service outage preserves the saved refresh grant; a
possibly consumed rotating grant requires sign-in instead of blind replay.

```sh
airs env list
airs env use work
airs --environment work doctor
airs --environment work doctor --verify-access
airs env remove retired
```

`env use` changes the saved default; `--environment` selects one command.
`env remove` unregisters a name and preserves its files/history; it does not
revoke gateway keys or upstream grants. Sign out inference and MCP separately
before removal when you intend to remove local credentials. No full production
renewal, owner-session Ubuntu fix or fresh live ServiceNow acceptance is claimed
by this documentation update.

## Product CLI and skills

Run `airs cli doctor --output json` before product operations. CLI tenants and
harness environments are independent. CLI 7 reads selected tenant JSON; company
SSO does not supply management API credentials. See [Bundled Prisma AIRS CLI and
skills](PRISMA-AIRS-CLI.md) for setup, capabilities and limits, and the
[public first-session guide](https://cdot65.github.io/prisma-airs-reference-architecture/learn/login/)
for the educational walkthrough.
