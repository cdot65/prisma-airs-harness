# Keycloak rollout gate

Update: the isolated [Keycloak identity foundation](identity/README.md) now proves
mandatory workspace JWT/scanner policies and public-client role enforcement.
The original shared-workspace failures below remain historical evidence. Strict
routing binding and refresh-token replay protection still block teammate rollout.


Live tests on September 7 found **three failed gateway authorization gates**.
Do not distribute a terminal public-client login against this configuration yet.
Workspace-key mode remains the supported pilot path. These findings concern
AIRS gateway 2.20.0 with `JWT_ENABLED=ON`, not Keycloak's ability to issue tokens.

| Probe | Expected | Actual |
| --- | --- | --- |
| Valid JWT, bound config, default model omitted | 200 with both scans | Passed |
| JWT for an unrelated audience | Denied | **200** |
| Inline config override despite `allow_config_override=false` | Denied | **200, no scanner hooks** |
| No default config, explicit model | Denied | **200, no scanner hooks** |
| Invalid config reference | Denied | 400 |
| MCP-only scope used for inference | Denied | 403 |

Receipt: `validation/2026-09-07/identity/prerequisites.json`. The probe used a
new temporary confidential service client solely to obtain controlled, signed
60-second test tokens. It did not use password grants or change any existing
client, user, scope, organization authentication settings, or production policy.
The client was deleted and readback confirmed removal. No tokens or client
secrets are in the receipt. These are server authorization tests, not evidence of
public-client browser login, device login, refresh, logout, or individual-user
acceptance.

The deployed gateway verifier validates signatures/expiry but does not pass issuer
or audience constraints to its JWT verification library. Its local JWT defaults
mapping does not copy `allow_config_override` into the API-key defaults checked
by request-header processing. The live failures substantiate the audience and
config-binding gaps; issuer rejection and JWKS rotation remain unverified live.
`defaults.config_slug` was included alongside `config_id` in the successful
bound-default fixture; do not assume `config_id` alone configures local JWT mode.

Keycloak discovery for `https://auth.dev.cdot.io/realms/truffles` advertises S256
PKCE and the device authorization endpoint. Existing AIRS clients are confidential
service clients. The terminal design still requires a dedicated public client,
loopback code flow with PKCE/state/nonce, supported device flow, secure refresh,
and separate resource-bound MCP authorization. No reusable service secret belongs
in the terminal.

Before implementing that rollout, require gateway-side signature, issuer,
audience, expiry, workspace, scope and mandatory config/policy enforcement that
cannot be bypassed with headers or missing claims. A config-attached guardrail
alone cannot repair a bypass that lets the caller replace that config. Close the
raw-request failures first using a supported gateway enforcement mechanism, then
rerun this matrix and the existing workspace-key/MCP regression suite. Preserve
other AIRS clients when changing deployment-wide settings. A new authentication
proxy would be an architecture change and is not silently substituted here.

The current vendor documentation describes JWT default configurations as
request-overridable: [Portkey JWT documentation](https://portkey.ai/docs/product/enterprise-offering/org-management/jwt).
Keycloak's supported flows are documented in its
[OIDC endpoint reference](https://www.keycloak.org/securing-apps/oidc-layers).
The deployed AIRS behavior and redacted live receipts govern acceptance.
