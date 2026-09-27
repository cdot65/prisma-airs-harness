# Exploration grouping — fullscreen prerequisite

Adapted upstream `71406edbd5`: adjacent read/list/search operations remain in one compact group across reasoning summaries and failed commands. Expanded history preserves command output, failure status and chronological reasoning; raw export retains its existing omission of reasoning. Search exit 1 is reported neutrally because it can mean no matches. Compound commands report one command-level status rather than assigning it to each parsed action.

Review covered active/orphan command completion, replay without begin events, boundary commands, assistant messages and repeated reasoning. No model context, protocol, gateway or credentials change. The upstream test's absent model_context field was removed; the existing AIRS work separator was preserved and explicitly asserted in both live/replay paths before comparing the remaining identical snapshot.

Focused command/history-replay/streaming validation: 222/222, retries disabled. Initial compile failure and subsequent separator expectation failures are retained; they are not hidden as retries. Existing snapshots pass without acceptance. This stage is a prerequisite, not a standalone feature score; connected native/fullscreen and release validation remain pending.

Scoped config/core/TUI lint passed with zero warnings and no fixes. Formatting passed; newly introduced unrelated Python formatter churn was restored.
