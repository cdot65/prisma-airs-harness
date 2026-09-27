# Entry-anchored transcript scrolling and navigation cancellation

Adapted a63ae005d4 and its directly related correction 392f56a611. Reading state
now identifies a retained entry plus a row rather than a transcript-wide offset.
Prepending history, consolidating streamed output and resizing clamp or preserve
that anchor. The footer no longer formats all offscreen history to calculate an
intermediate percentage. Pending Home navigation yields to subsequent scrolling,
End or prompt navigation without discarding the history request already in flight.

AIRS retains its user-message palette and existing modal/input ownership. Boxed
transcript/static variants do not import the upstream analytics variant. The
upstream `spoken` fixture field was omitted because this fork does not implement
that voice interface. No inference, gateway, credential or model-context behavior
is changed.

Regression migration review:

- Prior per-cell height and generic pager scroll tests move to the entry-based
  viewport tests: prepend, consolidation, resize, adjacent-entry separators,
  mutable per-frame layouts and offscreen formatting bounds.
- Existing source-aware URL tests are strengthened to verify the complete linked
  destination across wrapped history and live output.
- The earlier hyperlink/Unicode scrolled-buffer equivalence check is preserved
  against TextLayout and a full HyperlinkParagraph reference. Static pager
  wrappers retain their independent clipped-buffer comparison and maximum offset.
- Highlight-cache reuse remains covered with viewport-aware measurements; unseen
  entries are not eagerly formatted. New backtracking snapshots verify only text
  is reversed, with gutters and padding unchanged. Existing previous/next edit
  footer-hint assertions are both retained.
- Home cancellation runs before and after the first draw, plus the actual app's
  prompt-backtracking path. Passive highlight restoration does not cancel Home.

Initial focused execution passed 344/345; the only failed snapshot expected
upstream RGB 249 while AIRS uses its existing RGB 244 user-message background.
The geometry and highlighted text were unchanged. That snapshot was adapted,
and a full local TUI/config run follows. Failed receipts are retained.

This is one coherent viewport replacement; its larger diff consists largely of
replacing generic pager machinery and migrating tests. Production ownership is
split between the approximately 420-line view and 360-line overlay, with layout,
text and dedicated regression modules. Fullscreen selection/search and native
release gates remain pending; no standalone complete-feature score is claimed.

Final full local TUI/config suite: 4,868 executed tests passed, six skipped,
zero retries. Scoped lint passed with two reviewed iterator-to-contains test
simplifications and no warnings. Formatting passed.
