# Disposable prose preview prerequisite

Adapts upstream `b966240bea` separately because display-math `a8964cb1ba` depends on its preview ownership. Complete local TUI/config suite: 4,773 passed, six skipped, no retries. Scoped lint passed with no warnings. Formatting passed; unrelated Python formatter churn was restored.

Preview contents are bounded to 8 KiB at a UTF-8 boundary and are never counted as stable lines or injected into model context. Newline and finalization use the unchanged source. Table/code holdback, raw-mode behavior, resize, plan/agent redraws, recovery status and inline visualization ownership remain covered. Both new snapshots were inspected; their plan-header blank lines intentionally match existing rendering. Existing recovery tests now check retained status state while the visible partial response replaces the spinner. No gateway/authentication changes.

Self-review found this prerequisite missing from the original dependency map before the display patch could apply; the failed patch was atomic. Connected native math, source-parity, terminal tests and feature scores remain pending. This is not a release artifact or an independent review certification.
