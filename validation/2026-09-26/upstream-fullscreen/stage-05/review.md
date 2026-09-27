# Ordered activity details — fullscreen prerequisite

Adapted the generic/exec portion of upstream `de33f2d22a`. Shared activity storage retains reasoning after the calls already started, even when a follow-up command has not completed. History cells explicitly accept or decline reasoning; ordinary non-exploration cells preserve normal insertion. Expanded history includes the ordered summaries, while compact/raw views retain their existing contracts. This only changes presentation and does not rewrite model history.

Excluded unrelated computer-use activity modules and tests; AIRS continues to handle its gateway MCP services through existing cells. A new regression snapshot covers reasoning during the second running command, in addition to the prior live/replay and overlapping-command matrix.

Local command/history-replay/streaming subset: 223/223 passed, zero retries. New ordering snapshot inspected; existing snapshots pass unchanged. Connected fullscreen/native/release validation remains pending and this prerequisite has no standalone feature score.

Scoped config/core/TUI lint passed with zero warnings and no fixes. Formatting passed; unrelated Python formatter churn was restored.
