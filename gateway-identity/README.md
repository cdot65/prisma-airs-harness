# Terminal request policy at the existing AIRS gateway

This sidecar closes the deployed gateway's JWT config-override gap while keeping
the existing Keycloak realm/JWKS and original user JWT. AIRS remains the
cryptographic authenticator, provider router and mandatory guardrail executor.
There is no shared inference credential or second issuer in this component.

The filter identifies Terminal requests from the JWT client/workspace claims.
This decoding is classification, **not authentication**: it only adds restrictions.
A token altered to evade classification fails native AIRS signature validation;
a valid token from another client cannot satisfy the Terminal workspace's
mandatory authorized-party/audience/role checks.

Terminal inference allows POST `/v1/responses` and `/v1/chat/completions`. It
rejects routing/provider/config-version/custom-host/forward-header/metadata/
guardrail/nitro headers, provider credential headers, routing fields in the JSON
body, duplicate credentials, unsupported paths and malformed model selection.
A missing `model` remains missing; an explicit model must have `@provider/model`
syntax and is authorized/routed by AIRS. Allowed bodies are forwarded byte for
byte. The bearer credential helper is normalized into the native guardrail's
`x-portkey-api-key` header without replacing the token.

The two listeners (8887 inference, 8888 MCP) forward only to fixed loopback
backends (8787/8788). Existing non-Terminal credentials retain the native gateway
path, including MCP streams and websocket upgrades. Terminal MCP is denied until
its separate resource authorization is configured. Request bodies are bounded to
16 MiB, with eight concurrent Terminal requests per replica. Tokens, prompts and
responses are never logged by this filter.

Infrastructure lives in `cdot65/talos-cluster/airs-terminal-identity/gateway/`.
Every gateway Service targets the filtered ports; a NetworkPolicy denies access
to the raw pod ports. Direct raw-port access via cluster administrator mechanisms
such as `kubectl port-forward` is outside the teammate threat boundary. Ordinary
workload network traffic must not reach those ports. The rollout tested both
replicas from another pod and retained the public Service/LoadBalancer ports.

Run `node --test gateway-identity/server.test.mjs`. Tests use a recording backend
to verify pre-forward denials, exact body/token preservation, bounded bodies,
SSE delivery/cancellation, duplicate credentials and shared-stack websockets.
Live artifacts are in `validation/2026-09-07/auth-release/`. The Keycloak
refresh-token gate is independent of this filter and must pass before release.
