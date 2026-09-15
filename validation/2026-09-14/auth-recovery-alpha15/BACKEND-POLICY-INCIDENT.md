# Intermittent SCM authorization denials block alpha.15 acceptance

Prepared September 15, 2026, for investigation. This report has not been sent to
Palo Alto support. All times below are UTC.

The Prisma AIRS management API intermittently rejects authorized workspace,
configuration and guardrail reads with HTTP 403. A captured response includes
`x-opa-decision: false`. The service token is fresh, the configured tenant and
workspace scope match the intended resource, and other calls with the same
service-account configuration succeed. The cause remains unresolved.

Both exact native alpha.15 candidates completed gateway/CAS login, native token
storage, all eight production MCP tools and two scheduled activity intervals.
Their third interval failed on backend workspace reads while the gateway-facing
token remained unexpired. Neither reached its first frontend expiry. Coordinated
cleanup completed on both platforms. Alpha.15 is unpublished.

| Context | Value |
| --- | --- |
| Tenant/TSG | `1001464285` |
| Service principal | `pmcp_prod_gw_ro@1001464285.iam.panserviceaccount.com` |
| Workspace UUID | `ce06de57-3ccb-4a98-a653-bcc2b1de76ca` |
| Workspace slug | `ws-prisma-7b25ac` |
| IAM scope | `ws_prisma_airs_harness_lcc9nz` |
| Scope resource | `workspace` / `ws-prisma-7b25ac` |
| Access-policy resource | `prn:1001464285::::ws_prisma_airs_harness_lcc9nz` |
| Custom role | `pmcp_prod_gw_ro:1001464285` |
| API base | `https://api.apps.paloaltonetworks.com/ai_gw/v2` |

IAM readback found one expected service-account policy. The role contains only
`airs_gw.workspaces.list`, `airs_gw.workspaces.read`, `airs_gw.configs.list`,
`airs_gw.configs.read`, `airs_gw.guardrails.list`, and `airs_gw.guardrails.read`.
No permission or scope change was made during this investigation.

| Time | Request | Vendor request ID | Observed result |
| --- | --- | --- | --- |
| 00:12:51 | `GET /workspaces/ce06de57-3ccb-4a98-a653-bcc2b1de76ca` | `7d8820c4-7628-4c54-8821-9b76e4591222` | HTTP 403; cached service token had 896 seconds remaining |
| 00:12:55 | Same workspace read | `e82c20bc-7e52-46d5-acb7-51f4ec1222bb` | HTTP 403; 892 seconds remaining |
| 00:13:44 | Same workspace read | `04695db8-17c0-444b-951c-df0a634ae6b2` | HTTP 403; 843 seconds remaining |
| Subsequent bounded diagnostic | `GET /configs/b9075d69-b6a5-442d-a279-32a1ba801a22` | `41829134-1c9b-478c-9639-6bbfda45f716` | HTTP 403; `x-opa-decision: false`; 892 seconds remaining |
| 00:31:47 | `GET /configs?workspace_id=ce06de57-3ccb-4a98-a653-bcc2b1de76ca` | `81708d9c-365e-4630-bd21-291c9eb47be7` | HTTP 403; `x-opa-decision: false`; correct `x-tsg-id` present; 894 seconds remaining |

The final denial occurred on the corrected production image
`sha256:b57dbdebe53c9fa36b69fea8559a53ee92198a48dfc3595f84ff54e5125df77f`,
built from `ae68751bec4d72366c68c7a392143cc602aefe21`. Its 59 application tests,
type checking, build and image scan passed. Development verification passed after
an earlier separate HTTP 500; the failed check is preserved. Both production
replicas are healthy under manifest revision
`d6ae2c67f869b24b4063e3d1a1463bd1bab6004a`.

The adapter now supplies its configured `x-tsg-id`, matching the SDK management
client. Paired diagnostics passed 116 requests with the header and 116 without
it. The explicit denial recurred after deployment with the header present, so
that omission does not explain or resolve the incident.

A diagnostic retained six independently issued service tokens and interleaved
21 successful reads across them. That does not support a rule that minting a new
service token immediately invalidates its predecessor. A prior real-lifetime
diagnostic produced 12 successful pre-expiry reads and four HTTP 401 responses
at/after expiry. These observations do not establish why the policy denies the
listed requests.

The policy denial differs from the application response for a synthetic absent
workspace: the latter returned `403 AB03` without `x-opa-decision`. This is a
response-shape observation, not proof of which internal SCM component failed.

To reproduce within the existing read scope, obtain a service token for the
listed tenant, retain it in memory, and issue workspace/configuration/guardrail
reads with `Authorization: Bearer <token>` and `x-tsg-id: 1001464285`. Repeat across
normal token renewal and concurrent callers, recording status, request UUID,
policy-decision header and remaining token lifetime. The failure is intermittent;
this procedure does not reproduce it on every invocation. Never include actual
tokens, client secrets or raw authorization headers in an incident attachment.

The missing evidence is SCM's authorization trace for the request IDs above:
the evaluated principal, action, tenant, resource/scope, matching policy and role,
and policy/cache revision. Compare denied requests with nearby successful reads
and determine why the configured grant was not accepted. A human gateway login
does not repair this backend service-account decision.

The harness still targets AI Gateway for inference and MCP. The adapter retries
one authorized GET only after HTTP 401 rejects a cached token. HTTP 403 remains
denied; no broader role, admin-plane fallback or automatic denial retry was added.
`COORDINATED-ACCEPTANCE-FAILURE.json` binds the retained receipts. The remaining
backend soak was stopped once its peer reproduced the denial; it is not marked
passed. Exact native frontend lifecycle acceptance, package publication and
owner upgrades remain pending.
