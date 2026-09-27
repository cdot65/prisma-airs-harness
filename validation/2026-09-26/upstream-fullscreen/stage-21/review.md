# Live/replayed activity projection

This slice completes the exploration-group prerequisite omitted from the earlier
rich-tool restoration and applies the corresponding live/replay parts of
6ba6b73657. Restored commands use the live compatibility rules. Groups stop at
visible items and turn boundaries; reasoning/terminal bookkeeping stays ordered
inside compatible exploration groups. Gateway MCP calls remain ordinary independent
calls, including servers with names that upstream treats as computer activity.
No computer mode or direct upstream MCP route is introduced.

MCP completion uses the same typed converter for live and saved records, preserving
errors and reporting a missing completed result explicitly. Completed saved file
changes render their diff without requiring a prior start notification. Completed
exploration is flushed at turn completion even without an answer or saved label.
Finalized plan fragments are retained before consolidation, preserving source.

Reasoning presentation follows the adopted activity model: completed reasoning is
available in the detailed transcript, with status updates during execution, and
does not add separate compact rows. Raw reasoning still requires the existing
visibility setting; when enabled, restored detail retains summary plus raw text,
matching replay. This is a presentation change, not a model-context or rollout
rewrite. Exact persisted item IDs are retained for the following page-boundary
integration. The getter's temporary dead-code allowance must be removed there.

AIRS adaptations keep existing placeholder-header, deferred usage, subagent event
and authentication-recovery behavior. Source identity is passed from the completed
Reasoning item rather than importing unrelated upstream reasoning-resume state.
The grouped projection lives in a dedicated module. Upstream fixtures are adapted
to the fork's schema, without adding voice/model-context/MCP-app protocol fields.

Compiler attempts exposed the factory's erased return type and two fixture
conversions/imports. Their receipts are retained. Behavioral results and reviewed
snapshot changes are recorded below after execution. No completed fullscreen or
native release claim is made for this prerequisite.

## Validation and adversarial self-review

Final focused run: 307/307 passed, zero skips, failures or retries. Coverage
includes live/restored grouping, completed file changes, missing MCP results,
reasoning, plan retention, subagents and AIRS recovery. The first behavioral run
had nine old presentation expectations; reviewed snapshots and retained detailed
content assertions document their deliberate change. Earlier compiler attempts
are preserved rather than represented as passing runs.

Scoped `just fix --locked -p codex-config -p codex-core -p codex-tui` passed
with no warnings. `just fmt` passed; unrelated pre-existing Python formatter
churn was restored. No tests were rerun solely after formatting/lint.

Review checked that ordinary gateway MCP results remain separate from exploration,
turn boundaries flush groups, raw reasoning remains opt-in, and completed source
IDs survive replay. This is a prerequisite, not a completed feature score.
Pagination integration, connected fullscreen acceptance, native compilation,
signing and registry acceptance remain pending. No new package is claimed.

The detailed reasoning snapshot intentionally retains two-space indentation on
its blank line; diff whitespace review identified only that rendered blank line.
