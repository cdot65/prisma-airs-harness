---
title: Install Prisma AIRS Harness on your Mac
description: Install the native harness and connect inference and MCP through AI Gateway.
package_version: 0.1.0-alpha.21
status: gateway-release-acceptance
updated: 2026-09-17
audience: end-users
platform: macos-arm64
---

# Prisma AIRS Harness on your Mac

Use an Apple Silicon Mac on the organization's LAN/VPN. Intel Macs are
unsupported. Alpha.21 consolidates environment management under `env` and
removes top-level `setup` and `status`. Existing environments and history are
preserved. The deployed 30-minute SSO idle limit can require fresh login;
production active-expiry acceptance remains incomplete.

## Install or update

Open Terminal in your normal signed-in desktop session. Install Node.js, Git and
ripgrep if needed, then install the harness:

```sh
brew install node git ripgrep
npm install -g airs-harness@0.1.0-alpha.21 --include=optional --registry=https://npm.cdot.io
airs-harness --version
```

Downloads are anonymous. No GitHub token or npm login is required. An ordinary npm upgrade
needs no uninstall, manual binary deletion or force. Keep existing environments
and conversation history. If an old manual executable shadows npm, inspect
`command -v airs-harness` and `npm prefix -g` before changing that legacy link.

For the complete workflow—environment creation, inference SSO, ServiceNow MCP
registration and MCP SSO with the same company identity—follow
[SSO to ServiceNow](README.md#sign-in-with-sso-and-connect-servicenow).
The new environment commands require alpha.21 or newer.

## Sign in for inference

For a new connection, create a named environment with your administrator's
inference gateway URL. Keep an existing environment when upgrading.

Run `airs-harness env create work` for guided setup and sign-in. To supply the
gateway explicitly and sign in separately:

```sh
airs-harness env create work --gateway-url https://gateway.example.com/v1
airs-harness --environment work login
```

Creation selects the environment, so `airs-harness` opens it afterward. Use
`env use NAME` to change that default; `--environment NAME` selects an environment
for one command without changing the default.

Company sign-in asks for the OIDC issuer, public client ID and inference audience.
Enter your password only on your organization's browser page. A workspace API
key is a separate inference option entered at the hidden-input prompt. It is not
a Keycloak JWT and does not authorize the user MCP flow.

Allow the macOS Keychain prompt when shown. Confirm successful CLI completion:

```sh
airs-harness --environment work doctor --verify-access
```

`PASS gateway_access` proves the small inference check, not MCP authorization.

## Connect the gateway MCP integration

Your administrator must provision the integration and map your CIE directory
group to its gateway workspace. Use the connection URL from that gateway:

```sh
airs-harness --environment work mcp add prisma-airs \
  --url https://gateway-mcp.example.com/prisma-airs/mcp \
  --scopes mcp:servers:read,mcp:tools:list,mcp:tools:call
airs-harness --environment work mcp list
airs-harness --environment work
```

The example scopes match this deployment's gateway discovery. Set
`mcp_oauth_credentials_store = "keyring"` at the top level of the environment's
`config.toml` when native storage is required. Complete gateway CAS/company SSO
and any gateway-managed upstream consent. The gateway keeps the confidential
upstream client secret and tokens. Do not configure a direct upstream URL or
reuse the old upstream client ID in the native harness.

Use `/mcp` to inspect the connection, then ask the agent to list your authorized
workspaces. Browser success alone does not prove credential persistence or tool
access. Native MCP onboarding preserves the existing inference history.

## Return, sign out and recover

In alpha.15, `/signin` restores inference for the same verified company identity
while preserving the conversation and draft. Complete browser sign-in and
Keychain persistence, then explicitly retry the request. From another terminal,
use `airs-harness --environment work login --restore-session --no-browser`,
replacing `work` with the running session's environment name.

MCP sign-in is separate: follow the displayed gateway MCP login command and
start a fresh conversation afterward. The gateway's opaque token does not supply
verified account continuity for automatic MCP conversation recovery. A locked
Keychain calls for unlocking and retrying; an upstream management permission
denial does not call for another human login.

| Task | Command |
| --- | --- |
| Open the selected environment | `airs-harness` |
| Resume its history | `airs-harness resume` |
| Create an environment and sign in | `airs-harness env create work` |
| List saved environments | `airs-harness env list` |
| Inspect the selected environment | `airs-harness env show` |
| Check local credential availability | `airs-harness env status` |
| Select work | `airs-harness env use work` |
| Rename an environment | `airs-harness env rename work team` |
| Unregister, preserving local files | `airs-harness env remove team` |
| Sign out native MCP | `airs-harness mcp logout prisma-airs` |
| Sign back into native MCP | `airs-harness mcp login prisma-airs` |
| Sign out inference | `airs-harness logout` |
| Sign back into inference | `airs-harness login` |

Gateway MCP logout does not revoke the gateway-held upstream grant. If a browser
says it cannot connect to localhost after five minutes, restart native login and
complete the newest tab; the old callback listener has expired. If credential
saving fails under SSH, run login in the signed-in desktop session and allow
Keychain access. Headless Linux acceptance may use Safari here with a verified
SSH reverse tunnel to its own loopback callback; its native credential store
remains on Linux.

For `access_denied` with missing workspace access, check the existing CIE group
mapping, Full Sync and workspace Members tab. A successful inference login is
not evidence of MCP workspace access. Share the error category and version with
your administrator, never access tokens or client secrets.

Uninstalling with `npm uninstall -g airs-harness` preserves saved environments.
Sign out inference and MCP separately first if you want local credentials removed.
