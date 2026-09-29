---
title: Remote tools through AI Gateway
---

The native MCP client runs inside AIRS. Connect it to the administrator-provided
HTTPS **AI Gateway MCP listener**, including the final `/mcp` path. The gateway
proxies the upstream server and manages upstream OAuth. Company SSO for inference
and company SSO for MCP produce separate credentials and permissions.

## Connect and verify

1. Open `airs --environment work`, then enter `/mcp`.
2. Choose **Add gateway MCP server** and enter a local name such as `service-now`
   and the gateway MCP URL. The name does not select a gateway workspace.
3. Complete company sign-in and any gateway-managed consent. On SSH, open the
   displayed authorization link on your own device and paste the full callback
   URL into the manager's hidden field, even if the browser's localhost page fails.
4. Wait for credential storage and tool discovery, then start the new conversation
   offered by the manager. The previous conversation and draft are preserved.
5. Request a read-only operation and inspect the returned tool result.

For the example ServiceNow integration:

> Use service-now to list up to five active incidents, showing their numbers,
> short descriptions and priorities. Do not create or update records.

An authorized empty result is valid. A connected label alone does not establish
end-to-end access. See the complete [SSO-to-ServiceNow walkthrough](../generated/getting-started.md).

## Recovery and sign-out

Use `/mcp` to sign in, reconnect and verify, sign out or remove a connection.
`/doctor` also exposes connection diagnostics. Saved configuration, sign-in,
discovery and tool authorization are distinct states; fix the reported stage.
Removing configuration does not prove credential revocation. Sign out first when
retiring a connection, and revoke gateway-held upstream grants through the gateway.

`airs logout` signs out inference. `airs mcp logout service-now` signs out that
native MCP connection. Neither substitutes for the other. Stop running sessions
when discarding cached access tokens.

Inference supports `login --device-auth` when the company issuer permits it.
Native MCP device authorization depends on gateway support; use the MCP browser
and manual callback flow described above. Never paste a callback into the agent
conversation or use a direct upstream URL to bypass a gateway problem.

For shell-based setup, inspect `airs mcp --help` and the generated
[MCP login reference](../generated/reference/mcp-login.md).
