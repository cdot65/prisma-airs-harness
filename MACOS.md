---
title: Install Prisma AIRS Harness on your Mac
description: Install the native harness and connect inference and MCP through AI Gateway.
package_version: 0.1.0-alpha.22.mcp.3
status: test-channel-guide
updated: 2026-09-19
audience: end-users
platform: macos-arm64
---

# Prisma AIRS Harness on your Mac

Use an Apple Silicon Mac on the organization's LAN/VPN. Intel Macs are outside
this distribution. Run a native ARM64 Node installation from your signed-in
desktop session; avoid a Rosetta terminal selecting an x64 Node process.

## Install or update

The supported Node range is `^22.13.0 || >=23.5.0`: Node 22.13.0 or newer in 22.x,
or 23.5.0 or newer. Install Node, Git and ripgrep with your usual package manager,
then verify the versions and architecture in this terminal:

```sh
node --version
npm --version
node -p process.arch
```

The last command should show `arm64`. If an old standalone product CLI owns
`airs`, upgrade it to `@cdot65/prisma-airs-cli@7.0.1` first; that command becomes
`airs-cli`. A fresh machine only needs the harness, which bundles CLI 7.0.0.

```sh
npm install -g airs-harness@mcp --registry=https://npm.cdot.io
airs --version
airs cli --version
```

The `mcp` test channel supplies in-session `/mcp` and `/doctor`. This guide covers
**0.1.0-alpha.22.mcp.3**, an owner-authorized test release; real-account SSO and
ServiceNow acceptance remain separate. Stable tags remain onboarding.4. Downloads are
anonymous, and normal installs include optional dependencies without
`--include=optional`. No uninstall or force is needed to upgrade.

If another executable shadows npm, inspect `type -a airs airs-cli airs-harness`
and `npm prefix -g`; preserve existing environments and credential-binding paths.
Restart running AIRS processes after an upgrade.

## Create, sign in and connect ServiceNow

Follow [Getting started](GETTING-STARTED.md#sso-to-servicenow-a-complete-first-session)
for the complete flow with company SSO or a workspace API key for inference.
For a new local profile, `airs env create work` guides you through settings and
sign-in. With explicit settings, login is a separate step:

```sh
airs env create work --gateway-url https://gateway.example.com/v1
airs --environment work login
```

Enter company credentials only in the browser, or the workspace key only in the
hidden terminal prompt. Allow the Keychain authorization prompt when shown.
A workspace key's gateway workspace need not share your environment's name.
Saving a credential and verifying inference access are distinct outcomes.

Before opening the agent, run `airs env show work`, locate `state_directory`,
and set top-level `mcp_oauth_credentials_store = "keyring"` in that directory's
`config.toml` before any table header. Update an existing value rather than
adding a duplicate. Native-only MCP storage is not automatically enforced by the
manager; this setting requires Keychain persistence before reporting success.

Open `airs --environment work`. Inside it, `/mcp` → **Add gateway MCP server**
registers the gateway-provided integration URL and begins gateway SSO. Use the
same intended company account as inference SSO; a workspace key does not grant
MCP access. After sign-in and verification choose **Start new conversation**.
History and the draft remain saved, without automatic submission or tool replay.
The gateway keeps the upstream OAuth client secret and grant; never configure a
direct upstream ServiceNow URL or enter its integration password in AIRS.

## Return and recover

Open the selected environment with `airs`, or its history with `airs resume`.
Use `airs env list`, `airs env use work` and `airs env show work` for local
profiles. `--environment work` selects one command without changing the default.

Inside AIRS, `/doctor` inspects local health. **Verify gateway access** requires
confirmation before a quota-using inference probe. **Restore company sign-in**
or `/signin` renews the same inference identity; workspace-key replacement
remains `airs --environment work login --with-api-key` from the shell.
For MCP use `/mcp` → **Sign in** or **Reconnect and verify**, then **Start new
conversation**. A successful browser page is not evidence of saved credentials
or an authorized tool call; finish the read-only check in the getting-started guide.

If Keychain access fails, inspect the reported category and any OS authorization
prompt in your signed-in desktop session. Unavailable does not necessarily mean
locked. Restore service access before retrying pending login/logout cleanup.
Over SSH, MCP supports hidden manual callback input; the PKCE verifier and native
credential store remain on the machine running AIRS.

`airs env remove work` unregisters the local name and preserves files/history.
It does not revoke gateway keys or upstream grants. Sign out inference and MCP
separately first if you intend to remove their local credentials. Uninstalling
with `npm uninstall -g airs-harness` likewise preserves environment files.

The documented walkthrough is a future attended check with your account; it does
not claim a newly completed production SSO, timed renewal or ServiceNow call.
