# Shared source-backed text layout

Imports the text-layout, logical-line and tab-projection modules from d7d9f2b9b5,
with the bounded URL wrapper omitted by the earlier activity prerequisite now
included. The old per-row u16 scroll offset is replaced by usize row/source
positions; terminal coordinates are narrowed only for visible rows. Copy text
retains hard breaks, omitted wrapping whitespace, source styles and graphemes.
Synthetic prefixes stay outside source highlighting. Logical rows share exact
source provenance instead of guessing which display characters are gutters.

The algorithms remain in separate production modules (text/layout, logical source,
tab stops). Their mechanically imported source is one coherent unit: splitting tab
projection from coordinate lookup would temporarily produce wrong source offsets.
Small AIRS adaptations retain the crate-local TextLayout measurement interface,
use text length for the existing cache budget, rewrap retained live content before
adding its separator, and pass the full source range for the existing prompt
highlight. Cache limits remain text-byte accounting with the documented one-large-
entry exception, not a hard bound on total allocation.

The old scrolled hyperlink/style/wide-glyph test is retained in text_legacy_tests.
Imported tests cover source hit testing, diff signs/indentation, prompt background,
question labels and secret masking, tab expansion and resize. New stress tests
exercise >65,535 physical rows and a >65,535-character line at maximum width.
The first compile exposed missing URL-policy/helper prerequisites; failure receipts
are preserved and the bounded helper is isolated in wrapping/within_width.rs.

The upstream first test's actual mouse-selection portion depends on the following
view integration and remains to be connected. Upstream's different recap product
is not imported: AIRS recap source/selection must receive an explicit regression
at that integration gate. Temporary dead-code allowances name the pending
selection and AIRS recap consumers; remove them when those consumers land.

No feature score or platform/release claim is made until connected input, native
and release acceptance pass. Full local TUI/config/features validation is running;
results, corrections and final lint/format receipts will be recorded below.

## Validation and corrections

The first full TUI/config/features run executed 4,957 tests: 4,953 passed and
four failed. Investigation corrected a real provenance omission in reasoning
cells: their dim/italic treatment now preserves the original markdown source and
hyperlinks. The AIRS diff test retains the existing heading without upstream's
additional decorative bullet. The legacy narrow-Unicode comparison used a
half-clipped wide glyph as its baseline; its replacement asserts bounded rows,
complete graphemes, exact source text, hyperlink/style preservation and all
scrolled subviews of the rendered frame. The overlay snapshot was reviewed to
confirm continuation indentation and inherited background, without lost link text.

The second full run executed 4,959 tests: 4,957 passed and two failed. Both were
closed without subsequent production changes: native Paragraph height and owned
source-layout height now have separate rendering assertions (eight versus nine
rows for the long path), and the new complete-grapheme snapshot was reviewed.
The focused closing run initially passed 173/174 pending the new reasoning-height
snapshot. That snapshot visibly retains the full path through the final `going`,
two-column continuation indentation and dim/italic styling. After its review and
acceptance, all 174 focused tests passed, with zero retries. This is full-suite
failure investigation plus focused closure, not a claim of an all-green full run.
Initial, first, second, third and diagnostic receipts are retained independently.

Stress tests now pass for 65,552 rows and a 65,552-character line at maximum
terminal width, closing the earlier u16-wrap limitation. URL-aware oversized
reflow and adjacent streaming-fragment whitespace preservation also pass.
Bazel already includes the new Rust and snapshot paths through its recursive globs.

Adversarial self-review found no remaining blocker within this source-layout
slice. This is not independent review or a 9/10 score for the incomplete fullscreen
feature. Actual mouse selection, AIRS recap provenance, mode lifecycle and platform
acceptance remain required at the connected feature gate.

Scoped lint passed in 89.38 seconds with no warnings/fixes. Required formatting
passed in 18.12 seconds; unrelated formatter-only changes were restored.
