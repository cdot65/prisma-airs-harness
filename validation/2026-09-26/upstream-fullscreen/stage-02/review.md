# Logical source propagation — validated prerequisite

Completes applicable source propagation from `fcaa035438` through prefixed
history and user prompts, while preserving AIRS' existing URL annotation,
forced URL wrapping, prompt colors and sanitization. Copy metadata excludes
gutters/right padding, retains exact whitespace and keeps identical repeated
hard lines as separate source identities. Blank rendered rows retain their
original source but expose an empty visible range.

The math integration has a pending-TeX preview path absent from the upstream
source patch. It now uses source-aware wrapping independently for each hard
line, with the same textwrap algorithm preference and 8 KiB preview bound.
Soft fragments share source; hard lines remain distinct. The original stream
source and finalization contract remain unchanged. Rich spatial equation rows
retain separate identities, preventing later copy logic from treating a
numerator, rule and denominator as one soft-wrapped line.

New tests exercise forced URL wrapping and repeated prompt lines at widths
8/24/80, unfinished TeX whitespace and repeated hard lines, equation row
identities, and existing list/math combinations with explicit source-aware
equality in addition to visual equality. Existing raw/source round-trip
assertions remain. Actual selection-to-clipboard checks follow with the
connected selection implementation.

Local history/streaming/Markdown subset: **472/472**, retries disabled. Filter
exclusions are not platform skips. All existing snapshots in that subset pass
without changes. Scoped config/core/TUI lint passes with zero warnings and no
fixes. Formatting passes; only unrelated preexisting Python churn is restored.
No dependency/schema change is needed. Native/fullscreen and signed delivery
gates are pending; this prerequisite receives no standalone feature score.
