# Identity decision — one realm and one JWKS endpoint

The owner selected the existing stack on September 7, 2026. AIRS Terminal uses
`https://auth.dev.cdot.io/realms/truffles` and its existing
`/protocol/openid-connect/certs` endpoint. The earlier dedicated-realm and
credential-translation proposal is rejected. Do not create another realm,
broker Terminal through another issuer, combine JWKS documents, or exchange a
user JWT for a shared gateway credential.

## Stack implementation

The existing public client `airs-terminal-pilot` stays in `truffles`. Membership
in `/stacks/airs-terminal/users` grants its `terminal-user` client role. The
signed role mapper and mandatory AIRS workspace JWT guardrail enforce that role;
local configuration and caller metadata cannot grant it. The group is currently
empty and the pilot remains disabled outside controlled acceptance runs.

Authoritative group reconciliation and refresh-policy tests live in
`cdot65/talos-cluster`, under `keycloak/stacks/terminal/`. The reconciler modifies
only Terminal groups and their existing client-role mapping. It does not run the
older whole-stack migration, recreate retired applications, change client secrets,
or change the realm/JWKS configuration.

Inference keeps the original signed user JWT through AIRS. Issuer, audience,
authorized client, subject, operation scope and workspace roles are distinct
checks. MCP must use its own resource audience and role; inference permission
must not imply MCP permission. No local Terminal operation needs a PAH service.

AIRS owns provider credentials, model routing and mandatory scanning. Terminal
omits `model` for the gateway default route and sends `@provider/model` only for
an explicit selection. The terminal retains local files, tools, approvals,
skills and sessions.

## Refresh-policy finding

Keycloak 26.2.4 implements refresh-reuse enforcement at realm level. A controlled
rollout tested all 15 enabled browser-capable OIDC clients with registered
callbacks, using one temporary user and no application callback visits. Baseline
code exchange, two successive refreshes and the expected existing replay behavior
passed for all 15.

With `revokeRefreshToken=true` and `refreshTokenMaxReuse=0`, normal refresh still
passed for all 15, but an older token was accepted after two rapid refreshes for
13 clients. The transaction **restored the previous settings** and deleted its
user. Rotation is therefore a proposed policy, not an active or passing control.
The preserved failing receipt is `keycloak/stacks/terminal/refresh-policy-receipt.json`.

The deployed source compares token issue times at second resolution when checking
older tokens; this is a possible explanation for the rapid-sequence result, not
a proven attribution to a particular CVE. Checking the published 26.7.3 source
found the same timestamp comparison, so an upgrade alone has not been established
as the fix. Do not add sleeps to make the replay test pass. Keycloak's
`suppress-refresh-token-rotation` executor only suppresses replacement tokens in
the response; it does not provide isolated per-client replay enforcement.

These are IdP protocol tests. They do not exercise the applications' own token
storage, refresh concurrency or browser UIs. SAML and service-account grants
are outside this refresh test, as are browser-disabled/no-callback clients.

## Remaining implementation gates

1. Resolve refresh replay within this issuer, with rapid replay, concurrency and
   restart tests. Retain replacement tokens atomically under an OS-store lock in
   the terminal; require a new login after detected replay/session revocation.
2. Enforce routing/header restrictions on the original JWT path. AIRS 2.20.0
   drops JWT `defaults.allow_config_override` while constructing local auth
   details. Mandatory workspace identity and scanner guardrails survive routing
   replacement, but that does not prove strict config binding. Any stack-level
   enforcement must cover every reachable gateway address and preserve the
   user's JWT; a check only in the local CLI or an optional hostname is insufficient.
3. Implement browser PKCE/state/nonce, device authorization, verified identity,
   OS credential storage, refresh/logout and issuer/client/subject/resource
   history binding in Rust. Workspace-key environments remain supported.
4. Prove separate MCP authorization and persisted AIRS user audit attribution,
   then test packaged Linux login, expiry, resume and local-tool execution.

The installed alpha.5 binary still uses workspace credentials. This decision and
its deployed group mapping do not constitute the completed identity milestone.
Conjur remains an optional workload-secret integration; user refresh tokens
belong in the local OS credential store.
