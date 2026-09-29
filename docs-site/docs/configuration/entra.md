---
title: Microsoft Entra ID through Keycloak
---

Use Entra as an upstream OIDC identity provider for Keycloak. The first half of
this page explains how the login works, so the setup steps make sense. The second
half is the procedure: register the application, add the provider, map grants and
verify the result.

## How the login works

From the harness's point of view nothing changes. It still uses the Keycloak
issuer, native client, audience and gateway claim contract from
[Keycloak setup](keycloak.md). Entra sits behind Keycloak as the place the person
actually signs in, and the gateway only ever sees Keycloak-issued tokens. That is
why Entra's application secret stays on the server and never belongs in the
harness: the harness never talks to Entra at all.

```mermaid
sequenceDiagram
  actor User
  participant Harness
  participant KC as Keycloak broker
  participant Entra as Microsoft Entra ID
  participant Gateway as AI Gateway
  Harness->>KC: Native authorization request + PKCE
  User->>KC: Choose Microsoft Entra ID
  KC->>Entra: Tenant OIDC authorization + PKCE
  User->>Entra: Sign in / MFA / Conditional Access
  Entra-->>KC: Code to broker web redirect
  KC->>Entra: Server-side code exchange
  Entra-->>KC: Identity and assigned roles
  KC->>KC: Map identity and permitted groups
  KC-->>Harness: Native code, exchanged for Keycloak tokens
  Harness->>Gateway: Keycloak-issued access token
  Gateway->>Gateway: Validate Keycloak issuer and gateway grants
  Gateway-->>Harness: Inference result
```

There are two separate OIDC exchanges in that flow. The harness is a public
native client and talks only to Keycloak. Keycloak is a confidential web client
of Entra and completes its own code exchange from the server side. Those are two
different applications with two different trust relationships, which is why the
setup below registers a new Entra application instead of reusing the harness
client.

Access is decided in Keycloak, not in Entra. Entra supplies an identity and its
assigned application roles. Keycloak maps them to groups, and a group's
resource-client role reaches the inference access token through the mapper
configured in [Keycloak setup](keycloak.md). The gateway validates that token
and never sees the Entra assignment itself. Tool access follows a separate
grant: an inference group alone does not give a person MCP access.

Because grants are mapped at broker login, a change in Entra is not instant.
With forced synchronization, role and group mappings are reevaluated when the
person next logs in through the broker. Removing an Entra assignment does not
invalidate an existing Keycloak session or an issued token. Access ends at the
next broker login, or sooner only if you revoke the session or token yourself.

## Set it up

### 1. Register the broker application

In the intended Entra tenant, register a single-tenant application such as
`Example Corp Keycloak`. Add a **Web** redirect URI, exactly:

```text
https://sso.example.com/realms/example-corp/broker/entra/endpoint
```

This confidential web application is separate from Keycloak's public native
`ai-gateway-agent` client. Record the tenant ID and application client ID. Create
a time-limited client secret and place it in the server-side secret store used
by your Keycloak deployment. Record the expiration and rotation owner. Do not
save it in a realm export, Git, terminal history or a public documentation example.

Set **Assignment required** on the enterprise application. Assign an approved
test user or group. Where the tenant requires it, grant administrator consent
for the delegated OIDC permissions before testing. The reference deployment
requests `openid profile email` and optional ID-token claims `email`,
`given_name`, and `family_name`.

Create application roles if access is managed in Entra, for example
`Inference.Invoke` and `Tools.Read`, and assign them to the appropriate groups.
These are examples; the Keycloak mapping in step 3 must use the exact role values
you configured.

### 2. Add the identity provider to Keycloak

Under realm **Identity providers**, add OpenID Connect v1.0 with alias `entra`.
Use the tenant-specific discovery URL:

```text
https://login.microsoftonline.com/YOUR_TENANT_ID/v2.0/.well-known/openid-configuration
```

Use the Entra application client ID and server-held secret. The deployed pattern
uses client authentication `client_secret_post`, PKCE `S256`, default scopes
`openid profile email`, sync mode `FORCE`, and **Trust email off**. Retain the
standard first-broker-login flow with account verification. Do not automatically
link an existing account just because the email addresses match. A matching
address does not prove the same person controls both accounts.

Permit Keycloak egress to `login.microsoftonline.com:443` and keep external DNS,
TLS and time synchronization working. Check discovery's authorization, token,
and JWKS endpoints against the selected tenant. A browser reaching Entra does
not prove that Keycloak can exchange the code from inside the cluster.

Leave Entra as an explicit login choice until tested. Local Keycloak users can
remain available according to your recovery policy; changing the default login
provider is a separate operational decision.

### 3. Map identity and grants

Use a username template such as `${CLAIM.preferred_username | lowercase}` and
map the broker's email/name claims according to your organization's policy.
Retain Entra `oid` and `tid` as server-side attributes when needed for stable
identity correlation. Do not use a display name as an authorization identifier.

Map Entra application role `Inference.Invoke` to Keycloak group
`/stacks/ai-inference/production/users`. Map `Tools.Read` separately to the group
your MCP/CAS authorization policy expects.

## Verify it

Run the normal harness login and choose Entra in the browser. Confirm the final
issuer is Keycloak, the audience is `stack-ai-inference`, the client is
`ai-gateway-agent`, the workspace is correct and the `invoke` grant is present.
Inspect claims only in a private local tool; do not paste a live token into an
online decoder. Run `airs --environment work doctor --verify-access`, then the
separate MCP read-only check in [acceptance](../validation/acceptance.md).

Test removal as well as access. Remove the Entra assignment, log in again through
the broker, and confirm the grant is gone. Follow your session and token
revocation procedure when access must end immediately.

The reference deployment has recorded credential-free redirect validation to
the Entra sign-in page. That evidence does **not** establish a completed human
Entra login, post-broker token claims, or a live MCP operation. Record those
outcomes for your own assigned account before calling the federation accepted.

## Troubleshoot

| Symptom | Check |
| --- | --- |
| Redirect URI mismatch | Web redirect path, realm and identity-provider alias |
| User not assigned | Enterprise application assignment and app role |
| Consent required | Tenant administrator consent policy |
| Browser login succeeds, broker fails | Keycloak egress, client secret and expiration, tenant endpoints |
| Login succeeds, inference denied | Keycloak group mapping, role scope and gateway claim contract |
| Old access persists after role removal | Existing Keycloak sessions, token lifetime and revocation |

Microsoft documents the [Web redirect registration](https://learn.microsoft.com/en-us/entra/identity-platform/how-to-add-redirect-uri) and [OIDC protocol](https://learn.microsoft.com/en-us/entra/identity-platform/v2-protocols-oidc) used by the broker.
