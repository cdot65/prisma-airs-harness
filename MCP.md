# Prisma AIRS MCP through AI Gateway

Both inference and remote MCP must target Prisma AIRS AI Gateway. The existing
Codex MCP client runs inside the normal `airs` executable. The gateway
proxies upstream MCP servers and owns their upstream OAuth tokens. CAS/Keycloak
provides gateway-facing organizational login.

The published release is onboarding.4. The in-session manager below is a
**development-branch feature**, pending release acceptance and publication.
See [PUBLICATION.md](PUBLICATION.md) for published artifacts.

## In-session workflow (development candidate)

Start `airs --environment work` using your existing inference environment, then:

1. Enter `/mcp` and choose **Add gateway MCP server**.
2. Enter a local connection name, such as `service-now`, and your administrator's
   HTTPS AI Gateway MCP URL. The manager requests `mcp:servers:read`,
   `mcp:tools:list` and `mcp:tools:call`. That local name does not select a gateway
   workspace; your gateway access and credential determine authorization.
3. In the sign-in dialog, press **Ctrl+O** to open company SSO, or **Ctrl+Y** to
   copy the full link to a browser on another device. Choose the intended company
   account and complete any gateway-managed upstream consent. Workspace API key
   inference can stay signed in while you authorize MCP as an individual user.
4. On the same machine, the browser callback completes automatically. Over SSH,
   paste the full callback URL into the dialog even if the browser says its
   loopback page cannot load, then press Enter. Input is hidden and stays out of
   conversation history. Escape cancels; a connection already saved is retained.
5. After tool discovery succeeds, choose **Start new conversation**. Your prior
   conversation remains saved and your draft is preserved for review. Nothing is
   submitted or replayed automatically.
6. Ask for a read-only ServiceNow incident lookup and inspect the actual tool
   result. A connected label alone does not verify the full ServiceNow workflow.

For an existing connection, `/mcp` offers **Sign in**, **Reconnect and verify**,
**Sign out** and **Remove connection**. Sign-out and removal require confirmation.
Sign out before removal if you also want to clear the local MCP credential.
Neither action signs out inference or revokes gateway-managed upstream grants.
The development `/doctor` dashboard brings environment diagnostics and MCP
recovery into the session. Open it to see the active environment, configured
authentication method, credential cleanup, gateway health and MCP discovery
state. Select a server to sign in or reconnect using the existing MCP manager.
These checks do not replay conversation requests or tools.

**Verify gateway access** asks for confirmation before sending one fixed
connectivity message, capped at 16 output tokens. It sends no conversation,
local files or tools, and asks the provider not to store the response; gateway
logging policy still applies. A successful health response or saved credential
alone does not establish inference access.

For native inference credentials, the bounded storage check reads a random
diagnostic account without writing, deleting or recovering credentials. A
service response does not prove the saved credential is unlocked. If cleanup
is pending, restore access to the user credential service and retry login or
logout in the same environment. Workspace-key replacement still uses
`airs --environment NAME login`; company SSO recovery is available in-session.
Esc cancels diagnostics while preserving the conversation and draft.

`/doctor` is a development feature here; the published `mcp.1` package predates it.

`/mcp verbose` still displays the tool inventory. `/signin` also opens the same
MCP sign-in dialog for configured OAuth connections.

Changes remain bound to the running session's environment even if another
terminal changes `airs env use`. Finish or interrupt an active request before
managing connections. After an attempted change, use `/mcp` to retry or `/new`
to continue with freshly loaded connections. A remote app-server session must
manage credentials on its server host. The manager supports adding HTTPS OAuth
gateway connections; existing local or bearer-authenticated servers do not offer
browser sign-in.

The manager uses the existing credential store. An unlocked native store is
still required when `mcp_oauth_credentials_store = "keyring"` is configured.
No API key, SSO token or upstream integration password belongs in the URL field.

## Onboarding prerequisites

Provision a gateway MCP integration scoped to the harness workspace, configure
its separate upstream OAuth client, and verify CAS/CIE identity and workspace
access. The native client must use the connection URL supplied by that gateway
integration. An upstream resource URL is not a valid substitute.

Keep the existing named inference environment. Add the verified gateway URL
with native `mcp add` once your administrator has provisioned the integration.
For example, using a fictional deployment:

```sh
airs --environment work mcp add prisma-airs \
  --url https://mcp-gateway.example.com/workspace/prisma-airs/mcp \
  --scopes mcp:servers:read,mcp:tools:list,mcp:tools:call
```

Native OAuth discovery handles gateway client registration and browser login.
Use gateway scopes supported by its discovery document. Do not supply the old
upstream Keycloak client ID, upstream read scopes or an upstream resource override
to this gateway-facing registration. The gateway requests `airs.gateway.read`
and `airs.profiles.read` separately using its own upstream client.

The intended human must have gateway workspace access and upstream resource
permissions. A shared SSO browser session does not make the tokens interchangeable.
The stock MCP client does not compare its identity with the inference identity.

Set `mcp_oauth_credentials_store = "keyring"` at the top level of the environment's
`config.toml` when file fallback is prohibited. macOS requires an unlocked login
Keychain; Linux requires an unlocked Secret Service session. Native `mcp login`,
`mcp list`, `/mcp` and `/mcp verbose` remain the connection controls.
`--no-browser` prints the authorization URL for manual opening and accepts a
hidden pasted callback URL, including when a remote browser cannot reach the
process's loopback callback port.

`mcp logout` removes the local gateway-facing credential. It does not sign out
inference or revoke gateway-managed upstream tokens and Keycloak sessions.

## Request path

```mermaid
flowchart LR
  User[Human] --> Harness[airs with native Codex MCP client]
  Harness -->|Inference credential and tool declarations| Gateway[Prisma AIRS AI Gateway]
  Gateway --> Model[Authorized model route]
  Harness -->|MCP requests with gateway-facing OAuth token| Gateway
  Harness -->|Gateway browser authorization| CAS[CAS and Keycloak SSO]
  CAS -->|Gateway-facing login| Harness
  Gateway -->|Gateway-managed upstream OAuth| IdP[Upstream authorization server]
  Gateway -->|MCP requests with upstream user token| MCP[prisma-airs-mcp upstream]
  Policy[Human roles and resource bindings] --> MCP
  MCP -->|Dedicated backend read credentials| AIRS[AIRS management APIs]
```

The inference adapter flattens tool namespaces on the wire and restores them
before native dispatch. Canonical conversation history retains its namespaces.
Alpha.14 fixes inference history preservation when native MCP uses dynamic client
registration without an explicit OAuth client ID. Legacy helpers and explicit
static credentials remain pinned to their existing session binding.

## Release acceptance

`scripts/validate_gateway_mcp.py` drives native interactive SSO and tool checks.
The promotion gate requires the exact installed Linux and Apple Silicon packages,
two real frontend expiry cycles, history preservation, npm upgrades and Mac
signing evidence. Separately collected gateway/CAS/upstream observations bind
the route, client and run window. Each run needs a unique MCP server name because
the OS-user keyring is shared across isolated harness homes.

Parallel runs using the same browser also share a Keycloak client session.
Revoking one inference refresh token removes that client session and interrupts
the other run. Supply a distinct `--cleanup-barrier` directory to each peer.
After every peer has written `ready.json`, the controller writes `release.json`
containing only that peer's `nonce`. A failed peer must wait too. The runner
performs native logout after release; an unreleased barrier fails acceptance and
still cleans up after three hours. Keep the controller active until both peers
finish cleanup. Credential metadata is collected even if a post-expiry tool call
fails; it never substitutes for successful tool results.

Run `scripts/coordinate_gateway_cleanup.py --local /linux/run/cleanup --remote
/mac/run/cleanup --ssh-host user@mac` alongside both validators, using those
paths for their respective `--cleanup-barrier` arguments. Add `--identity-file`
when desktop SSH needs a file key instead of an agent.

Use the normal npm registry update after an accepted release is published. An
older manual command symlink can shadow npm; inspect `command -v airs-harness`
and `npm prefix -g`. Follow the existing installation handover procedure only for
that known legacy layout. Preserve the inference environment and old executable.


The interactive runner takes an exact native executable, a new isolated state
folder, the gateway and upstream URLs, an output receipt and a
`--gateway-evidence` JSON file. On Linux, run it under `dbus-run-session` and use
`--browser-via-ssh cdot@desktop-host` when the browser is on an SSH-reachable Mac;
it forwards only the native loopback callback and terminates that tunnel afterward.
The human completes the browser login; no password is collected by the runner.

The observations file must bind `endpoint`, `upstream_endpoint`,
`binary_sha256` and `started_at` to the printed run window. Its `results` must
contain the gateway/CAS cases in the promotion contract, plus
`gateway_native_refresh_observed`, each with a measured `evidence` reference.
Preserve timestamped, redacted request/correlation evidence separately. A success
boolean with no observation is insufficient.

The inspected gateway 2.22.0 issues one-hour opaque MCP access tokens. The runner
therefore waits two real one-hour expiry intervals for `--refresh-cycles 2`;
upstream Keycloak tokens expire separately after five minutes. A zero-cycle smoke
run is incomplete and cannot pass the standard lifecycle promotion gate. Both
exact installed alpha.14 packages passed production native login, credential
storage and all eight tools with gateway/upstream correlation. For alpha.14 the
owner explicitly authorized publication without another hourly test. The
exception records zero completed frontend expiry cycles and preserves the failed
idle-session receipts. See [PUBLICATION.md](PUBLICATION.md).


## Active sessions and idle reauthentication

The deployed Keycloak realm has a 30-minute SSO idle timeout. Observed upstream
refresh grants expire after 30 minutes without renewal, even though a
gateway-facing MCP access token lasts an hour. A valid frontend token alone
does not ensure that the gateway can renew an expired upstream grant. Prolonged
inactivity can require fresh inference and MCP login.

The release driver tests active use across two real gateway expiries. It performs
authorized reads every ten minutes while outside the final pre-expiry quiet
window. Each read must leave the frontend token and expiry unchanged. Fresh
concurrent processes then run after the actual one-hour expiry. This tests
renewal under normal activity; it does not claim an hour of idle SSO continuity.
The earlier silent-wait runs are failed receipts, not release acceptance.
