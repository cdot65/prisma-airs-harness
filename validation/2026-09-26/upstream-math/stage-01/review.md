# Inline math prerequisite — connected math gate still open

Adapt `ad8a5e3a1b`. Preserve the already validated Unicode-list parser option and shared preferences when merging the overlapping renderer changes. No new dependency or configuration field is introduced in this prerequisite.

Local full TUI/config suite passes 4,767/4,767, six skips, no retries. Both new inline snapshots were inspected. Tests cover unsupported and malformed TeX, byte/depth/display-width limits, code/link/shell/currency exclusions, citation paths containing dollar signs, and streamed display-delimiter boundaries at multiple widths. No provider/authentication/model-routing change is made.

Review: recognition walks UTF-8 boundaries, preserves byte offsets by masking only a rendering copy, and restores unsupported expressions. Conversion does not execute TeX or external programs. Protected Markdown contexts and display delimiters remain outside inline recognition. Fraction parentheses avoid changing expression scope. This is a deliberately bounded subset, not a general TeX renderer.

Scoped TUI lint and repository formatting passed; lint emitted no warnings. Unrelated Python formatter changes were restored. Native connected coverage, standalone display layout, accents/aligned equations, the math preference and real terminal acceptance belong to subsequent steps of the same math feature. No math feature score or signed distribution is claimed at this prerequisite.
