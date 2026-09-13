# AIRS endpoint and feature reconciliation

This is the implementer inventory. Independent review remains pending.

| Request surface | Authority / decision | Evidence required |
| --- | --- | --- |
| Responses and local tool continuations | Selected AIRS provider; default root model omitted, qualified model retained; serial tool requests; HTTP redirects prohibited | Native assertions, interactive route switch and live scans |
| Local compaction | Same Responses client/provider; AIRS provider still advertises remote compaction unsupported | Forced local compaction and route assertions |
| Memory summarization | Gateway request-model mapping and effort filtering retained in ModelClient | Client suite; do not claim a live memory scan from an unrelated call |
| Thread titles | Hidden temporary thread inherits current provider/model; later untitled turns can retry naming | Native title request-to-originating-turn assertions |
| WebSocket transport | Disabled in AIRS generated config; inherited API request type retains optional model | Source review and request serialization tests |
| Realtime | Direct rejection on a gateway provider before auth, override endpoint or network work; feature pinned off | Core no-request integration case and native feature override case |
| Guardian V2 classifier | Deferred, feature pinned off: fixed classifier model/new HTTP path needs a separate gateway contract | Native override case; no claim of classifier support |
| Hosted apps/plugins/image generation/remote control/discovery/Code Mode | Deferred runtime feature pins apply beyond setup output, including older homes and later feature mutation | Native old-environment override case |
| Account usage/reset/reserve flows | Require applicable upstream account capability; AIRS retains its own authentication, catalog and no hosted onboarding | AIRS startup/doctor/model picker tests and upstream account capability tests |
| MCP discovery/calls | Selected home owns server map, per-resource credential helper, allowlist and header rules; upstream tool catalog refresh retained | Live authorized scan, wrong-destination denial, recovery/no-replay tests |
| Identity browser/device/refresh | Bound issuer/client/audience and native store; never substituted for runtime scanner/management credentials | OIDC/native-store lifecycle and isolation receipts |
| Managed Prisma CLI | Absolute managed wrapper, exact CLI/SDK pins, selected trusted config; no automatic project dotenv | CLI contract, installed invocation and agent skill workflows |
| Worktrees / inline questions | Opt-in; inherit gateway/session permissions and catalog authority | Native worktree create/resume/fork; question submit/draft; live inline local effects |

All eight Prisma skill assets are byte-unchanged from alpha.11. CLI and SDK dependency versions are unchanged. This establishes source preservation alongside executable contract tests; it does not claim a live red-team campaign, model scan or DLP detection when those services were not exercised.

Raw service credentials and traces remain outside version control. Missing live scan completion, missing artifact lineage or incomplete independent review must remain a gate failure in the evaluation ledger.
