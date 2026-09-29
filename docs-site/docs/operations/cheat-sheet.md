---
title: Hands-on command cheat sheet
---

These commands use sanitized example endpoints. Replace public settings with
those supplied by your administrator. Enter secrets only in hidden prompts or
the identity provider's browser. Use [configuration](../configuration/keycloak.md)
for the policy behind the commands and [validation](../validation/acceptance.md)
for the expected results.

## Install and identify

```sh
node --version
npm install -g airs-harness@0.1.3 --registry=https://registry.npmjs.org
airs --version
airs cli --version
type -a airs airs-cli airs-harness
```

Supported: Apple Silicon, Linux x64 and Linux ARM64. Node requires
`^22.13.0 || >=23.5.0`. If optional dependencies were omitted by local npm
configuration, reinstall the same version with `--include=optional`.

## Create a profile and sign in

```sh
# New profile only; this saves the URL without opening sign-in.
airs env create work --gateway-url https://gateway.example.com/v1

# Browser OIDC through Keycloak (choose Entra there if configured).
airs --environment work login \
  --issuer-url https://sso.example.com/realms/example-corp \
  --oidc-client-id ai-gateway-agent \
  --audience stack-ai-inference

# Attended SSH alternative, when the issuer enables device authorization.
airs --environment work login --device-auth

# Separate workspace-key profile; key is entered in a hidden prompt.
airs env create workspace-api --gateway-url https://gateway.example.com/v1
airs --environment workspace-api login --with-api-key
```

The device command reuses saved public OIDC settings; supply the issuer, client
and audience flags on the first attempt if they are not yet saved. Skip `env
create` for an existing profile.

## Inspect, select and start

```sh
airs env list
airs env show work
airs env status work
airs --environment work doctor --verify-access
airs env use work
airs --environment work
```

`env show` can include private local paths and endpoints. Doctor's access probe
sends an inference request; it does not test MCP. `env use` affects new processes.

## Inside the terminal

| Action | Command |
| --- | --- |
| Connection health / sanitized diagnostic report | `/doctor` |
| Renew inference SSO | `/signin` |
| Add, sign in, discover or retire gateway tools | `/mcp` |
| Inspect permitted model routes | `/model` |
| Configure the optional judge credential | `/typesafe` |

After an MCP identity or inventory change, choose **Start new conversation**.
Use `/mcp` → **Add gateway MCP server** with local name `incident-tools` and the
complete gateway URL. On SSH, paste the final browser callback only into the
hidden MCP authorization field.

Read-only test prompt:

> Use incident-tools to list up to five active incidents, showing their numbers,
> short descriptions and priorities. Do not create or update records.

Inspect the actual tool call and result; an answer without a tool call is not
a connection test.

## MCP shell alternative

```sh
airs --environment work mcp add incident-tools \
  --url https://gateway-mcp.example.com/tools-dev/mcp \
  --scopes mcp:servers:read,mcp:tools:list,mcp:tools:call
# Only if sign-in remains incomplete:
airs --environment work mcp login incident-tools --no-browser
airs --environment work mcp --help
```

Use the scopes published for your gateway integration. Inference credentials
and upstream system passwords do not belong in this connection configuration.

## Product administration (separate credentials)

```sh
airs cli tenant create platform-admin
airs cli --tenant platform-admin doctor
airs cli --tenant platform-admin aigateway --help
airs cli --tenant platform-admin runtime --help
```

`tenant create` prompts for management credentials. Keep `cli` immediately after
`airs`; a harness environment flag does not select a CLI tenant. Prefer explicit
`--tenant` in operational commands.

## Recover and retire

```sh
# Restore the same person's inference identity for saved sessions.
airs --environment work login --restore-session

# Replace a workspace key through the hidden prompt.
airs --environment workspace-api login --with-api-key

# Retire each credential separately, then unregister the local profile.
airs --environment work mcp logout incident-tools
airs --environment work logout
airs env remove work
```

Unregistering preserves local files and history and does not revoke remote
credentials. Revoke a workspace key or gateway-held upstream grant at its server
when required. Identical native MCP connection names and URLs can share a
credential record for the same OS user across environments.

## Update or roll back

```sh
npm install -g airs-harness@0.1.3 --registry=https://registry.npmjs.org
# Previous stable distribution is available from the original registry:
npm install -g airs-harness@0.1.2 --registry=https://npm.cdot.io
```

Close running harness sessions before switching installed versions. Preserve
state and consult the release record for upgrade/rollback results. Do not delete
credential stores or conversation history to repair an installation.
