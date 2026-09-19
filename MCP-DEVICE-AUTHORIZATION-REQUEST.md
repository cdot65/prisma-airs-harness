# Native device authorization for gateway MCP clients

Status: proposed gateway enhancement; current deployment does not support this flow.
Prepared September 19, 2026. This request has not been submitted to the vendor.

## Problem and requested outcome

A Prisma AIRS harness running over SSH can complete gateway MCP OAuth today by
having the user open the authorization URL on a laptop and paste the resulting
localhost callback into the harness's hidden field. The browser cannot deliver
that callback directly to the remote host. This manual workflow has been confirmed
working, but is easy to misunderstand.

Request native RFC 8628 device authorization for the AI Gateway MCP OAuth layer.
The remote harness should display a verification URL and user code, then poll for
approval. The browser should complete company SSO and MCP consent on a reachable
gateway verification page. No callback copying or inbound connection to the
harness should be needed.

Keep the existing CAS authentication, CIE directory/group-to-workspace policy and
gateway-managed upstream OAuth. No separately operated callback relay is wanted.

## Observed deployment and reproducible failure

The hybrid deployment uses gateway enterprise 2.22.0. On September 19:

- MCP protected-resource metadata identified the gateway as its authorization server.
- Resource-specific authorization metadata advertised `authorization_code` and
  `refresh_token`, without `device_authorization_endpoint`.
- A token request with a deliberately invalid device code and an unregistered
  diagnostic client returned HTTP 400:

```json
{"error":"unsupported_grant_type","error_description":"Invalid grant_type parameter"}
```

This demonstrates rejection of that grant by the deployed endpoint. It does not
establish whether another release or an unpublished feature flag supports it.
Read-only inspection also showed that the data plane forwards token requests to
its control plane. A data-plane discovery change alone would therefore be insufficient.

The following reproduction uses placeholders and no real credential:

```sh
curl --silent --show-error --include \
  --request POST 'https://mcp.example.com/oauth/token' \
  --header 'Content-Type: application/x-www-form-urlencoded' \
  --data-urlencode 'grant_type=urn:ietf:params:oauth:grant-type:device_code' \
  --data-urlencode 'client_id=airs-device-feasibility-unregistered' \
  --data-urlencode 'device_code=deliberately-invalid-probe'
```

After adding support, an invalid-client or invalid-grant result still would not
prove successful device authorization. Acceptance requires a registered public
client and a real approved request.

## Ownership and boundaries

| Connection | Required behavior |
| --- | --- |
| Harness to gateway authorization server | New device grant; gateway-issued MCP credential |
| Browser to CAS and organizational IdP | Existing company authentication, MFA and identity selection |
| CIE-backed workspace policy | Existing membership, group mapping and resource authorization |
| Gateway to upstream MCP server | Existing separately managed upstream authorization and tokens |
| MCP server to ServiceNow API | Existing integration credential and ServiceNow permissions |

Device authorization is needed at the harness-facing authorization server. Neither
the upstream MCP server nor the ServiceNow machine identity needs to adopt the same
grant merely because the harness uses it. Device flow must not substitute an
inference JWT, skip CAS, grant missing workspace membership or bypass the gateway.

## Required gateway contract

1. **Discovery and client registration.** Advertise the device endpoint and
   `urn:ietf:params:oauth:grant-type:device_code` through the authorization-server
   metadata discovered for the MCP resource. Document the supported public-client
   registration method, grant allowlist, scopes and resource/audience binding.
   No confidential client secret may be embedded in the CLI.
2. **Initiation.** Accept a registered public client's device request and return
   `device_code`, `user_code`, `verification_uri`, `expires_in`, and optionally
   `verification_uri_complete` and `interval`, with RFC 8628 semantics. Serve the
   endpoints over HTTPS and prevent sensitive responses from being cached.
3. **Browser approval.** Preserve the pending grant through CAS federation and
   MFA. Show the client and requested MCP access, require deliberate approval or
   denial, resolve the intended identity using existing CIE policy, and enforce
   workspace/resource permissions. Prefilled codes must not silently approve access.
4. **Polling and completion.** Support the device grant at the token endpoint,
   including `authorization_pending`, `slow_down`, `access_denied`, and
   `expired_token`. Bind approval to the initiating client and requested resource;
   make redemption atomic and single-use. Return gateway-facing access credentials
   and, where policy allows, refresh credentials compatible with existing MCP use.
5. **Distributed state and lifecycle.** Pending and approved grants must work
   across replicas, expire reliably, and enforce polling/code-entry rate limits.
   Define denial/cancellation behavior and prevent replay or cross-workspace
   redemption. Preserve existing refresh, revocation and deprovisioning policy;
   do not extend session lifetimes simply to enable this flow.
6. **Upstream integration.** When upstream consent is required, provide a supported
   browser continuation through the gateway. Define whether upstream consent must
   complete before issuing the gateway credential or is a subsequent step. The
   harness must not receive the upstream access/refresh token.
7. **Operational support.** Identify the required control-plane and data-plane
   versions, tenant/client configuration, rollout and rollback procedures, and
   safe correlation fields. Audit records must exclude tokens, device secrets and
   callback codes while retaining identity, client, resource, decision and outcome.

## Questions for the gateway team

- Is device authorization already supported in another release or by a supported
  tenant/client setting? If so, provide its discovery, registration and configuration contract.
- Which component owns device initiation, verification state and token issuance
  in the hybrid deployment? Does the control plane require an enhancement?
- Does the same CAS/CIE integration support the proposed browser continuation,
  including MFA, denied workspace access and gateway-managed upstream consent?
- What MCP resource/audience parameters and refresh-token behavior are supported?
- Can a development tenant/client be enabled before any production policy changes?

## Acceptance required before a harness test handoff

| Check | Passing result |
| --- | --- |
| Real SSH sign-in | Browser on another machine approves; CLI completes by polling, without callback copying or forwarding |
| Local desktop sign-in | Explicit device login also works locally; normal browser login remains available |
| Policy | Authorized member succeeds; valid SSO user without workspace access is denied |
| Protocol errors | Pending waits, slowdown increases the interval, denial/expiry stop, cancellation stops client polling |
| Binding and replay | Wrong client/resource, reused code and expired grant are rejected |
| Storage | Native credential persists; a fresh harness process can list tools and perform one approved read |
| Lifecycle | Refresh and revocation work under actual configured policy; storage failures do not produce a false success |
| Integration | CAS, CIE and upstream ServiceNow authorization remain effective; tool discovery alone is insufficient |
| Regression | Existing browser/manual-callback login and workspace-key inference remain usable |

Fixture tests can establish client behavior, but cannot substitute for the actual
gateway/CAS/CIE checks. A test package should be presented as ready for local
acceptance only after the gateway contract is available and integrated.

## References

- [OAuth device authorization, RFC 8628](https://www.rfc-editor.org/rfc/rfc8628.html)
- [Gateway authentication layers](https://portkey.ai/docs/product/mcp-gateway/authentication)
- [Gateway OAuth](https://portkey.ai/docs/product/mcp-gateway/authentication/oauth)
- [MCP authorization](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization)
