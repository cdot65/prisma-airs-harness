# Transcript layout cache

Adapted upstream 30ab0ee65b. Measurement and painting share retained layouts;
mutable sources refresh on each frame. Width, animation tick, syntax theme and
terminal palette changes invalidate cached layouts. Retention is limited to 64
entries and 8 MiB of visible text, with the upstream exception for one oversized
visible entry. This is not a total-process memory bound: metadata and transient
layouts also occupy memory.

AIRS retains its three live sources: active output, asynchronous usage cards and
rate-limit hints. The usage composite already declares mutable state; the leaf
card now does too. A regression completes the actual usage handle without a
revision change and verifies that the transcript replaces the loading message.
No upstream voice or command-center feature is imported.

Existing long-URL, reasoning, image and plan height regressions now measure the
new TextLayout rather than the removed HistoryCell height method. The snapshot
expands styles, wide characters, wrapping and hyperlinks. The only AIRS-specific
expected differences are existing cyan hyperlinks and shortcut styling.
Additional regressions exercise the byte budget, oversized-entry exception,
separator invalidation and weak ownership of source cells.

The first compiler attempt caught an overly broad test extraction; it was
corrected to the one intended new test. The first execution passed 338 of 340
checks. The snapshot differences above were reviewed and accepted. The budget
stress test initially used a 65,535-column viewport; Ratatui's existing u16
word-wrapper overflowed on that artificial width. The budget check now uses a
120-column viewport. The maximum-width wrapping boundary remains a known issue
to re-evaluate when the subsequent source-aware text layout replaces this
intermediate renderer; this stage does not claim that edge is fixed. Both failed
attempts are retained alongside the final receipts.

This is a prerequisite, not a separately scored complete fullscreen feature.
Connected selection/search, rendering bounds and native acceptance remain gates.

Final focused run: 340/340 passed, zero retries. Scoped lint passed with one
reviewed closure-to-method-reference fix in the usage test and no warnings.
Formatting passed; unrelated existing Python formatting churn was restored.
