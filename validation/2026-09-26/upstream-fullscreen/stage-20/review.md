# Live dynamic tools and ordered native history

First coherent slice of 6ba6b73657: retain mutable dynamic tool rows, settle
out-of-order completions, mark unfinished calls interrupted at turn end, and
queue terminal output behind unfinished live calls. The native queue emits at
most 32 settled cells per frame and schedules another frame when needed. It
tracks canonical retained owners through consolidation, removal and rollback,
so prefix fragments already emitted to the terminal are not emitted twice.
Historical in-progress records outside the live queue cannot block new history.

AIRS-specific adaptations preserve the existing `flush_interrupt_activity`
boundary instead of upstream `flush_interrupt_queue`: turn cleanup must not open
queued approvals or questions. The dynamic finalization regression explicitly
queues a question and verifies it remains queued with no active prompt. Existing
subagent and AIRS authentication-recovery regressions are included in the gate.
No upstream prompt-revert/voice state is imported into the different AIRS event
path. AIRS thread replacement already resets transcript state, including the new
queue; existing rollback/reflow tests exercise that path.

Scoped tests cover 4,097 queued outputs, completed results emitted once before
later output, stale historical records, rollback replay larger than a frame,
usage refresh during deferral, out-of-order calls and interruption. An additional
regression consolidates both fully unprinted and partly printed streams and
verifies only remaining source is emitted, in order, exactly once.

Initial 288/289 passed; the one snapshot mismatch was AIRS's existing cyan tool
label, now reviewed and adapted. Final expanded gate: 313/313 passed, no retries.
The exact first attempt is retained. Scoped lint/format receipts follow.

The code remains narrowly scoped (under 800 changed source/test lines). Activity
group projection, richer replay and paginated-history changes from the same
upstream commit are separate subsequent slices. No standalone fullscreen score,
Mac acceptance or package publication is claimed here.

Scoped lint completed with no warnings or fixes. Formatting passed; unrelated
preexisting Python formatter churn was restored.
