# Right-click selection copy

Selective adaptation of upstream 2db0ab7af1. A right-button press inside the
transcript or composer copies an existing nonempty selection using the TUI-owned
clipboard path. Release/outside presses do not copy. Confirmed right-click copy
clears the selection; failure and unconfirmed terminal delivery retain it.
Keyboard composer copies retain selection. Draft, caret and reading position are
preserved. The app refreshes layout before mouse copy hit testing.

The AIRS adapter preserves handle_paste sanitization, selected-payload
reconciliation, prompt-browsing cancellation and private-view input ownership.
Bottom-pane forwarding stays in the small composer_mouse module rather than the
large parent module. Tests cover all copy outcomes, resize bounds, no-selection
and outside/release behavior, cross-surface ownership and private TypeSafe/MCP
views. No authentication, gateway, config, dependency or schema changes.

Focused validation passed 162 of 163 checks; the imported app snapshot differed
only in the existing AIRS footer height and model label casing. The reviewed
snapshot was accepted. No production change followed; the full run additionally
exercises right-click copy exclusion in both private dialogs with Find on/off.
Full CLI/TUI/config/features passed all 6,209 executed checks, seven existing skips and zero retries (80.334 seconds execution; 161.707 including rebuild). Scoped CLI/core/TUI/config/features lint passed in 100.641 seconds with zero warnings/fixes. Formatting passed in 17.712 seconds; unrelated Python changes were restored. Tests were not rerun solely for lint/format.

Bounded source scores: implementation 9/10, code quality 9/10, design 9/10, feature completeness 9/10. This is evidence-backed self-review, not independent certification. Native SGR terminal/clipboard acceptance
remains part of the final feature gate. No release claim is made here.
