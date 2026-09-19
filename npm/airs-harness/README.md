# Prisma AIRS Harness

The `airs` command runs a native Codex terminal agent. Inference and
remote MCP both use Prisma AIRS AI Gateway. Its built-in MCP client connects to
the gateway listener; the gateway proxies upstream servers and owns upstream
OAuth. No separate MCP executable is required.

## Prerequisites

Use Node.js **22.13.0 or newer in the 22.x line**, or **23.5.0 or newer**
(`^22.13.0 || >=23.5.0`). Check the versions in the same terminal where you
will install and run AIRS:

```sh
node --version
npm --version
```

Installing npm alone does not upgrade Node.js. If Ubuntu supplies Node 18,
install a supported Node.js version using your organization's usual method,
open a new terminal and check again before installing the harness. npm may
otherwise finish installation with an `EBADENGINE` warning; the AIRS launcher
rejects unsupported Node versions before starting either the harness or its
bundled product CLI.

## Branded onboarding

Release `0.1.0-alpha.22.onboarding.4` distinguishes saved credentials from verified gateway access, reports HTTP status and a gateway-searchable trace ID, and rejects guardrail denials even when the gateway returns HTTP 200. Local environment names are independent of gateway workspace names; a workspace API key selects its gateway workspace. Both SSO and workspace keys require the corresponding gateway policy. The npm distribution targets Linux x64, Linux ARM64 and signed Apple Silicon at `https://npm.cdot.io`.

```sh
npm install -g airs-harness@0.1.0-alpha.22.onboarding.4 --registry=https://npm.cdot.io
airs --version
airs cli --version
airs
```

The product CLI and eight skills are bundled. Normal installs do not require `--include=optional`. Returning signed-in users enter the agent directly. [Release evidence](https://git.cdot.io/cdot/prisma-airs-harness/src/branch/feat/airs-branded-onboarding/validation/2026-09-18/workspace-key-release/README.md) records native acceptance and remaining attended production checks. The earlier review archive on [PR 47](https://git.cdot.io/cdot/prisma-airs-harness/pulls/47) remains available as a separate installation path.

## Install or update

On the organization's LAN/VPN, install from Verdaccio:

If the old standalone Prisma AIRS CLI already owns `airs`, upgrade it **first**:

```sh
npm install -g @cdot65/prisma-airs-cli@7.0.1 --registry=https://registry.npmjs.org
airs-cli --version
```

A fresh machine needs only the harness installation below. It includes CLI 7.0.0
as `airs cli` and the Prisma AIRS product skills.

```sh
npm install -g airs-harness@0.1.0-alpha.22.onboarding.4 --registry=https://npm.cdot.io
airs --version
airs cli --version
```

Alpha.22 includes Linux x64, Linux ARM64 and Apple Silicon native packages. The package uses anonymous downloads;
no npm login is required. Optional dependencies carry the matching native binary.
The launcher does not compile or download code at startup and has no install
scripts. See the bundled [Mac guide](MACOS.md).

Alpha.21 adds `env create/list/show/status/use/rename/remove` and removes the
old top-level `setup` and `status` commands. `env use` changes the saved default;
`--environment` selects one command. Renaming preserves environment identity;
removing unregisters the environment and retains its local files.

Follow the [complete SSO-to-ServiceNow walkthrough](https://cdot65.github.io/prisma-airs-reference-architecture/learn/login/#sso-to-servicenow-a-complete-first-session)
to create an environment, sign into inference, add the gateway ServiceNow MCP
connection and authorize it with the same company identity. The two grants
remain separate. Verify a read-only incident tool result after login.

Alpha.20 adds manual callback input to `airs mcp login <server> --no-browser`.
Open the printed authorization URL on your browser host, complete sign-in, and
paste the full callback URL into the terminal running that command. Input is
hidden and validated; a working HTTP callback can still finish the same flow.
Never paste the callback into a conversation or share it with another person.
This release also corrects OAuth status reporting, clears MCP interaction state
on reconnect, and supports same-issuer OIDC discovery after an OAuth metadata 503.
The 30-minute idle policy and uncertain-refresh protections are unchanged.

Alpha.19 fixes Linux musl DNS lookups when one address family resolves and the
other returns NXDOMAIN. It uses the configured DNS servers, keeps IPv4 and IPv6
available, and retains transport error details when issuer discovery fails.

Alpha.18 adds the Linux arm64 package to the release and distinguishes an
unsupported architecture from an omitted optional dependency.

Alpha.17 adds guided gateway MCP sign-in inside the terminal and retains a definite
expired/rejected inference grant classification across later credential reads.
Alpha.16 fixed the normal inference provider path so it displays sign-in guidance.
The deployed 30-minute SSO idle policy is preserved. The client attempts silent renewal during active
use while grants remain valid; production active-expiry acceptance is incomplete; leaving a terminal open does not request
a longer session. Release evidence records which gateway lifecycle scenarios
were actually verified.

Ordinary npm upgrades preserve configuration and history without uninstalling or
using force. Fresh state uses `~/.airs-harness`; existing `~/.airs-terminal` state
is reused when the new directory is absent. `AIRS_HARNESS_HOME` overrides the
default, with `AIRS_TERMINAL_HOME` retained as a compatibility fallback. If an old
manual executable shadows npm, inspect the resolved command and npm prefix;
preserve old targets referenced by stored credential bindings.

## Workspace API keys and environments

An environment is a local profile for one gateway connection, its credentials,
MCP configuration and session history. Create one for a separate connection or
identity; reuse it for later sessions. Its name does **not** need to match an AI
Gateway workspace name. A workspace API key belongs to the gateway workspace
where you created it, independently of the local name.

In Strata Cloud Manager, open AI Security → AI Gateway, select the intended
workspace, and create a user workspace API key with inference permission
(`completions.write`). Apply the required expiry and usage limits. That workspace
also needs a default saved config/model route, or an explicit route supplied by
your administrator. Its gateway policy must permit workspace keys alongside SSO.

For a new local profile:

```sh
airs env create workspace-api --gateway-url https://gateway.example.com/v1
airs --environment workspace-api login --with-api-key
```

Explicit creation with `--gateway-url` saves the connection settings without
signing in. The separate `login --with-api-key` command asks for the key in a
hidden prompt. For guided creation and a choice of authentication methods,
run `airs env create` without arguments.

If the profile already exists, reuse it:

```sh
airs --environment workspace-api login --with-api-key
airs --environment workspace-api doctor --verify-access
airs --environment workspace-api
```

A saved credential is not yet proof of gateway access. Verification sends one
small inference request. HTTP 401 indicates credential rejection; 403 indicates
an authorization denial; 446 indicates a guardrail denial. Some gateway policies
return a blocking denial with HTTP 200; the harness checks the hook results too.
Use the reported request/gateway trace ID when asking the gateway administrator
to inspect the request. Creating another local environment does not change the
key's workspace or fix its policy.

```sh
airs env list
airs env use workspace-api
airs --environment work doctor --verify-access
airs env use work
airs env remove workspace-api
```

Removing an environment unregisters its local name and preserves its files. It
does not delete the gateway workspace or revoke its API key. Select another
environment before removing the default.

MCP authorization is separate: add the gateway-provided MCP URL in this same
environment, then complete `airs --environment workspace-api mcp login SERVER`
if the add flow did not already sign you in. Company SSO and any upstream
ServiceNow consent still apply; an inference API key does not grant MCP access.
See the [getting-started guide](https://cdot65.github.io/prisma-airs-reference-architecture/learn/login/)
for the complete SSO and ServiceNow flow.

## Connect inference and MCP

Keep an existing named inference environment. For a new connection:

```sh
airs env create work --gateway-url https://gateway.example.com/v1
airs --environment work login
airs --environment work doctor --verify-access
```

Use your administrator's inference URL and company issuer/client/audience, or
choose a workspace API key for inference. A workspace key is not the MCP user
credential. With a configured gateway integration:

```sh
airs --environment work mcp add mcp-server-1 \
  --url https://gateway-mcp.example.com/mcp-server-1/mcp \
  --scopes mcp:servers:read,mcp:tools:list,mcp:tools:call
airs --environment work mcp list
airs --environment work
```

Replace the example with the gateway-provided MCP URL and discovery-supported
scopes. Complete gateway CAS/company SSO and any upstream consent. Your directory
group needs the gateway workspace mapping, and your upstream identity needs the
`utilities.use` scope, invoke role and subject binding for mcp server 1.
Its eight tools compute on the MCP server without calling management APIs.
Upstream OAuth client secrets stay at the gateway.

macOS uses Keychain; Linux requires an unlocked Secret Service session. Installing
npm does not provision a Linux keyring. Set `mcp_oauth_credentials_store =
"keyring"` at the top of the environment's `config.toml` to require native MCP
storage. Confirm CLI completion and a tool call after browser consent. The
callback expires after five minutes; headless tests require a tunnel from the
browser's loopback port to the native process.

Use `/mcp` in the terminal for inventory, and `airs resume` for history.
`mcp logout mcp-server-1` signs out native MCP separately from inference `logout`;
it does not revoke the gateway's upstream user grant. Git, ripgrep and project
tools remain prerequisites; Linux also requires usable Bubblewrap.

## Renewing sign-in

When inference needs fresh company sign-in, use `/signin` in the open terminal.
The harness verifies the same issuer, client, audience and subject before saving
credentials. It keeps your conversation and draft, and does not replay completed
requests. Cancel leaves your work in place. In another terminal, the equivalent
command is `airs --environment work login --restore-session --no-browser`.
Use the environment name shown by your running session.

MCP authentication is separate. On an MCP authentication failure, choose **Sign
in** in the terminal, or use `/signin` and select the affected connection. The
harness opens gateway consent, saves native credentials and verifies MCP
initialization and tool discovery before reporting a connected tool count.

Choose **Start new conversation** afterward. Your previous conversation stays
saved and your unsent draft carries over for review, without automatic submission
or tool replay. The current gateway integration does not provide trusted account
continuity for restoring the old MCP conversation. Cancel keeps your work in place;
no separate terminal command is needed for the normal desktop flow. Headless
terminals retain the displayed `mcp login --no-browser` fallback.

If credential access fails, open `/doctor` in the running environment, or run
`airs --environment work doctor` using your environment's name. An unavailable
service does not establish that the store is locked. Check the credential service
and any operating-system authorization prompt in the same user session. If
cleanup is pending, restore access and retry login or logout for that environment;
the harness retains the original failure alongside any cleanup failure.

A pre-exchange identity-service outage preserves the saved refresh grant. If an
exchange may have consumed a rotating grant, the harness will require sign-in
instead of blindly replaying it.
Restart running terminals after upgrading so they use the new native client.

## Managed Prisma AIRS CLI

This release pins `@cdot65/prisma-airs-cli@7.0.0` and includes eight
built-in Prisma AIRS skills. Run `airs cli doctor --output json` before
Prisma AIRS operations. CLI 7 uses the selected tenant JSON configuration;
credential environment variables are ignored. Register existing protected config
files with `airs cli tenant create NAME --config PATH`. Harness environments and
CLI tenants have independent selections. Keycloak sign-in does not supply product
management credentials. See the bundled `PRISMA-AIRS-CLI.md` for setup,
capabilities, known CLI limits and the compatibility policy.
