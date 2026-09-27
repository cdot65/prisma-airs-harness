# September 26 adoption: 0.1.3-alpha.6.mcp.1

Runtime, packaging and frozen tooling: `f7e03c65933041621ef9c6aecc0062610ae0a9b5`. Apple Silicon is Developer ID
signed and Apple notarized; `mac-preview` is published at https://npm.cdot.io.
Stable remains 0.1.2; Linux distributions are deferred. Bundled CLI is 7.1.5.

This completes the accepted sandbox/TLS, transport/session, terminal polish and
opt-in fullscreen/search slices. Fullscreen is selected with `/tui` per environment
and takes effect after restart. Inline remains the default. F3 searches; F4 controls
activity detail. Copy/selection preserve logical source text and private dialogs
retain input ownership. Gateway inference and remote MCP remain gateway-only;
CAS/OAuth, workspace-key inference and approved Jev key access remain separate.

Final Mac source suite: 6,254 passed, zero retries. Local stage36: 6,217 passed,
scoped lint/format passed. Full GNU: 18,995 passed, three assertion-matched inherited
failures, 35 skips, zero retry-only passes. The suite is not wholly green. Three
preexisting release-profile warnings remain (two debug-only imports and one
mutable binding); source review is retained in ../final-source.

Exact candidate and fresh registry installs passed all ten standard stages,
native TypeSafe access and the actual agent approval boundary. Stable 0.1.2 and
previous alpha.5 upgrade/rollback passed. The legacy evidence directory named
alpha4-roundtrip contains the actual alpha.5 roundtrip; inspect version fields.
No owner login, real ServiceNow call or paid Jev judgment is claimed by fixtures.
Documentation passed 23 browser checks and all 16 live page comparisons.

READINESS.json records four separate scores for each bounded feature and delivery.
Detailed corrective iterations and scope are in ../../../2026-09-26 (repository
validation/2026-09-26) and the vault PRD. These are adversarial self-review scores,
not independent certification. Owner testing covers real inference, ServiceNow,
restart/reuse, config/model routing, optional Jev and terminal visual/clipboard
behavior. The vault acceptance checklist contains install and rollback commands.

This final release record supersedes the historical open delivery gates in
../final-source/review.md. SHA256SUMS inventories all retained release evidence;
the separate immutable Git audit validates stage outputs and dependencies.
