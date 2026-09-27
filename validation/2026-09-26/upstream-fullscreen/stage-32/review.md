# Environment-scoped TUI mode picker

Selective adaptation of upstream 2f34d236f5. `/tui` chooses Scrollback or Fullscreen
for the next launch. A confirmed choice writes only the selected environment's
user configuration and updates the saved preference after a successful write.
It never switches terminal ownership midway through a session. The picker states
that restart is required and launch overrides still apply. Both configured key
hints and narrow terminal layouts use existing AIRS picker primitives.

Adversarial review checked environment isolation, unrelated config preservation,
unsent drafts, cancellation, failed writes and launch override messaging. Tests
compare the complete local settings object and reload the selected config. A real
PTY runs three processes, proving both transitions happen only after restart.
Snapshots cover both current choices at 40 and 80 columns. The first upstream
layout clipped at 40 columns; descriptions now stack and compact key hints fit.

Full CLI/TUI/config/features validation passed 6,206 executed tests with no
failures or retries; seven existing skips (74.076 seconds execution; 76.195 including startup). Earlier
compile and focused snapshot failures are retained. Scoped CLI/core/TUI/config/features lint passed in 120.705 seconds with zero warnings or fixes. Formatting passed in 19.130 seconds; unrelated Python formatting was restored. Tests were not rerun solely for lint/format.
No gateway/authentication, dependencies, protocol or schema changes.

Bounded source scores: implementation 9/10, code quality 9/10, design 9/10, feature completeness 9/10, supported by the environment isolation, failure, snapshot and real PTY tests above. This is self-review of the
bounded source feature, not independent certification or a signed-package gate.
Native Mac/GNU, whole-feature interaction acceptance and publication remain.
