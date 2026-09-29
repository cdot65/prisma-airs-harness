---
title: MCP authorization and tool routing
---

The harness's built-in MCP client connects to the **AI Gateway MCP listener**.
The gateway authenticates the user through CAS/organizational SSO, authorizes the
workspace integration, and owns OAuth to the upstream MCP service. There are
three distinct checks: gateway login, tool discovery, and authorization of the
actual tool operation.

## Administrator setup

Deploy the upstream tool service and configure its backend credential with the
minimum required permissions. Register the gateway's confidential OAuth client
with that upstream service and configure the exact callback and scopes required
by the gateway integration. Store that client secret on the server.

Publish the integration into the intended gateway workspace. Configure the
listener's organizational SSO/CAS policy and the user/group grants. Use a separate
MCP resource audience and credential flow as required by that service; do not
reuse the inference token or assume `completions.write` grants tool access.

For the sanitized incident-service example, grant user `alex` a subject binding
that permits `list_incidents` and `get_incident`. Keep creation and modification
outside the read-only group. Provide the integration's full gateway URL, such as
`https://gateway-mcp.example.com/tools-dev/mcp`.

## User connection

Open `airs --environment work`, enter `/mcp`, and select **Add gateway MCP server**.
Use local name `incident-tools` and the administrator-provided gateway URL.
Complete organizational sign-in and any gateway-managed upstream consent. On
SSH, open the displayed link on your browser device, then paste the entire final
callback URL only into the manager's hidden field. The browser's localhost page
may fail because the harness is on the remote host; the manual callback field
completes this flow.

Wait for credential persistence and tool discovery. Choose **Start new
conversation** after a connection or identity change. The old conversation and
unsent draft are preserved; the harness does not insert a changed tool inventory
into the previous conversation.

```sh
# Shell alternative to the in-session manager:
airs --environment work mcp add incident-tools \
  --url https://gateway-mcp.example.com/tools-dev/mcp \
  --scopes mcp:servers:read,mcp:tools:list,mcp:tools:call
# Use only if the add flow did not finish login:
airs --environment work mcp login incident-tools --no-browser
```

Scopes must match the gateway's published authorization contract. The example
scope list is for a gateway exposing these permissions, not a universal upstream
MCP convention. Device authorization support is independent of inference SSO.

```mermaid
sequenceDiagram
  actor Person
  participant Agent as Native MCP client
  participant Gateway as AI Gateway MCP and CAS
  participant IdP as Organization identity
  participant Upstream as Upstream MCP service
  participant Backend as Target system
  Agent->>Gateway: Connect to integration MCP URL
  Gateway-->>Agent: Authorization requirements
  Person->>IdP: Browser sign-in
  IdP-->>Gateway: Organization identity
  Gateway->>Gateway: Check workspace and integration grant
  opt Upstream authorization needed
    Gateway->>Upstream: Gateway-owned OAuth flow
    Person->>Upstream: Consent when required
    Upstream-->>Gateway: Upstream grant held by gateway
  end
  Gateway-->>Agent: Gateway-facing MCP credential
  Agent->>Gateway: Initialize and list tools
  Gateway->>Upstream: Authorized discovery
  Upstream-->>Agent: Tool inventory via gateway
  Agent->>Gateway: Read-only tool call
  Gateway->>Upstream: Proxy with upstream authorization
  Upstream->>Backend: Enforce subject and operation permissions
  Backend-->>Agent: Result via service and gateway
```

## Prove a tool call

Ask the agent to list up to five active incidents using `incident-tools`, without
creating or updating records. Inspect the transcript for the actual
`list_incidents` call and returned result. An authorized empty list is valid; a
model-generated answer without a tool call is not evidence.

Restart and repeat the read-only call to test persisted credentials. Then test a
user without the tool grant and confirm denial. Retain gateway trace IDs and
sanitized result status, not incident bodies or authorization tokens.

Use `/mcp` → **Reconnect and verify** for current discovery, and **Sign in** for
renewal. Inference `/signin` and `airs logout` do not replace MCP login/logout.
Sign out a connection before removing it when retiring local credentials; revoke
the gateway's upstream grant separately when it must no longer be usable.
