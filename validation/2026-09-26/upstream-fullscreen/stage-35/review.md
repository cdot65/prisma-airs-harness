# Shift-click transcript selection extension

Selective adaptation of 39598ed178. A Shift-only left press extends an existing
nonempty selection from its original anchor and text unit, then allows dragging.
Repeated extension can reverse direction; an ordinary click resets selection.
The change reuses AIRS's source-anchored selection snapshot and existing extension
logic. It does not import upstream's separate automatic link-on-release behavior,
pressed-link state or pointer-origin tracking. Existing Ctrl/Super link activation
keeps precedence, and ordinary typing/private input routing is unchanged.

Two upstream behavioral tests were isolated in a dedicated module. They cover
both extension directions, word/line/grapheme units, continued dragging and normal
click reset. The reviewed word-selection snapshot passes unchanged. Focused
selection/owned-transcript/real-SGR checks passed **208/208**, zero retries
(2.923 seconds execution; 181.524 including rebuild). Full CLI/TUI/config/features passed all 6,215 checks, seven existing skips and no retries (84.283 seconds execution; 86.339 total). Scoped CLI/core/TUI/config/features lint passed in 91.713 seconds with zero warnings/fixes. Formatting passed in 18.193 seconds; unrelated Python formatting was restored. Tests were not repeated solely for lint/format.

Bounded source scores: implementation 9/10, code quality 9/10, design 9/10 and feature completeness 9/10.

No auth, gateway, config, schema or dependency changes. Complete native Mac/GNU
and signed-package acceptance remain separate delivery gates. This is self-review,
not independent certification.
