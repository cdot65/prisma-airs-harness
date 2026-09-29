---
title: Hands-on command cheat sheet
---

This page is for doing things: the commands, in the order you usually need them.
It keeps explanations short on purpose. When you want to know why a command
behaves the way it does, follow the link at the top of each section to the page
that explains it.

These commands use sanitized example endpoints. Replace public settings with
those supplied by your administrator. Enter secrets only in hidden prompts or
the identity provider's browser. Use [configuration](../configuration/keycloak.md)
for the policy behind the commands and [validation](../validation/acceptance.md)
for the expected results.

## Install and identify

How releases are organized: [Releases and installation channels](../guides/releases.md).

```sh
node --version
npm install -g airs-harness
airs --version
airs cli --version
type -a airs airs-cli airs-harness
```

Supported: Apple Silicon, Linux x64 and Linux ARM64. Node requires
`^22.13.0 || >=23.5.0`. If optional dependencies were omitted by local npm
configuration, reinstall the same version with `--include=optional`. To install a
specific version or use your organization's registry, see
[release channels](../guides/releases.md#install-upgrade-and-roll-back).

## Create a profile and sign in

How environments and credentials fit together: [Environments](../guides/environments.md).

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

`env show` can include private local paths and endpoints, so review its output
before sharing it. Doctor's access probe sends an inference request; it does not
test MCP. `env use` affects new processes.

## Inside the terminal

How the terminal session and routing work: [Terminal workflow](../guides/terminal.md).

| Action | Command |
| --- | --- |
| Connection health / sanitized diagnostic report | `/doctor` |
| Renew inference SSO | `/signin` |
| Add, sign in, discover or retire gateway tools | `/mcp` |
| Inspect permitted model routes | `/model` |
| Configure the optional judge credential | `/typesafe` |

To add a tool connection, use `/mcp` → **Add gateway MCP server** with local name
`incident-tools` and the complete gateway URL. On SSH, paste the final browser
callback only into the hidden MCP authorization field. After an MCP identity or
inventory change, choose **Start new conversation**.

Read-only test prompt:

> Use incident-tools to list up to five active incidents, showing their numbers,
> short descriptions and priorities. Do not create or update records.

Inspect the actual tool call and result; an answer without a tool call is not
a connection test. Why: [Remote tools through AI Gateway](../guides/mcp.md).

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

How the bundled CLI and skills fit in: [Skills and the local agent loop](../guides/skills.md).

```sh
airs cli tenant create platform-admin
airs cli --tenant platform-admin doctor
airs cli --tenant platform-admin aigateway --help
airs cli --tenant platform-admin runtime --help
```

`tenant create` prompts for management credentials. Keep `cli` immediately after
`airs`; a harness environment flag does not select a CLI tenant. Prefer explicit
`--tenant` in operational commands.

## Provision gateway resources

How a request is authorized and routed: [Workspace, models and API keys](../configuration/gateway.md).

Run these commands as the management administrator after configuring the
`platform-admin` tenant. The workspace command creates and binds its SCM scope.
An administrator must then grant that scope to the intended SCM service account.
This management grant is separate from the end user's Keycloak role and gateway
workspace membership.

```sh
airs cli --tenant platform-admin aigateway workspaces create \
  --name agent-production \
  --description 'Production agent inference and gateway tools' --output json

airs cli --tenant platform-admin aigateway workspaces list --plane admin --output json
airs cli --tenant platform-admin aigateway integrations providers
```

Record the returned workspace UUID, slug and scope name. Set the following public
identifiers to your actual values; they are examples, not credentials:

```sh
AIRS_DOCS_ORG_ID='1234567890'
AIRS_DOCS_WORKSPACE_ID='11111111-1111-4111-8111-111111111111'
AIRS_DOCS_WORKSPACE_SLUG='replace-with-returned-workspace-slug'

# The provider credential is requested in a hidden prompt.
airs cli --tenant platform-admin aigateway integrations create \
  --organisation-id "$AIRS_DOCS_ORG_ID" --ai-provider open-ai \
  --name agent-models --slug agent-models

airs cli --tenant platform-admin aigateway integrations list --output json
```

Use the integration UUID returned by your tenant, then bind only this workspace
while preserving other assignments:

```sh
AIRS_DOCS_INTEGRATION_ID='22222222-2222-4222-8222-222222222222'
airs cli --tenant platform-admin aigateway integrations workspaces set \
  "$AIRS_DOCS_INTEGRATION_ID" \
  --workspace-binding "$AIRS_DOCS_WORKSPACE_ID=true" --preserve-existing
airs cli --tenant platform-admin aigateway integrations workspaces list \
  "$AIRS_DOCS_INTEGRATION_ID" --output json
airs cli --tenant platform-admin aigateway integrations models list \
  "$AIRS_DOCS_INTEGRATION_ID" --output json
```

Select an allowed model and create the saved route and JWT policy using the
[gateway configuration checklist](../configuration/gateway.md). Provider model
names and policy schemas depend on the deployed gateway. Confirm the saved-config
slug before entering it in the identity claim mapper.

Bind a provisioned MCP integration in the same way:

```sh
AIRS_DOCS_MCP_ID='33333333-3333-4333-8333-333333333333'
airs cli --tenant platform-admin aigateway mcp integrations workspaces set \
  "$AIRS_DOCS_MCP_ID" \
  --workspace-binding "$AIRS_DOCS_WORKSPACE_ID=true" --preserve-existing
airs cli --tenant platform-admin aigateway mcp integrations workspaces list \
  "$AIRS_DOCS_MCP_ID" --output json

# Inspect traffic after the user's validation request.
airs cli --tenant platform-admin aigateway telemetry requests \
  --workspace "$AIRS_DOCS_WORKSPACE_SLUG" --days 1
```

These commands follow the bundled CLI's command contracts. Check
`airs cli aigateway --help` if a flag differs on your installation. Creating a
resource does not establish a successful model request or tool call; complete the
[validation workflow](../validation/acceptance.md) with your actual user.

## Recover and retire

```sh
# Restore the same person's inference identity for saved sessions.
airs --environment work login --restore-session

# Change the credential: rotate a key, or switch between SSO and a key (guided).
airs env auth workspace-api

# Rotate a workspace key without prompts; tests access afterwards.
airs --environment workspace-api login --replace --with-api-key < new-key.txt

# Retire each credential separately, then unregister the local profile.
airs --environment work mcp logout incident-tools
airs --environment work logout
airs env remove work
```

Unregistering preserves local files and history and does not revoke remote
credentials. Revoke a workspace key or gateway-held upstream grant at its server
when required. Identical native MCP connection names and URLs can share a
credential record for the same OS user across environments, so signing out in
one environment can affect another.

## Update or roll back

```sh
# Update to the latest stable release.
npm install -g airs-harness@latest

# Roll back by naming the earlier version.
npm install -g airs-harness@<previous-version>
```

Use the same registry the earlier version was published to; see
[release channels](../guides/releases.md). Close running harness sessions before
switching installed versions. Preserve state and consult the release record for
upgrade and rollback results. Do not delete credential stores or conversation
history to repair an installation. Upgrade and rollback leave them in place, and
deleting them loses sign-ins and history without touching the package itself.
