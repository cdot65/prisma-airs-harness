# Tmux mouse policy and terminal-protocol acceptance

Selective adaptation of 4b664e0ef0. One shared tmux probe retrieves keyboard format
and mouse policy, targeting TMUX_PANE when available. Only explicit 0/off disables
mouse capture. Missing settings, failed probes and invalid output preserve the
previous policy. The older keyboard-format fallback remains. Alternate-screen
entry/restoration refreshes policy; every subsequent overlay mouse request honors
it. Existing cleanup and retry-after-partial-write behavior remains intact.

The fork has no Usage overlay, so that upstream fixture case was removed; all
actual AIRS overlay variants remain in the matrix. No unrelated keyboard detection
or product surfaces were imported. Tests cover pane arguments, policy values,
fallbacks, malformed/failed results and policy changes across owned/overlay modes.

A new real PTY test sends SGR selection and right-click input over the SSH terminal
clipboard path. It verifies the exact hello payload in OSC52 twice, preserves the
draft and unconfirmed selection, and replaces only selected text afterward. It
never reads or writes the host clipboard. This exercises terminal decoding and
production event routing rather than injecting Rust mouse event structures.

Initial compile failed on the upstream-only Usage enum. Focused tests then passed
86/86. The first full run passed 6,212/6,213 with seven existing skips and no retries;
the new PTY test sampled cursor position between text painting and cursor restore.
The test now locates painted cells. A second fixture correction handles vt100's
empty-string representation of blank cells. No production changes followed the
first full run. Ten stress iterations passed with no retries (5.496 seconds test
execution). The final full CLI/TUI/config/features run passed **6,213/6,213**, seven
existing skips and zero retries (80.677 seconds execution; 82.687 total). All
failures are retained. Scoped CLI/core/TUI/config/features lint passed in 154.827 seconds with two mechanical method-reference fixes in the PTY test and no remaining warnings. Formatting passed in 18.921 seconds; unrelated Python formatting was restored. Tests were not repeated solely for lint/format.

Bounded source scores: implementation 9/10, code quality 9/10, design 9/10 and feature completeness 9/10, supported by the policy matrix, full run and repeated real terminal acceptance. Native Mac delivery remains a separate gate.

No authentication, gateway, config, dependency or schema changes. Native Apple
Silicon terminal and actual signed-package acceptance remain final delivery gates.
This is evidence-backed self-review, not independent certification.
