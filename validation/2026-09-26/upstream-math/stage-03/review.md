# Accents, symbols and named delimiters

Adapt upstream `608825d511`; no conflicts, dependencies or configuration changes. New Unicode segmentation uses the existing dependency. The parser remains under 500 lines in this stage. Accents are accepted only for a single visible grapheme; zero-width, empty, compound and multiline arguments retain literal source. Named delimiter recognition consumes bounded input and rejects unsupported delimiters rather than guessing.

Inspected the three new snapshots for accents/symbols, named delimiters and a display equation. New tests also verify following subscripts/superscripts and explicit fallback for ambiguous accent arguments. Existing source-mask and streaming behavior remain unchanged. Complete local TUI/config: 4,789 passes, six skips, no retries. Scoped lint has zero warnings and formatting passes. This remains a prerequisite inside the connected math feature, with no feature score or release claim.
