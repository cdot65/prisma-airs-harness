# Math rendering — connected review, workspace gate pending

Runtime source: `01d1a56d7d40cf63cca0cc8bb502b21565f62235`.
This is an adversarial engineering self-review, not independent certification.
Scores remain unassigned until the full GNU workspace result is compared with
the retained baseline. Signed distribution belongs to the final delivery gate.

## Scope and design

The rendering copy recognizes bounded inline and standalone display TeX before
Markdown consumes escapes. Exact byte offsets remain available for citations;
original conversation/model context and raw transcript source are preserved.
Protected code, links and HTML stay with their original renderer. Unsupported,
ambiguous, oversized, deeply nested or spatially unsuitable expressions retain
their source. Conversion is limited to 4 KiB expressions, 16 rows and 256 columns.
This is a terminal notation subset, not a complete TeX engine.

Display matching owns its closer even when conversion is rejected. Streaming
holds a supported unfinished display outside committed scrollback, while rejected
prose/shell prefixes use bounded lookahead. Disposable partial-prose previews have
an 8 KiB UTF-8-safe budget; completion renders the retained full source. Narrow
terminals do not wrap spatial formulas into misleading geometry.

`[tui.rendering] math = false` disables conversion independently of list rendering,
animations and raw mode. Recognition/masking remains active when disabled so
Markdown cannot eat TeX backslashes. Existing client-owned preference propagation
handles initial previews and widget replacement; backend session configuration
does not silently take ownership of this display setting. The actual config
schema generator ran and its default/property changes were inspected.

No dependencies, provider calls, external renderers, credential flows, gateway
routing, model selection or model-visible content were added. Parser/layout
modules remain below 500 lines. Structured aligned layout is a private module.

## Executable evidence

- Every prerequisite retains its tests, lint, formatting and inspected snapshots
  in `stage-01` through `stage-05`, including the required `stage-01b` preview
  dependency. Receipt logs preserve initial failures as well as their closure.
- Final initial local TUI/config run: 4,802 passes, one new-snapshot assertion,
  six skips, no retries. The inspected new snapshot's focused rerun passed. This
  is deliberately not described as a fully passing initial run.
- Final Apple Silicon TUI/config run: 4,809 passes, six skips, no retries. All
  four new real-PTY math cases and three existing list cases are included.
- Combined streaming tests exercise every list/math setting, widths 1/8/24/80,
  rich/raw modes, character-at-a-time chunks, Unicode and incomplete reflow.
  Source round trips are asserted separately from rendered output equality.
- Four real-PTY cases cover default rich output, disabled math with rich lists,
  textual lists with rich math, and raw source. These execute the real TUI against
  local streamed-response fixtures; they do not claim production gateway access.
- Scoped config/core/TUI lint has zero warnings locally and on Apple Silicon.
  Formatting passed. Final source parity matches all 7,218 tracked `codex-rs`
  files on the Mac. No tests were repeated solely for lint or formatting.
- Full GNU run 3859 / Actions 312 is pending on the exact runtime/tooling source.
  Its complete log, assertion-level baseline comparison and retry outcomes are
  required before this feature clears its gate.

## Adversarial findings

1. The display patch required upstream's partial-prose preview interface; it was
   adopted and validated in a separate prerequisite, preserving the existing
   Unicode-list setting on overlap.
2. A list PTY initially mistook the new disposable last-line preview for complete
   stable output. Its synchronization now waits for every expected line and then
   checks the simultaneous terminal screen; the product and inputs were not
   changed to hide the failure.
3. Disabling math by bypassing recognition would corrupt literal TeX through
   Markdown escape handling. The flag therefore controls conversion only, and
   disabled-output snapshots plus real terminal checks cover the distinction.
4. Source transfer via archives could invalidate compilation caches through
   unchanged timestamps. The replacement checksum sync preserves unchanged
   mtimes, refuses active Rust work, removes only previously tracked deleted
   paths and verifies source hashes. Its isolated Mac proof is retained.

## Explicit limits

Font glyph support and visual usability still need owner acceptance on the final
installed signed package. Unsupported TeX is a literal fallback, not a rendering
error or a promise of mathematical equivalence. The next fullscreen feature must
add source-aware equality coverage once `HyperlinkLine` visual equality starts
excluding source metadata, and preserve hard line breaks in spatial math copies.
No new signed/notarized binary or npm release is claimed by this source review.
