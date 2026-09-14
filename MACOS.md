---
title: Install Prisma AIRS Harness on your Mac
description: Install the native harness and connect inference and MCP through AI Gateway.
package_version: 0.1.0-alpha.14
status: gateway-release-acceptance
updated: 2026-09-14
audience: end-users
platform: macos-arm64
---

# Prisma AIRS Harness on your Mac

Use an Apple Silicon Mac on the organization's LAN/VPN. Intel Macs are
unsupported. The alpha.14 candidate has passed signing, Apple notarization,
Keychain, npm upgrades and all eight production gateway tools. Two actual token
expiry cycles remain in progress; `latest` stays alpha.13 until promotion.

## Install or update

Open Terminal in your normal signed-in desktop session. Install Node.js, Git and
ripgrep if needed, then install the harness:

```sh
brew install node git ripgrep
npm install -g airs-harness@latest --include=optional --registry=https://npm.cdot.io
airs-harness --version
```

Downloads are anonymous. No GitHub token or npm login is required. At this
checkpoint, use the maintainer-provided alpha.14 package for acceptance; the
command above installs alpha.14 once it is promoted. An ordinary npm upgrade
needs no uninstall, manual binary deletion or force. Keep existing environments
and conversation history. If an old manual executable shadows npm, inspect
`command -v airs-harness` and `npm prefix -g` before changing that legacy link.

## Sign in for inference

For a new connection, create a named environment with your administrator's
inference gateway URL. Keep an existing environment when upgrading:

```sh
airs-harness setup --environment work --gateway-url https://gateway.example.com/v1
airs-harness --environment work login
```

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

| Task | Command |
| --- | --- |
| Open the selected environment | `airs-harness` |
| Resume its history | `airs-harness resume` |
| List saved environments | `airs-harness env list` |
| Select work | `airs-harness env use work` |
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
