# Unicode lists — validation in progress

Adapt upstream `5a11c456060c2ce131de1ba195fc7729f7d7db66`, with the applicable shared rendering-preference interface from `a86631502d`. This fork has no Mermaid renderer or upstream ListSpacing interface; neither is introduced merely to satisfy neighboring upstream contexts.

## Acceptance and counterexamples

- Render unordered markers and valid GFM task markers as Unicode, preserving ordered numbering and hanging indent.
- Match streaming and completed output, including character-by-character partial markers, Unicode and widths 1/8/24/80.
- Preserve raw source and raw-mode output; keep code, escaped markers and ordinary bracket text intact.
- Support `tui.rendering.lists = false`, independently of animation/whimsy; seed before startup previews and preserve the client setting across widget replacement.
- Keep existing gateway, authentication, tool and routing behavior.
- Generate the real config schema and exercise full workspace compatibility because resolved core configuration gains a field.

## Review observations

The initial 4,758-test local run had 31 failures. All 26 assertion failures were expected bullet changes; three existing snapshots changed only rich marker presentation; two new snapshots required acceptance. Every difference was inspected. Raw portions of the transcript-toggle snapshot remain unchanged. A bare `- [x]` without trailing whitespace/content remains textual under the parser's GFM rules; valid markers preceding empty/block content are recovered without duplicating marker events.

The implementation reuses existing renderer style/indent owners. Parser recovery is bounded to the current item prefix. Tests cover nested/ordered/quoted/wrapped items, HTML/block starts, code and escapes. No provider calls, new dependencies, transcript rewriting, fullscreen default changes, or credential access are added.

## Gate

Scores withheld pending corrected local suite, native suite, lint, full workspace validation and source audit. No signed or distributed preview is claimed.
