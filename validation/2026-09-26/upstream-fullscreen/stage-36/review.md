# AIRS recap source fidelity

The AIRS checkpoint recap keeps its labeled rule, spacing and indented body.
Rendering now carries logical body provenance and web-link metadata, using the
existing URL-aware wrapper. Copy/search join soft wraps while preserving hard
breaks, blank lines, indentation, tabs and Unicode. Body gutters remain outside
the source. Selecting the decorative heading includes it as painted; raw history
continues to use the plain Conversation recap heading. No upstream alternate recap
layout or next-action product behavior is imported.

The cell is isolated in a small history_cell/recap module. The shared source
wrapper and TextLayout handle reflow; no renderer-specific parsing heuristics or
new dependencies were added. Tests connect the real cell to TextLayout, verify
body selection offsets and source across resize, preserve link destinations and
bounded rows for long URLs, compare painted buffers and snapshot 24/40/80-column
layouts. All existing recap generation and raw-history checks continue to pass.

Focused tests passed 520/521; only the new reviewed snapshot needed acceptance.
Full CLI/TUI/config/features passed **6,217/6,217**, seven existing skips, zero
retries (74.534 seconds execution; 76.606 total). Scoped lint passed (115.942 seconds), with no warnings or fixes. Formatting passed (18.909 seconds); unrelated Python formatting churn was restored.
The Mac-only pagination snapshot correction from native-stage34 is also included;
it is expectation-only and will be checked in the final native run.

No authentication, gateway, protocol/schema or dependency changes. This is
self-review; final native/GNU/docs/signed-package acceptance remains outstanding.

## Evidence-based self-review

| Dimension | Score | Basis |
|---|---|---|
| Implementation | 9/10 | Real recap rendering and source-aware selection agree across widths. |
| Code quality | 9/10 | Dedicated module and shared wrapper; scoped lint and formatting pass. |
| Design | 9/10 | Preserves AIRS heading and raw history while making body provenance explicit. |
| Feature completeness | 9/10 | Unicode, URLs, whitespace, selection and reflow covered; final platform release gates remain separate. |
