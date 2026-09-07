# Keycloak identity foundation — rollout gate still open

The alpha.6 candidate implements public-client authentication and role-based
inference/MCP authorization in a dedicated AIRS workspace. Its live native-client
acceptance passed; optimized artifact acceptance and user-audit correlation remain
open. The installed version 0.1.0-alpha.5 still uses its working workspace credential. No teammate has been granted
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
`x-portkey-api-key`; the alpha.6 inference and MCP credential helpers both retain
the original signed token in that transport.

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
MCP now has a separate resource/client/scope and its own native user login,
validated by the newer CLI acceptance fixture.

The initial refresh and routing findings below have been resolved on the server:

- The gateway now runs the original-JWT identity filter on both replicas. Every
  gateway Service targets its filtered ports; NetworkPolicy blocks ordinary pod
  access to the raw listeners. Default inference omits `model`; explicit routes
  retain `@provider/model`. Routing override headers and body fields are rejected.
- Keycloak 26.2.4 uses an opt-in, database-backed single-use refresh ledger for
  Terminal clients. Realm-wide behavior is unchanged. Rapid replay, concurrent
  replay, transaction rollback and full replica restart tests passed. This uses
  internal Keycloak SPIs: pin the image and rerun protocol tests before upgrading.
- A distinct `airs-terminal-mcp` public client in **the same `truffles` realm**
  requests audience `airs-terminal-security`, scope `portkey.mcp.invoke` and role
  `scanner-user`. The original resource JWT reaches native AIRS signature
  verification. A successful live scan and invalid-signature, audience, client,
  scope, role and inference-isolation checks are recorded in
  `validation/2026-09-07/auth-release/mcp-jwt-acceptance.json`.

Live deployment: identity-filter v0.1.1, digest
`sha256:98c3fa682331624ba201fb18f339675a97150060f8628409fbf561a6aea3ef8e`.
The Terminal scanner endpoint is
`https://mcp-airs.cdot.io/ws-prisma-ff3d74/airs-terminal-runtime-scanner/mcp`.
Only `pan_inline_scan` is enabled. Clients remain disabled and the isolated
server's default user access remains denied until packaged-client acceptance.
No broad teammate role grants have been made.

Role removal denies the next freshly issued access token. Existing access JWTs
may remain valid for their 120-second lifetime. Logout revokes refresh credentials
and disables local helpers; stop running sessions to discard cached access tokens.
The original failed trials remain historical evidence, not current status.

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

## Remaining release gates

The Rust client implements browser S256/state/nonce and device login, signed
identity/resource verification, refresh rotation with durable pending state, and
stable history binding to issuer/client/subject/resource. The OS store holds token
bundles; config files contain only nonsecret identity and binding metadata.
Interrupted refresh requires login rather than risking replay of a consumed token.

Linux Secret Service, macOS Keychain and Windows Credential Manager all passed
native persistence checks in three separate processes, along with the identity
and interrupted-write tests. See `native-platforms.json` and GitHub Actions run
34147583219. These are native credential-store checks; the distributed full
terminal executable is still Linux-only. Bundles are chunked below Windows entry
limits and committed through a generation manifest. An ambiguous OS write or
cleanup failure can leave unreachable encrypted chunks; they are never treated
as an active credential without the committed manifest. No plaintext fallback
is implemented.

The unoptimized alpha.6 candidate passed 22 live CLI checks including browser/device
login, inference, separate MCP identity, local files, a real scan, refresh, logout
and user/history isolation. The 323 affected Rust checks and scoped Clippy passed.
Before release: repeat executable and live acceptance on the optimized artifact,
then publish/install and grant the owner access. Server and crate-level receipts alone do not satisfy this
gate. The installed alpha.5 client is still the earlier workspace-key release.

MCP OIDC setup requires the same verified user and issuer as inference with a
distinct public client and audience. Reauthentication preserves its binding ID;
a different user/resource requires a new environment. MCP helper commands use
platform-specific quoting, including encoded PowerShell on Windows.

Conjur manifests and a cutover/rollback runbook are prepared in the separate
`talos-cluster` checkout under `airs-terminal-identity/conjur/`. Kubernetes server
dry-run accepts them. Conjur policy authorization and secret retrieval have not
been tested; they are not activated. The documented admin escrow is unavailable
to the provided operator accounts. Their scope is three Terminal workload
secrets, not users' local refresh tokens.
