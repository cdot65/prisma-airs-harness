# Prisma AIRS Harness

The `airs-harness` command runs a native Codex terminal agent. Inference and
remote MCP both use Prisma AIRS AI Gateway. Its built-in MCP client connects to
the gateway listener; the gateway proxies upstream servers and owns upstream
OAuth. No separate MCP executable is required.

## Install or update

On the organization's LAN/VPN, install from Verdaccio:

```sh
npm install -g airs-harness@0.1.0-alpha.15 --include=optional --registry=https://npm.cdot.io
airs-harness --version
```

Linux x64 and Apple Silicon are supported. The package uses anonymous downloads;
no npm login is required. Optional dependencies carry the matching native binary.
The launcher does not compile or download code at startup and has no install
scripts. See the bundled [Mac guide](MACOS.md).

Alpha.15 adds coordinated native token renewal and guided inference sign-in.
The deployed 30-minute SSO idle policy is preserved. Tokens renew during active
use while their grants remain valid; leaving a terminal open does not request
a longer session. Release evidence records which gateway lifecycle scenarios
were actually verified.

Ordinary npm upgrades preserve configuration and history without uninstalling or
using force. Fresh state uses `~/.airs-harness`; existing `~/.airs-terminal` state
is reused when the new directory is absent. `AIRS_HARNESS_HOME` overrides the
default, with `AIRS_TERMINAL_HOME` retained as a compatibility fallback. If an old
manual executable shadows npm, inspect the resolved command and npm prefix;
preserve old targets referenced by stored credential bindings.

## Connect inference and MCP

Keep an existing named inference environment. For a new connection:

```sh
airs-harness setup --environment work --gateway-url https://gateway.example.com/v1
airs-harness --environment work login
airs-harness --environment work doctor --verify-access
```

Use your administrator's inference URL and company issuer/client/audience, or
choose a workspace API key for inference. A workspace key is not the MCP user
credential. With a configured gateway integration:

```sh
airs-harness --environment work mcp add prisma-airs \
  --url https://gateway-mcp.example.com/prisma-airs/mcp \
  --scopes mcp:servers:read,mcp:tools:list,mcp:tools:call
airs-harness --environment work mcp list
airs-harness --environment work
```

Replace the example with the gateway-provided MCP URL and discovery-supported
scopes. Complete gateway CAS/company SSO and any upstream consent. Your directory
group needs the gateway workspace mapping, and your upstream identity needs the
read permissions. Upstream OAuth client secrets stay at the gateway.

macOS uses Keychain; Linux requires an unlocked Secret Service session. Installing
npm does not provision a Linux keyring. Set `mcp_oauth_credentials_store =
"keyring"` at the top of the environment's `config.toml` to require native MCP
storage. Confirm CLI completion and a tool call after browser consent. The
callback expires after five minutes; headless tests require a tunnel from the
browser's loopback port to the native process.

Use `/mcp` in the terminal for inventory, and `airs-harness resume` for history.
`mcp logout prisma-airs` signs out native MCP separately from inference `logout`;
it does not revoke the gateway's upstream user grant. Git, ripgrep and project
tools remain prerequisites; Linux also requires usable Bubblewrap.

## Renewing sign-in

When inference needs fresh company sign-in, use `/signin` in the open terminal.
The harness verifies the same issuer, client, audience and subject before saving
credentials. It keeps your conversation and draft, and does not replay completed
requests. Cancel leaves your work in place. In another terminal, the equivalent
command is `airs-harness --environment work login --restore-session --no-browser`.
Use the environment name shown by your running session.

MCP authentication is separate. On an MCP authentication failure, the turn stops
before an automatic alternate-credential attempt. Follow the displayed gateway
MCP login command, then start a fresh conversation. The gateway's opaque token
does not provide verified account continuity for automatic post-login resume.
A backend management denial is not a reason to repeat human SSO.

Unlock a locked native credential store and retry. A pre-exchange identity-service
outage preserves the saved refresh grant. If an exchange may have consumed a
rotating grant, the harness will require sign-in instead of blindly replaying it.
Restart running terminals after upgrading so they use the new native client.

## Managed Prisma AIRS CLI

This release pins `@cdot65/prisma-airs-cli@5.2.0` and includes eight
built-in Prisma AIRS skills. Run `airs-harness airs doctor --output json` before
Prisma AIRS operations. Existing protected CLI configuration and explicit
`PANW_AI_SEC_API_KEY`, `PANW_MGMT_CLIENT_ID`, `PANW_MGMT_CLIENT_SECRET`, and
`PANW_MGMT_TSG_ID` environment settings are supported. Keycloak sign-in does not
supply management credentials. See the bundled `PRISMA-AIRS-CLI.md` for setup,
capabilities, known CLI limits and the compatibility policy.
