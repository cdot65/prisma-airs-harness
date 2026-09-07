# Identity boundary proposal — decision needed

Native AIRS workspace guardrails now enforce signed Keycloak roles and mandatory
scanning. Two live constraints prevent the intended teammate rollout: JWT routing
config overrides are still accepted, and the shared Keycloak realm accepts
refresh-token reuse. This proposal changes the authentication architecture; it
has not been deployed or selected as the client's implementation target.

## Recommended option

Use a dedicated Terminal Keycloak realm with refresh-token rotation and zero
reuse. Broker authentication to the existing `truffles` realm so teammates keep
their existing login/MFA. Grant Terminal access explicitly through mapped roles;
never make all brokered users members by default. The new realm isolates session
and rotation policy from existing applications. Keep browser S256/state/nonce,
short-lived access JWTs, device login and no offline refresh grants.

Add a small authentication boundary at a dedicated Terminal API hostname. It
validates the new issuer's signature/JWKS, exact audience, authorized client,
subject, expiry and operation roles before any upstream call. It accepts only
explicit inference/MCP paths and methods. It rejects caller config/provider/
custom-host/guardrail overrides and identity metadata; derives audit identity
from verified issuer plus subject; and uses a narrow server-side workspace
credential to call AIRS under a fixed config/policy. This is **credential
translation**, not forwarding the user's JWT unchanged to AIRS.

AIRS continues to own provider credentials, model routing, mandatory scanning and
model authorization. The boundary preserves Responses SSE and cancellation,
omits `model` when the client omits it, and forwards only authorized explicit
`@provider/model` values. It does not choose a default model. Local filesystem,
shell, skills, approvals and sessions remain entirely in the terminal.

The gateway credential must never be distributed to users or included in logs.
Its fixed config must reject overrides and preserve mandatory workspace policies.
The existing shared gateway must reject the new realm's user JWTs directly, so
changing a local base URL cannot bypass this boundary. If deployments share the
same organization JWKS, do not add the new realm's keys there; validate them only
at the boundary. Network restrictions protect any dedicated backend deployment.

MCP gets a separate audience, operation role and upstream `mcp.invoke` credential.
Inference tokens cannot be reused for MCP. Forward verified trace/identity fields
under server control and prove that AIRS telemetry preserves both the user
subject and the credential translation boundary; a workspace credential alone
must never be described as individual identity.

Service secrets can begin in owner-protected Kubernetes Secrets. Conjur/ESO can
manage those server secrets after the administrator access prerequisite is met;
it is optional to the first identity rollout. Store user refresh tokens in the
local OS keyring with locked, atomic rotation and no silent plaintext fallback.

## Alternative

Stay with the native AIRS JWT path and wait for strict config/header binding plus
a suitable refresh policy. This keeps fewer runtime components and preserves the
user JWT through AIRS. It requires platform changes or an agreed relaxation of
the strict config-binding contract. Enabling refresh rotation globally in
`truffles` needs compatibility testing across its other applications.

## Acceptance before release

- Browser/device login with two users; explicit role grant and denial; callback
  state/issuer/nonce and PKCE failures; no password collection by the terminal.
- Refresh rotation, replay denial, concurrent refresh serialization, logout,
  expired tokens and unavailable IdP/keyring recovery without identity switching.
- Raw inference and MCP requests cannot bypass audience, role, config, provider,
  model or scanning checks via headers, alternate URLs or direct backend access.
- Default and explicit model requests preserve the agreed wire contract through
  multi-turn SSE, local tool execution, cancellation, compaction and resume.
- Trace correlation to verified user subjects, plus cross-user/environment
  session isolation. Tokens and credentials absent from logs and artifacts.
- Existing workspace-key pilot and other applications still pass their live
  integration tests. Linux package passes clean installation and owner review.

The installed release remains unchanged until these gates pass. Neither the
native pilot nor this proposal is a completed M3 release.
