# Authentication recovery implementation

Status: alpha.15 candidate implementation; native package acceptance is pending.
This document records source findings and validation boundaries; it is not a
claim that guided reauthentication or full gateway lifecycle acceptance shipped.

## Ownership

Both inference and native MCP target Prisma AIRS AI Gateway. The harness owns
its inference OIDC credential and gateway-facing MCP credential. Gateway/CAS
owns upstream MCP OAuth. The MCP service owns its separate management API
service accounts. Device authorization changes initial login, not renewal.

The production SSO idle timeout remains 30 minutes. Renewal is demand-driven;
an idle terminal must not send synthetic traffic or refresh on a keepalive timer.

## Gateway contract evidence — September 14, 2026

Both live gateway replicas report enterprise version 2.22.0. Read-only discovery
against the development integration returned HTTP 401 with a Bearer
`resource_metadata` challenge. Authorization metadata advertises PKCE S256,
authorization-code and refresh grants, and the gateway's `/oauth/authorize`,
`/oauth/token`, `/oauth/revoke` and `/oauth/register` endpoints.

Inspection of that deployed version's JavaScript identifies two upstream-only
authorization signals:

- JSON-RPC error code `-32000`, with `data.type = upstream_auth_required`.
- Session initialization returns that error with HTTP 401 and a Bearer challenge.

The upstream-auth error path revokes downstream OAuth tokens associated with the
frontend client. A pending-authorization path can include `authorizationUrl`, but
the failure path does not consistently supply it. These are source findings,
not yet authenticated expiry-test receipts. Recovery must use bound gateway
discovery; never automatically open an arbitrary URL from a tool response.

The opaque frontend token response does not establish account continuity. The
gateway has an internal introspection method with identity fields, but discovery
does not advertise a client-facing introspection or user-info endpoint, and the
inspected router has no `/introspect` route. Automatic post-login replay remains
gated on trusted identity proof. A display name, opaque-token decoding or an
operator's Redis inspection is not an acceptable client identity mechanism.

## Native refresh transactions

Before a rotating exchange is dispatched, native MCP persists an expired,
tokenless record in its already-selected store. Endpoint, issuer and client
binding remain present. The old refresh grant exists only in the live transaction.
This uses the existing serialization format, so rollback cannot load a consumed
predecessor from that record. Do not restore credential backups on rollback.

On success, save the returned generation before exposing it. Store-write retries
reuse exactly that returned generation. A timeout, response loss or process death
leaves no reusable predecessor. Pre-dispatch metadata/store failures preserve the
unconsumed grant. Caller cancellation cannot abandon the proactive transaction's
persistence task. The same retirement hook applies to RMCP-coordinated reactive
refresh. AIRS selects coordinated mode and acquires a per-user binding lock before
the legacy home lock, so different environment homes sharing a native credential
serialize their exchanges.

## Remaining release gates

The candidate adds typed inference failures, bounded helper completion, verified
same-identity `login --restore-session`, and an explicit `/signin` terminal action.
The draft and conversation remain in place; completed work is not replayed.
MCP authentication failures stop the turn before model or code-mode fallback.
Gateway MCP sign-in requires a fresh conversation until the gateway exposes a
trusted identity continuity contract. Backend service-account renewal is separately
deployed on both production MCP replicas (50 backend tests passed).

Validate both gateway OAuth legs, idle return,
cancellation, persistence failures and logout races. Run exact Linux and signed
Apple Silicon package acceptance and registry upgrades, then publish the measured
behavior in the canonical education lessons and Docusaurus site. Keep alpha.14
receipts and their limited release exception unchanged.
