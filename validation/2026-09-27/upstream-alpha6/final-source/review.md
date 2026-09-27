# Final source review — release gate pending

Runtime and frozen tooling: `f7e03c65933041621ef9c6aecc0062610ae0a9b5`.
Preview: `0.1.3-alpha.6.mcp.1`, Apple Silicon only, `mac-preview` channel.

Final native CLI/TUI/config/features: 6,254 passed, zero retries, zero failures;
359.824 seconds execution. Source parity: all 7,407 tracked Rust files matched.
Stage36 local scoped tests: 6,217 passed; scoped lint and formatting passed.
These are exact-source native checks plus the pre-version-stamp local checks,
not installed signed-package acceptance.

Boundaries retained: gateway-only inference and remote MCP; CAS/workspace-key
inference and gateway-facing MCP authorization; independent Jev credentials and
actual-agent approval; confirmed config/model pair on cancel/denial; no automatic
replay after policy denial. Fullscreen stays opt-in and environment-scoped.

Feature evidence is in validation/2026-09-26, including failed diagnostic receipts
and closure. Source review is adversarial self-review, not independent certification.
Final feature-completeness scores remain contingent on release acceptance below.

Open gates: GNU full workspace run3863 (review any failure against retained stable
assertions); release build3864; Developer ID/notarization; installed candidate
checks including native store and actual-agent Jev permission; fresh registry
verification and stable/previous-preview roundtrips; published documentation and
owner real-account/visual acceptance instructions. No production account acceptance
is inferred from fixtures. Stable and Linux distribution are unchanged.

## Release-profile warning review

The final Mac release compiler reports two unused cloud-tasks imports
(HttpClientFactory, OutboundProxyPolicy) and an app-server unused_mut on
loader_overrides_with_test_user_config_file. All three declarations exist in
pre-adoption baseline f8ba5ec356bf97fc5b98fc7dbda252f69d091265. Their uses or
mutation are gated by debug_assertions. No suppression or production change was
added. Retain the release build log and do not claim a warning-free full release
build; scoped affected lint results remain separately accurate.

## Final GNU workspace result

Run3863 finished: 18,995 passed, three failed, 35 skipped, zero retry-pass tests,
2,010.099 seconds execution. Both attempts of all three failures exactly match
the retained stable assertions after the documented normalization. The suite is
not all green. WORKSPACE-REVIEW and WORKSPACE-BASELINE-COMPARISON contain the proof.
The skip count increased by one since the math checkpoint: owned_screen_lifecycle_child
is intentionally ignored as a standalone test and is invoked with --ignored by
owned_screen_preserves_inline_viewport_across_overlay_handoff_and_resume, which passed.
The related real-PTY owned-screen entry/exit test also passed. No behavioral test
was disabled to obtain this result.
