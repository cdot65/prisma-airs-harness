# Composer pointer selection — validation in progress

Selective adaptation of e39839509a and its f3f07753ca shared-selection prerequisite.
The fullscreen composer accepts click-to-position, drag, double-click words and
triple-click logical lines, with Unicode and atomic image/paste boundaries.
Replacing selected content reconciles removed attachment and paste payloads;
Vim edit history and typing buffers retain the existing AIRS handling.

Draft copy uses the existing TUI clipboard owner, lease and confirmed/unconfirmed
status. Selection survives copy. Private TypeSafe/MCP views and approval dialogs
retain input ownership. Tests create an actual underlying draft selection before
opening private dialogs, then verify neither copy nor agent submission leaks the
private entry. Transcript drag retains ownership across the composer boundary;
an explicit composer gesture ends browsing before h/l resume ordinary typing.

AIRS adaptation keeps handle_paste sanitization instead of importing a duplicate
paste implementation, and puts bottom-pane routing in a dedicated adapter module.
The shared helper extraction changes no rendering/source contract; UTF-8 offsets
are clamped and key release does not copy. Initial missing-helper compile failure
is retained. Coordinated composer/textarea/shared selection changes exceed the
usual patch-size guideline because hit-testing, selection mutation and payload
reconciliation share state. New logic uses separate modules.

Focused checks passed 798/799; the upstream popup fixture used stale composer
coordinates after AIRS footer layout changed. It now clicks the newly painted
composer. Full CLI/TUI/config/features run passed 6,201/6,202, seven existing skips,
no retries (72.98 seconds execution, 252.32 including rebuild). Only the new
browsing-to-typing fixture failed because rapid synthetic h/l remained buffered
before an explicit paste. It now dispatches End to exercise the normal flush path.
No production changes followed the full run. Focused closure passed 154/154 with no retries (11.08 seconds execution,
91.03 including rebuild). This is a full run plus focused test-only closure, not
an all-green full-run claim. Scoped CLI/core/TUI/config/features lint passed in 161.48 seconds with zero warnings or fixes. Formatting passed in 18.59 seconds; unrelated Python changes were restored. Tests were not repeated solely for lint/format. All failures remain in the receipts.

No authentication, gateway, schema or dependency changes. This is adversarial
self-review, not independent certification. Complete-feature scoring and native
Mac/GNU/documentation/signed-package/registry acceptance remain pending.
