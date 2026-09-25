# Gateway routing and MCP recovery: 0.1.3-alpha.5.mcp.1

Runtime, packaging and frozen tooling: `fd038ac880d2155654b3db3d4032290460b69833`. The Apple Silicon package is
Developer ID signed, Apple notarized, and published under `mac-preview` at
https://npm.cdot.io. Stable and Linux distributions are unchanged; CLI remains 7.1.5.

This preview adds actionable MCP recovery, guided gateway MCP onboarding and
conversation-scoped `/config` and `/model`. Changing config clears the override;
selecting the same config preserves it. Following config routing is the default.
A fixed minimal request verifies the proposed pair before application; gateway
policy remains authoritative. Settings are applied only on confirmation from the
session. Failure/cancellation preserves the prior pair and draft. Resume restores
routing; new conversations use environment defaults. Requested routing is not a
claim about the gateway's effective backend. Inference, MCP OAuth and Jev credential
boundaries remain separate.

All 13 restoration regression tests passed. Full GNU: 18455 passed,
3 failed, 34 skipped. The retained raw baseline
comparison identifies the inherited failures; the suite is not described as green.
The first signed candidate was withheld after nine NEW restoration failures; the
corrected runtime was rebuilt and signed again. The superseded failure logs remain.

Exact candidate and fresh registry installs passed all ten standard stages, native
TypeSafe checks and the actual agent approval path. Stable and alpha.4 roundtrips
preserved native credentials and conversation state. These are isolated fixtures,
not owner SSO/ServiceNow or paid Jev acceptance. Documentation passed 23 browser
checks; all 16 live pages match reviewed content after CSS hash normalization.

Implementation evidence is in the adjacent mcp-routing-phase1 through phase4
folders. READINESS.json scores the bounded phases at 9/10; owner real-account
routing acceptance remains separate. SHA256SUMS inventories this evidence and the
subsequent immutable Git audit verifies stage references and output hashes.
