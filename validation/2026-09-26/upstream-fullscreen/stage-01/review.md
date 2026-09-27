# Logical source foundation — validated prerequisite

Adapts the source/wrapping portion of upstream `fcaa035438`. Shared logical text
and styles, exact byte ranges, gutter bytes and right margin metadata accompany
visible rows. Grapheme projection preserves compound characters; first-fit reuse
is restricted to compatible tokenizers, widths and complete-word boundaries.
Hyperlinks use authoritative source ranges instead of matching guessed display
text. Source-only whitespace changes invalidate prose previews. Visual equality
remains separate from source-aware cache equality.

The only three-way conflict involved an upstream trusted-workspace-file
constructor absent from this fork. Keep the existing destination variants and
security checks; expose only column remapping. No new trusted-file constructor,
voice, spoken artifacts, animation, prompt-background or credential behavior.
Existing AIRS user-prompt URL handling is retained for the next propagation stage.

This coherent stage includes the Markdown flush and live agent/plan cache
consumers as well as separate wrapping regressions and preview tests. Production
changes remain below 500 lines; new source metadata
lives in its own 114-line private module. Existing wrapping routines are adapted
in place so the exact ranges remain owned by the same algorithms as styled
slicing. Do not count this prerequisite as a completed fullscreen feature.

Focused local checks cover wrapping, hyperlinks and previews with the same
TUI/config root package set used by the preceding feature. Later propagation
adds explicit metadata comparisons to the combined math/list streaming matrix
and hard-line handling for spatial equations. Connected full local/native
validation, fullscreen/search/input ownership and signed distribution remain.

## Findings during validation

The initial 320-case subset had 319 passes and one source-only preview failure.
The first stage split had omitted unwrapped Markdown source metadata and live
agent/plan source equality, so those narrow consumers moved into this stage.
The next run had two failures (blockquote provenance and the same preview),
revealing an AIRS preimage difference: Markdown's wrapping branch still called
the legacy remapper rather than the shared hyperlink wrapper used upstream.
That branch now uses the authoritative-range wrapper. Both original assertions
are preserved. Initial and intermediate failure receipts are retained; the
corrected subset passes **320/320**, with retries disabled. The 4,493 excluded tests are filter exclusions, not platform skips. Scoped config/core/TUI lint has zero warnings and makes no fixes; formatting passes with only preexisting unrelated Python churn restored. No new snapshot has been accepted or test weakened. Connected native/fullscreen feature gates remain pending.
