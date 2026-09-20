---
id: glossary
title: "Glossary and misconceptions"
sidebar_label: "Glossary and misconceptions"
---

## Vocabulary used in this course

Most of these terms have broader meanings elsewhere. The table gives the meaning each one carries in this course, which is usually narrower and tied to a specific component or boundary in the running example.

| Term | Meaning here |
| --- | --- |
| Harness | Native terminal application that owns the conversation and dispatches tools |
| Native MCP client | Built-in harness component that discovers and calls gateway-proxied MCP tools |
| mcp server 1 | Separately deployed server exposing eight locally executed utilities |
| Local utility execution | Computation on the MCP server host, without a downstream tool API |
| AI Gateway | Destination for both inference and remote MCP requests |
| Inference | Model request and response, including proposed tool calls |
| MCP tool | Named operation with input/output schemas |
| MCP resource | Separately advertised data capability; this server currently exposes tools only |
| Issuer and subject | Pair identifying a user within an identity authority |
| Audience | Intended receiver of an access token |
| Scope | Permission issued in an OAuth grant |
| Resource role | Grant belonging to the configured resource client |
| Subject binding | Server policy permitting utilities.use for an explicit subject |
| JWT | Signed token representation; it must be verified, not merely decoded |
| Opaque token | Credential interpreted by its issuer/resource, without client-readable JWT claims |
| JWKS | Trusted issuer public keys used for signature verification |
| Access token | Credential presented to a resource |
| Refresh token | Credential exchanged at its authorization server for a new generation |
| PKCE | Binding between the initiating native client's verifier and authorization-code exchange |
| OIDC | Identity layer used in native inference sign-in |
| SSO | Shared browser authentication experience; it does not imply one universal token |
| CIE Directory Sync | Directory identity and group context |
| CAS | Cloud Authentication Service supporting gateway-facing federated login |
| SCIM | User/group provisioning protocol |
| SAML | Federation protocol carrying authentication assertions |
| Workspace | Gateway context for permitted integrations and configuration |
| Guardrail | Configured policy check on the applicable gateway path |
| GitOps | Deployment reconciliation from declared source |
| Native credential store | macOS Keychain or Linux Secret Service for this distribution |

## Keep these distinctions clear

The misunderstandings below tend to come from collapsing two things that happen to occur together: a login and a grant, a tool result and a policy check, a word like "local" and the place it actually refers to. Each paragraph names the pair and the fact that separates them.

**"Local" names the server, not the workstation.** Utility computation runs on mcp server 1. That does not keep the input on Alex's machine: the arguments still travel from the workstation through the gateway to the server, and the result may come back and enter a later inference request. Local describes where the work happens, not where the data stays.

**The client is built in; the server is not.** The harness contains the MCP client, so no secondary local MCP executable is required to reach remote tools. mcp server 1 is a separate deployment with its own release boundary, which is why updating one does not update the other.

**Three parties handle one model-proposed function, and none of them does the others' job.** The model selects the function. The harness dispatches the MCP request, the gateway proxies it, and mcp server 1 authorizes and computes it. A returned function call is a proposal from the model; execution evidence comes from the completed tool result.

**An empty resources list says nothing about tools.** Tool discovery and resource discovery are separate MCP capabilities, so the resources list reports only the resource inventory. This server currently exposes tools only, and an empty resources list is the expected result rather than a sign that something is missing.

**A simple utility still needs the full grant.** The operation being a multiplication does not change what the server checks. It still enforces the upstream token, the invoke role, the utilities.use scope and role, and the subject binding, because the cost of the operation is not the thing being authorized; the caller's access to this resource is.

**Read-only MCP does not make the whole agent read-only.** The read-only annotations describe this server's eight tools. The harness's local shell and file tools have their own permissions and are unaffected by them. One further caution: the time and UUID tools return different values on repeated calls even though they change nothing, so non-destructive is not the same as deterministic.

**CAS supports one login; it does not supply every credential.** CAS handles gateway-facing federated login. The gateway obtains upstream OAuth credentials for mcp server 1 separately and keeps them, and the harness holds its own inference credential on top of that. One browser sign-in can sit behind all three without any of the resulting tokens being interchangeable.

**"No downstream API" is a statement about the tools, not the host.** The utility tools need no downstream API to do their work. The server's authentication layer may still retrieve the issuer's public keys to verify signatures, and the deployment has the ordinary network and observability dependencies of any service. When a DNS or key-retrieval failure appears, it belongs to the identity boundary, not to the calculation.

**A tool result does not prove content scanning.** `calculate` returning 84 establishes that the calculation ran and returned 84. Whether a scanner evaluated the inference that led to it is a different observation that needs its own correlated evidence, because the utility never sees the gateway's policy path.

**Logout is not immediate global invalidation.** Deleting a local credential, revoking a session at the issuer, waiting for a token to expire, removing workspace membership and changing server policy are five different actions with five different effects and propagation times. Which one actually removes access depends on which receiver is enforcing it, so the question to ask is always where the access is enforced and what that component consults.

Return to [Start here](./start-here.md) or try the [labs](./labs.md).
