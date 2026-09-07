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
webhook guardrail. No new service or policy has been deployed by this hotfix. Do not infer mutation
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
