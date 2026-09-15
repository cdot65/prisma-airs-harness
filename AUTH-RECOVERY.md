# Authentication recovery implementation

Status: alpha.16 sign-in prompt correction; exact package publication evidence is
recorded separately.
This document records source findings and validation boundaries; it is not a
claim that guided reauthentication or full gateway lifecycle acceptance shipped.

## Ownership

Both inference and native MCP target Prisma AIRS AI Gateway. The harness owns
its inference OIDC credential and gateway-facing MCP credential. Gateway/CAS
owns upstream MCP OAuth. mcp server 1 executes local utility tools and has no management API
service-account dependency. Device authorization changes initial login, not renewal.

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
MCP authentication failures stop the turn before another model request. Code mode
and its separate host remain disabled by the existing AIRS boundary.
Gateway MCP sign-in requires a fresh conversation until the gateway exposes a
trusted identity continuity contract. The upstream server now exposes local utilities; SCM backend renewal is outside
the current harness architecture.

Validate both gateway OAuth legs, idle return,
cancellation, persistence failures and logout races. Run exact Linux and signed
Apple Silicon package acceptance and registry upgrades, then publish the measured
behavior in the canonical education lessons and Docusaurus site. Keep alpha.14
receipts and their limited release exception unchanged.

## Candidate validation — September 14, 2026

The combined authentication and terminal suites passed 5,833 tests (14 skipped).
Four focused core MCP tests passed, including the gateway case that records the
failed tool result, stops before another model request and displays recovery
without replay. The terminal regression covers the actual `Fatal error:` wrapper
on that recovery message. Packaging contracts, launcher checks and bundle tests
also passed.

The broader core run was not green: 4,055 passed, 104 failed and nine skipped.
One failure was the gateway fixture's old two-request expectation, corrected and
verified by the focused run above. Other failures are concentrated in Code Mode
and its missing host; they are not claimed as a clean baseline comparison.
The required full workspace attempt failed during compilation because Rusty V8
150.4.0 does not publish the requested Linux musl archive. AIRS's existing disabled
Code Mode boundary was preserved; no feature or test gate was weakened.

The first Apple Silicon candidate (`2544697ad202`) passed native build and private
acceptance, but predates the terminal correction and must not be promoted. Final
source-bound packages, Developer ID signing, installed gateway lifecycle tests
and publication remain pending. No alpha.15 production consent or expiry cycle
is represented by these offline and native-fixture checks.

## Inference dispatch correction — September 15, 2026

After about 30 minutes idle, alpha.15 showed a generic fatal helper error. Manual
`/signin` restored the same verified identity and the next inference reply in the
same conversation. The normal AIRS provider dispatch bypassed the typed recovery
classifier. Alpha.16 routes gateway command credentials through that classifier
so the terminal can present its sign-in prompt. Other custom-provider paths are
unchanged. A real helper integration test failed before the fix; all 78 provider
tests and scoped Clippy pass afterward. This does not establish full active token
renewal or post-login MCP acceptance.
