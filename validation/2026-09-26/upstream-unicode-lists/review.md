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

Scores withheld pending full GNU workspace validation. Corrected local/native suites, lint and the final source audit pass. No signed or distributed preview is claimed.


## Native diagnostic and closure

Initial native execution reports 4,761/4,764 passes, six skips, retries disabled. All three reported failures contain successful Rust assertions followed by Python cleanup errors: a Node helper was still writing `jiti` or its compile cache after its parent test returned. This is retained as an infrastructure failure, not a clean test pass. The final test runner retries only `ENOTEMPTY` cleanup races, bounded to two seconds; other errors still fail. The subsequent fresh full suite on the formatted committed source passed, as recorded below. No product behavior changed for this correction.

Local four-package lint passed with zero warnings; formatting completed and 17 unrelated preexisting Python formatting changes were restored. Full GNU run 3856 (Actions run 311) uses exact source/tooling `d07ec0743140b4fb710a5b99c436ea29862bd192`.


Native closure: the complete fresh suite passed **4,764/4,764**, six skips, retries disabled, including all three previously reported wrapper failures. The corrected wrapper cleaned successful temporary roots. The source audit matches all **7,190 tracked codex-rs files**, including schemas and snapshots (4,018 are Rust files). Final native four-package lint passed with zero warnings, and all 7,190 tracked workspace files still match after lint. Full GNU remains pending; no score is assigned yet.


## Additional terminal acceptance

A further review found the dedicated list PTY requirement was not yet covered by the renderer snapshots. Three new integration cases now stream a controlled local response through the real TUI process: default Unicode markers, disabled-list textual markers, and raw Markdown output. All three pass on Linux and Apple Silicon with retries disabled. Filtered-suite skip counts are exclusions, not additional platform skips. Fresh `codex` and `airs-harness` debug binaries built on both hosts and passed version probes; their byte hashes are retained. These are development builds, not new signed/notarized packages.

The extra tests passed scoped lint on both hosts with zero warnings, then formatting. All 7,191 tracked workspace files match the native source. Full GNU 3856 runs runtime source `d07ec07431`; these subsequent test-only additions have separate local/native receipts and are not claimed as part of that earlier full-run count.
