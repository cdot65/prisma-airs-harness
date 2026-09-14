# Required MCP route and outstanding acceptance

Both inference and remote MCP must target Prisma AIRS AI Gateway. The built-in
Codex MCP client stays in `airs-harness`; it connects to the gateway's MCP
listener. The gateway proxies upstream servers and manages their OAuth tokens.
CAS/Keycloak federation supports gateway-facing user login. These two OAuth legs
have separate credentials and lifecycles.

## Deployment and acceptance checkpoint — September 14, 12:55 UTC

Development and production gateway integrations are deployed with dedicated
confidential OAuth clients and explicit workspace access. Both upstreams accept
only their gateway-owned client. CIE group-to-workspace mapping was repaired and
the owner confirmed membership. Both exact installed alpha.14 candidates now
pass native production login, credential persistence and all eight tools with
matching gateway/upstream logs. Signing, notarization, native tests and npm
upgrades pass. The silent-wait runner exposed a 30-minute Keycloak refresh-grant idle limit.
Its failed runs are preserved. The owner directed publication without another
hour-long test. Hourly frontend refresh is unverified, with zero completed
cycles; the package carries this explicit release exception. The active-session
runner is available for future validation but is not a release blocker under
the owner’s revised scope. The dated review findings below describe the initial state.

## Review findings — September 14, 2026

- The September 13 plan explicitly replaced the earlier gateway-mediated design
  with direct access. That recommendation is withdrawn by the owner's correction.
- Alpha.13 `MCP.md` and `validate_builtin_mcp.py` target `prisma-airs-mcp*.cdot.io` directly.
  The driver logs straight into Keycloak and checks upstream read scopes. Its
  results prove that experiment, not gateway routing or CAS.
- The deployed gateway 2.22.0 exposes `https://mcp.redtail.cdot.io` and publishes
  OAuth discovery, a registration endpoint and a protected-resource challenge.
  Its control-plane MCP integration list had no harness integration at review.
- Running code has an `oauth_auto` implementation, including configurable OAuth
  client metadata and a gateway upstream callback. Its unimplemented machine
  client-credentials branch is a separate mode and does not justify direct access.
- Verdaccio `latest` and `alpha` resolve to alpha.13. Its existing receipts remain
  historical. This correction does not republish packages or change production.

## Alpha.14 source changes

The isolated branch is based on the signed alpha.13 publication record at
`5b2f7c194`. Package metadata now targets alpha.14. Native HTTPS OAuth servers
using dynamic registration no longer invalidate the inference session binding
when added or removed; explicit credential alternatives remain pinned. Ten
focused session/relocation tests and eight promotion-gate tests pass.

Companion infrastructure and MCP application branches prepare workspace-scoped
OAuth Auto registrations, gateway-owned upstream clients, exact upstream-host
allowlist entries and resource-server client allowlists. Those changes are now deployed; measured package lifecycle results remain
separate, as recorded in the checkpoint above.

## Concrete remediation sequence

1. Register the read-only Prisma AIRS MCP service as a gateway upstream, scoped
   to the harness workspace. Keep global workspace access disabled. Verify the
   gateway's actual upstream reachability and private-host allowlist.
2. Provision a gateway-owned upstream OAuth client with the gateway callback,
   exact resource audience and read scopes. Configure gateway OAuth Auto using
   that registration. Preserve upstream object authorization; never weaken it to
   make a probe pass. Do not reuse the native loopback client's registration.
3. Verify the gateway's CIE directory, CAS authentication profile, selected identity
   attribute and harness group/workspace mapping. Complete browser login as the
   intended human. A related workspace's login is not this workspace's acceptance.
4. Configure the native MCP client with the connection URL supplied by the gateway
   for the provisioned integration. Discover gateway OAuth; require native token
   storage. Preserve the working inference environment and history. Do not copy
   upstream tokens to the harness or use the direct URL after a gateway error.
5. Replace the historical direct driver with a gateway/CAS driver. Observe and
   correlate gateway ingress and upstream requests, successful model-selected
   reads, denied access, gateway-facing refresh and gateway-managed upstream
   refresh. Verify from both Linux and Apple Silicon installed packages.
6. Promote only after the new evidence passes. The promotion validator now rejects
   the old receipt shape and requires explicit gateway/CAS/upstream evidence.
   Those receipt checks enforce the evidence contract; they do not generate live
   proof. No gateway acceptance producer or completed live result exists in this
   change, so the existing direct driver cannot promote a new MCP release.

## Evidence contract

The acceptance receipt must identify its actual client `endpoint` and a `routing`
object with `mode: gateway-proxied-mcp`, the matching `gateway_endpoint` and a
distinct HTTPS `upstream_endpoint`. Required result cases include
`gateway_cas_browser_login`, `gateway_mcp_request_observed`,
`gateway_upstream_request_observed`, `gateway_mcp_denial`,
`gateway_managed_upstream_oauth` and `gateway_managed_upstream_refresh`, in addition
to the existing native executable, tool, history, expiry and package checks.
Receipts must derive from measured requests and redacted correlation evidence;
do not add successful rows merely to satisfy the validator.

Primary references: [SCM CAS](https://portkey.ai/docs/product/mcp-gateway/authentication/cas),
[MCP authentication layers](https://portkey.ai/docs/product/mcp-gateway/authentication),
and [CIE Directory Sync](https://portkey.ai/docs/product/enterprise-offering/org-management/directory-sync/cie-directory-sync).


## Authorized rollout checkpoint

The owner authorized rollout after source preparation. Harness PR13,
infrastructure PR407 and application PR1 are merged. Gateway Helm revision 12
adds both upstream hosts while preserving inference settings. Both dedicated
OAuth Auto integrations and Keycloak upstream clients exist with explicit harness
workspace access. Argo has cut over development to the gateway-owned upstream
client; production resource-server cutover remains gated on development acceptance.

CAS initially denied the owner access to the harness workspace. Redtail CIE
readback confirmed the existing user and MCP group; the owner added the group
mapping, ran Full Sync and confirmed the workspace member in SCM. The general
workspace-detail API's `users` field is not a reliable check for those
Directory Sync members. Upstream Keycloak now shows the owner's session on the
gateway client. Native credential receipt and successful proxied tools still need
verification; a browser login alone is not completed MCP acceptance.

The first correction binaries compiled and passed native tests but retained the
alpha.13 version constant. PR14 corrects that constant and rejects native/npm
version mismatches before packaging. Frozen runtime `6195ca83e` is rebuilding for
both platforms; do not promote the earlier mismatched artifacts. The interactive
runner now exists, but independently correlated gateway observations and full
live acceptance remain pending. Alpha.14 is not published yet.
