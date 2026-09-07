# Keycloak identity foundation — rollout gate still open

The September 7 native pilot proves public-client authentication and role-based
inference authorization in a dedicated AIRS workspace. It does **not** implement
Keycloak login in the installed `airs-terminal` binary. Version 0.1.0-alpha.5
continues using its working workspace credential. No teammate has been granted
the pilot role; synthetic acceptance users are deleted after every run. The
pilot client is disabled between operator acceptance runs.

The owner requires one realm and one JWKS endpoint. The accepted implementation
stays in the existing `truffles` stack; see the [single-realm decision](boundary-proposal.md).
The earlier new-realm/credential-translation proposal is rejected.

## Deployed foundation

| Resource | Value |
| --- | --- |
| Keycloak issuer | `https://auth.dev.cdot.io/realms/truffles` |
| Public client | `airs-terminal-pilot` |
| Client UUID | `32a46109-1ed6-450b-b3cc-be5a54666953` |
| Required client role | `terminal-user` |
| Access group | `/stacks/airs-terminal/users` (currently empty) |
| Inference audience | `airs-terminal-inference` |
| AIRS workspace | `f4aca25e-fe23-4cae-bca7-91f8f3c78594` / `ws-prisma-ff3d74` |
| Routing default | `774fed74-6cc0-419d-8f71-54442f748e88` / `pc-termin-943065` |
| Mandatory JWT guardrail | `f432b9ec-7325-40d9-a37b-84e3e3c9d844` / `pg-termin-da0e73` |
| Mandatory compatibility guardrail | `pg-termin-b06244` |
| Mandatory scanner guardrail | `pg-termin-054976` (input and output) |
| Workspace provider binding | `8cde4076-049e-4e57-8da6-00649df4f8d5` / `openai-terminal-auth` |

The previously created workspace lacked its SCM scope. Added
`ws_airs_terminal:1852583913` for workspace resource `ws-prisma-ff3d74` and scoped
access policy `27d7ce85-433d-489e-b80a-b18f686496c3` for the existing operator
`coder-cli@1852583913.iam.panserviceaccount.com`. Its existing role is `superuser`;
the new assignment is limited to this workspace. No teammate receives this role.
The OpenAI integration's Terminal workspace binding was disabled; enabled only
that binding, preserving all other enablement flags and limits by readback.

`keycloak-pilot-client.json` records public-client settings and protocol mappers.
Create the `terminal-user` client role and include it in this client's allowed
role scope mappings. Default client scopes are `portkey.completions.write` and
`portkey.logs.write`; neither grants management or MCP invocation. Do not grant
the role by default to realm users. The dedicated `oidc-sub-mapper` is necessary:
this realm does not provide the normal `basic` scope and otherwise omits access
JWT `sub`. Roles are mapped from actual assignments into `airs_roles`, never from
user-editable attributes or caller headers.

The client uses S256 PKCE, loopback callback registration
`http://127.0.0.1/callback` (a dynamic port was verified), 120-second access tokens,
15-minute client idle and one-hour maximum session timeouts. Password grants,
implicit flow, service accounts and offline access are excluded. On this
Keycloak deployment, the device endpoint also requires S256 challenge parameters
and the verifier on token polling when the client mandates PKCE; both were
exercised successfully. The test's HTML
login form submits a synthetic user's password to Keycloak over TLS; the terminal
will open the browser and will never collect that password itself.

`gateway-jwt-check.json` is the in-process mandatory check. Signature, issuer,
audience, authorized party, subject, lifetime, organization, workspace, operation
scope, role and routing-default claim presence are required. It reads
`x-portkey-api-key`; the future JWT credential adapter must support that transport
instead of assuming the existing Bearer-only credential helper is compatible.

## Live acceptance and limits

Run the operator fixture with an owner-only Keycloak admin access-token file:

```sh
uv run --script scripts/validate_keycloak_pilot.py \
  --admin-token-file /absolute/private/admin-token \
  --output /absolute/path/public-client-receipt.json
```

The fixture requires an initially disabled pilot with no role members, enables
it for the run, then disables it during cleanup. It creates two synthetic users,
joins only one to the stack access group, verifies
OIDC signatures/issuer/audience/state/nonce, and deletes both users in `finally`.
It exercises gateway scans, config/header spoof attempts, PKCE rejection, device
approval, refresh, role removal and logout. No passwords, codes, refresh tokens,
access tokens or admin tokens enter its receipt. The checked-in result is
`validation/2026-09-07/single-realm/public-client.json`.

This is an operator protocol test, not a browser UI automation suite or installed
CLI acceptance. Two signed subjects are verified; correlation to persisted AIRS
telemetry is still required before claiming end-to-end user audit attribution.
MCP remains a separate resource/scope and has no per-user login implementation yet.

Two rollout gates remain:

- **Refresh replay succeeds.** Keycloak 26.2.4's shared `truffles` realm has
  `revokeRefreshToken=false`. The fixture deliberately reports failure when the
  old refresh token is accepted. The controlled realm-wide rotation trial tested
  all 15 browser-capable clients: normal refresh passed, but rapid replay still
  succeeded for 13. The rollout restored `revokeRefreshToken=false`. The
  single-realm decision records the failing evidence; this remains a real gate.
- **Routing configs remain overridable.** Mandatory workspace identity/scanner
  hooks survive inline replacement, including explicit empty hook arrays. This
  repairs the scanner-policy bypass, but does not meet strict routing-config
  binding or establish rejection of every provider/custom-host override.

Role removal denies the next freshly issued access token. Existing JWTs may stay
valid until expiry; logout prevents refresh but does not promise immediate
revocation of an already issued bearer token. No terminal role is distributed
while the two gates above remain open.

## API details established live

Workspace default writes accept guardrail **slug strings**. AIRS stores UUIDs and
expands them into objects internally. Sending objects with `slug` on the write
caused a control-plane 503; sending strings succeeded and survived readback.
Workspace policies are enforced independently of config-attached hooks.

Integration access is updated with `workspaces: [{id, enabled}]` and
`override_existing_workspace_access: false`, preserving other bindings. SDK
0.17.0's documented boolean `global_workspace_access` write is inaccurate for
this endpoint. Reading the integration workspace list also fails that SDK schema
when `last_updated_at` is null. The targeted operation used authenticated REST and
verified the unchanged sibling bindings. Do not broaden workspace access to
work around the SDK discrepancy.

## Next implementation stages

1. Resolve strict gateway enforcement and refresh replay within the existing stack as described in
   [the single-realm decision](boundary-proposal.md). Retest raw requests
   before granting users access.
2. Add isolated Rust OIDC modules using a maintained OIDC library: discovery
   validation, browser S256/state/nonce, verified ID-token subject and access-token
   audience, device polling, OS-store refresh persistence, locked/atomic refresh,
   logout/revocation and cancellation. Bind history to issuer/client/subject/
   resource, not rotating token bytes. Keep workspace-key environments supported.
3. Add server-authorized per-resource MCP login. A workspace key, inference JWT
   and MCP token must not substitute for each other. Add two-user telemetry and
   local-session isolation evidence.
4. Build and install the Linux release, run the packaged regression suite and
   hands-on TUI authentication/expiry/resume tests before marking M3 complete.

Conjur manifests and a cutover/rollback runbook are prepared in the separate
`talos-cluster` checkout under `airs-terminal-identity/conjur/`. Kubernetes server
dry-run accepts them. Conjur policy authorization and secret retrieval have not
been tested; they are not activated. The documented admin escrow is unavailable
to the provided operator accounts. Their scope is three Terminal workload
secrets, not users' local refresh tokens.
