---
title: Remote tools through AI Gateway
---

This page has two parts. The first explains how remote tools reach the agent, so
you know what a sign-in does and does not give you. The second is the procedure:
connect, verify, renew and sign out. For the administrator's side of the setup and
the full authorization flow, see [MCP authorization](../configuration/mcp.md).

## How remote tools work

The native MCP client runs inside AIRS. It connects to the administrator-provided
HTTPS AI Gateway MCP listener, including the final `/mcp` path. The gateway
proxies the upstream server and manages upstream OAuth. Company SSO for inference
and company SSO for MCP produce separate credentials and permissions, so a working
inference login says nothing about tool access.

**Four states, not one.** Saved configuration, sign-in, discovery and tool
authorization are distinct states. A connection can be saved but not signed in,
or signed in but with no tools discovered, or discovered but not authorized for a
particular tool. When something fails, the useful question is which state you are
in, because each has a different fix.

**Why the callback goes in a hidden field.** On SSH, the browser's localhost page
may fail because the harness is on the remote host. The manager's hidden field
takes the full callback URL instead. Keep it out of the agent conversation, where
it can become model context.

**Why device authorization differs.** Inference supports `login --device-auth`
when the company issuer permits it. Native MCP device authorization depends on
gateway support, so MCP uses the browser and manual callback flow.

**Why the gateway is never bypassed.** A direct upstream URL skips the gateway's
authorization instead of fixing a gateway problem, so the harness does not use one
as a fallback.

## Connect and verify

1. Open `airs`, then enter `/mcp`.
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
end-to-end access. The [MCP getting started guide](../generated/getting-started-mcp.md)
walks through each step with screenshots and the expected output.

## Renew and sign out

Use `/mcp` to sign in, reconnect and verify, sign out or remove a connection.
`/doctor` also exposes connection diagnostics. Fix the state that is reported
rather than starting over.

`airs logout` signs out inference. `airs mcp logout service-now` signs out that
native MCP connection. Neither substitutes for the other. Stop running sessions
when discarding cached access tokens.

Removing configuration does not prove credential revocation; the credential can
still be valid after the local entry is gone. Sign out first when retiring a
connection, and revoke gateway-held upstream grants through the gateway.

For shell-based setup, inspect `airs mcp --help` and the generated
[MCP login reference](../generated/reference/mcp-login.md).
