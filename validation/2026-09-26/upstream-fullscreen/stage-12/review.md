# Explicit compact copy sources and plan activity

Adversarial review identified an upstream selection boundary: clipped compact rows lost their source metadata and could copy decorative gutters. The shared prefix-aware clipper now records only visible content, including visible ellipses and meaningful status labels; callers supply their decorative prefix explicitly. It does not infer prefixes from model/tool text or expose clipped content. Command and MCP previews preserve their exact prior rendering. Regression coverage spans widths0–39, CJK/combining/emoji, real gutter-like output, styles, status labels and a real MCP compact/full-output boundary.

Adapted compact plans from321c50fc2c with stable identities, current/pending/completed priority, three-row previews, expanded source-aware notes/steps, and checkboxes retained as meaningful copy labels. Existing AIRS cyan styling remains. Source order and full explanation remain in detailed/raw history. Narrow widths and a new visible snapshot are covered.

The copy-only checkpoint passed249 focused checks; the composed plan/copy stage passed250, zero retries. Scoped lint passed without warnings/fixes; formatting passed. Connected selection tests and patch-copy coverage remain required before scoring the full feature. No standalone fullscreen score or release claim is made here.
