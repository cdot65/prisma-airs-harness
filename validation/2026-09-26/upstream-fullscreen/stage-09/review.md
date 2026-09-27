# Shared activity disclosure interfaces — fullscreen prerequisite

Adapted shared activity/detail interfaces from `321c50fc2c`. Retained detail cells are shared immutably; prepending older calls shifts their positions without recreating live state or changing text. Existing command rendering uses a temporary compatibility bridge until its source-aware conversion lands. Compact/live-raw/expanded history interfaces preserve default behavior; composite cells forward compact/live-raw rendering.

Excluded dormant warning-picker metadata because no selected connected consumer uses it. AIRS warning diagnostics remain intact. The initial test invocation caught a leftover import-conflict marker; it was corrected and the failed compile receipt retained.

Local command/history/replay subset:289/289 passed, zero retries. New behavioral tests check older-history prepend ordering, reasoning omitted from raw while terminal input remains, preserved hyperlink destinations and shared logical-source identity. Existing snapshots pass without changes. Temporary compatibility methods and activity-preview dead-code allowance must be removed when compact command rendering lands; no standalone feature score is claimed.

Scoped config/core/TUI lint passed with zero warnings and no fixes. Formatting passed; unrelated Python churn was restored.
