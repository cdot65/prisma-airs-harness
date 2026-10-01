# SSO and ServiceNow walkthrough

This walkthrough has been split into two getting started guides, because company
sign-in for inference and OAuth sign-in for remote tools are separate connections
with separate credentials:

- [Getting started with company SSO](https://github.com/cdot65/prisma-airs-harness/blob/main/GETTING-STARTED-SSO.md) signs the harness
  into AI Gateway with Keycloak, or Microsoft Entra ID through Keycloak, and
  validates the token and gateway access. Its
  [Sign in over SSH](https://github.com/cdot65/prisma-airs-harness/blob/main/GETTING-STARTED-SSO.md#sign-in-over-ssh) section replaces
  the browserless sign-in section that was here.
- [MCP servers with OAuth](https://github.com/cdot65/prisma-airs-harness/blob/main/GETTING-STARTED-MCP.md) connects a remote MCP server,
  such as the gateway's ServiceNow integration, and proves it with a read-only tool
  call. Its
  [storage section](https://github.com/cdot65/prisma-airs-harness/blob/main/GETTING-STARTED-MCP.md#environments-created-before-native-mcp-storage)
  replaces the MCP storage check that was here.

For ServiceNow, use the MCP guide with your ServiceNow integration's gateway MCP
URL, and ask for a read-only incident lookup in step 7. The workspace API key path
is in [Getting started](https://github.com/cdot65/prisma-airs-harness/blob/main/GETTING-STARTED.md), the optional TypeSafe judge key in
[TypeSafe Jev judge](https://cdot65.github.io/prisma-airs-harness/guides/judge/),
and product CLI tenants in
[Provision a workspace with the bundled CLI](https://github.com/cdot65/prisma-airs-harness/blob/main/PRISMA-AIRS-CLI.md).
