# Compact MCP activity — fullscreen prerequisite

Adapted the MCP portion of upstream 321c50fc2c with stable activity IDs, compact tail previews, retained detail sources and original text/JSON in transcript/raw views. AIRS credential-helper state and separate validated image cells remain intact; computer-use products are not introduced. Both preview paths share a title normalizer capped at 1024 UTF-8 bytes before allocation and 80 graphemes, while full invocation arguments remain available in details.

First run:241/245 passed. Four snapshot differences were inspected: existing AIRS shortcut spacing, clipping at narrow width, and existing separate image result representation. Expected snapshots were adapted. Added large Unicode, combining-character and whitespace title coverage with narrow viewports and retained raw arguments. Final:246/246 passed, zero retries. Scoped lint passed without warnings/fixes; actual formatting passed. Initial failures remain retained.

This is a source prerequisite, not a completed feature or release score. The connected fullscreen/selection gate remains pending, including explicit collapsed-preview source metadata to exclude decorative gutters without disclosing hidden output. Native/GNU and signed distribution checks follow the composed feature.
