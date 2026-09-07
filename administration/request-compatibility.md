# Model-specific request compatibility at AIRS

The owner proposed gateway guardrails to strip unsupported model parameters during
alpha.5 repair on September 7. This is a separate concern from scanner security
policy and the terminal's capability-aware picker.

## Verified deployed mechanisms

Inspected AIRS gateway 2.20.0's deployed `/app/build/start-server.js`, SHA-256
`b2710c4f6a580a5a1bb525695f942175e4698d61a0c90ba066c48f93490f6331`.
The configured workspace also contains a preexisting `max_completion_tokens`
guardrail using `default.regexReplace`; it was read but not changed or attached
to the terminal routing configuration.

| Mechanism | Verified behavior |
| --- | --- |
| Target `drop_params` | Removes named JSON keys or dotted paths after default/override parameter merging and before guardrail context creation. |
| `default.regexReplace` | Extracts message/input/output text and replaces matching text; it does not rename or delete root request parameter keys. |
| `default.requestParametersCheck` | Checks root parameter keys/values and tool allow/block lists; returns a verdict, not a transformed body. |
| `default.webhook` | A before-request hook can return `transformedData.request.json`; the gateway accepts that as transformed request JSON. |

The installed gateway supports native parameter removal and a JSON-transform
webhook guardrail. The alpha.5 binary hotfix itself did not deploy a service or policy; the later
implementation is recorded below. Do not infer mutation
support from a guardrail's display name or from a successful regex-text test.

## Prefer native removal for simple compatibility rules

A model-specific target can declare `"drop_params": ["reasoning"]`. The gateway
clones the request and removes only that key. Dotted paths such as
`reasoning.effort` are also supported. Four local checks against the reviewed
transform functions extracted from the deployed bundle passed: root removal,
preservation with no drop list, dotted-path removal, and isolation of the original
client body from the gateway's internal default model. Receipt:
`validation/2026-09-07/alpha5/gateway-drop-params-local.json`.
These checks verify transform behavior; they are not live routing acceptance.

For this workspace, a candidate change is to add `drop_params` to the existing
GPT-4.1 default target and introduce an exact `@openai/gpt-4.1` conditional target
before the generic explicit passthrough branch. Keep the generic explicit branch
without a drop list. Do not put this rule at the shared root: drop lists accumulate
through parent/child targets and could remove reasoning for other models too.
Re-check the current management configuration before applying any candidate.
Treat each drop list as part of its model target: changing the default model also
requires reviewing its compatibility rule.

The webhook receives `provider`, `requestType`, metadata and the merged request
JSON. The handler omits request headers from its body. Different hooks may execute
concurrently; do not assume separate transforming hooks form an ordered pipeline.
Use one deterministic compatibility transform if more than simple removal is
needed, and verify actual default/explicit context with redacted live fixtures.

## Acceptance for either mechanism

- Match the resolved provider/model using verified hook context. Establish the
  exact default-route and explicit-route context shapes before implementation.
- Remove only explicitly configured unsupported keys (for example `reasoning`
  for GPT-4.1). Preserve reasoning for capable models. Do not modify unknown
  models or silently rename arbitrary fields without a reviewed mapping.
- Operate on parsed JSON. Preserve prompt text containing parameter names, nested
  tool schemas, credentials, authorization decisions, and all unrelated fields.
- Do not inject a client/proxy root `model` for the gateway-default route. Internal
  gateway routing still selects the provider/model; explicit routes retain theirs.
- Use authenticated, bounded requests and deterministic transforms, with no
  inference or external network calls inside the transformer. Limit body size
  and concurrency appropriately for the supported context budget.
- Keep mandatory synchronous input/output AIRS scans and their failure behavior.
  Record transform decisions as field names and rule versions, without secrets or
  prompt contents. Test hook failure and rollout/rollback before attaching it.
- Prove raw GPT-4.1 requests containing reasoning succeed after transformation,
  reasoning-capable requests retain their effort, malformed JSON fails safely,
  and denied scanner fixtures remain denied on both routing branches.

This policy can cover other clients too. The terminal still needs the picker fix:
a gateway cannot dismiss a stuck local popup or correct local capability display.

## Live implementation — September 7, after alpha.5

The candidate above was tested and corrected. The live gateway first normalizes
`@openai/gpt-4.1` to `gpt-4.1`; the original qualified conditional never matched.
`live-first.json` retains this failure. Native removal works on the fixed default
GPT-4.1 target. Explicit requests now select a candidate branch on the normalized
model, then an authenticated webhook verifies the resolved `openai` provider and
`createModelResponse` API before removing root `reasoning`. Unknown providers
with the same model name receive no transformation. Generic explicit routes have
no drop list or compatibility webhook.

The standalone implementation is [gateway-compatibility](../gateway-compatibility/README.md).
Guardrail `32e37aa9-8dab-4038-89c1-53d09dd40a5e`, slug `pg-airs-t-6116e0`,
uses `default.webhook`, synchronous deny-on-error, a dedicated server-side key,
and a 3000 ms timeout. Current routing version:
`6103fc1a-a966-4a42-8fc1-6e1614e94a5e`. The explicit target includes both this hook
and `pg-prisma-ff3021`: target input hooks replace inherited hooks, so omitting the
scanner from that target would remove its input protection. Output scanning is
inherited. No scanner settings were relaxed.

The six live compatibility cases verify default and explicit GPT-4.1 success,
prompt/nested schema acceptance, supported reasoning preservation on GPT-5 mini,
and invalid reasoning rejection on that capable model. Nested mutation isolation
is checked directly in the server tests; live schema cases use `tool_choice=none`.
A separate forced-function-only, non-streaming response returned 446 because the
existing output scanner sent an empty-text scan and received HTTP 400. That is an
unresolved scanner integration limitation, not a successful tool-call test.

The separate canary proved an invalid webhook credential denies inference (446),
then restoration to the accepted policy permits it (200). Production was not
switched to the failing hook. Existing routing, scan-denial and independent MCP
checks remain in `gateway-contract.json`. Retain failures alongside successes.
Management for this change used the CLI installation's SDK **0.17.0**; the
scanner application still pins SDK 0.21.0. Do not infer the installed operator SDK
version from the scanner package.

Rollback material is in the operator-only directory
`~/.local/share/airs-terminal/operator-recovery/2026-09-07/compatibility/`.
`before.json` is the full original config (version
`9eb657da-1373-4643-b49a-a384ae4d2226`); it includes the original mandatory scanner
policy. Restore its routing body through the management API, verify propagation
with actual default/explicit requests, and only then remove the webhook service.
Reverting solely the service would cause explicit GPT-4.1 requests to fail closed.
The accepted previous image 0.1.0 remains in Harbor for service rollback; current
0.1.1 adds connection limits and strict UTF-8 decoding with the same rule.
