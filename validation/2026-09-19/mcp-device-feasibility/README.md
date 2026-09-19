# MCP device authorization feasibility — September 19, 2026

Result: native device authorization is blocked by the deployed gateway. No runtime
change, new package, or successful device-MCP login is claimed.

`gateway-capability.json` records public discovery and one deliberately invalid
RFC 8628 token request. The endpoint returned HTTP 400 `unsupported_grant_type`.
The request contained no real user credential and created no registered client.
`deployed-gateway.json` records the live enterprise image digest and read-only
packaged-code observations. No proprietary bundle or secrets are retained.

The inspected data plane forwards OAuth token requests to the control plane.
Literal marker absence is supporting evidence only; it does not prove that no
other release or vendor-controlled configuration could support the capability.

The owner selected native device authorization and explicitly declined a new
callback relay service. See the repository-root
[MCP enhancement request](../../../MCP-DEVICE-AUTHORIZATION-REQUEST.md) for the
contract and acceptance gates. The request is a draft, not a submitted ticket.
Existing mcp.5 browser/manual-callback login remains the working path.

Only read-only Kubernetes inspection and unauthenticated diagnostic HTTP requests
were performed. No gateway, CAS, CIE, Keycloak, Talos or owner-profile configuration
was changed. Product runtime and package release tests are inapplicable to this
documentation/evidence-only change; no Rust suite result is claimed.
