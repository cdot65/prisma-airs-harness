# Direct OAuth MCP resources

`setup-mcp` can connect directly to an OAuth-protected MCP resource using browser
PKCE, separate native credentials and an HTTP Bearer token. Sign in to inference
in the selected Redtail environment first; both logins must identify the same
human subject and issuer, with different clients and audiences.

```sh
airs-harness setup-mcp --name prisma-airs \
  --url https://prisma-airs-mcp.cdot.io/mcp \
  --issuer-url https://auth.redtail.cdot.io/realms/redtail \
  --oidc-client-id prisma-airs-harness-mcp \
  --audience https://prisma-airs-mcp.cdot.io/mcp \
  --resource https://prisma-airs-mcp.cdot.io/mcp \
  --scope airs.gateway.read --scope airs.profiles.read
```

`--resource` must equal the endpoint and audience. Before opening the browser,
the harness fetches protected-resource metadata and verifies that it advertises
the configured resource, issuer and requested scopes. Authorization-code and
refresh exchanges both carry the resource parameter. Device login is unavailable
for this mode. The credential helper sends `Authorization: Bearer` for the bound
resource; legacy gateway bindings retain their existing header and serialization.

Scopes are part of the stored identity fingerprint. Use the same setup options
when signing in again. A changed endpoint, client, audience, scope set or subject
requires a new binding/environment. Never use an inference access token as an
MCP credential. Use `mcp list` or `/mcp` to inspect the configured server.

For development, use `prisma-airs-mcp-dev.cdot.io` and client
`prisma-airs-harness-mcp-dev` in the command above. Permissions come from the MCP
server's human role and resource policy; registering a public client alone does
not grant workspace or profile access.
