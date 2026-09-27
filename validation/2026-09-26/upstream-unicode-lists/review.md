# Unicode lists — completed source integration

| Criterion | Score | Evidence |
| --- | --- | --- |
| Implementation | 9/10 | Shared streamed/completed rendering, client-owned configuration, nested/ordered/quoted/wrapped task lists and real terminal checks pass. |
| Code quality | 9/10 | Small private helper modules, zero-warning local/native four-package lint, generated schema, formatting and 7,191 matching tracked workspace files. |
| Design | 9/10 | Applicable upstream preference interface; no inert Mermaid/table fields, extra dependencies, transcript rewriting or authentication changes. Startup and replacement widgets retain the client's choice. |
| Feature completeness | 9/10 | Local/native affected suites, three PTY cases per host, real debug builds/version probes, full GNU baseline comparison and documented textual fallback. Signed distribution remains in the delivery phase. |

These are bounded engineering self-review scores backed by executable evidence, not independent certification or completion of the broader PRD.

## Behavior

Adapt upstream `5a11c45606`, using the applicable preference interface from `a86631502d`. Default rendering uses Unicode bullets and task checkboxes. `[tui.rendering] lists = false` retains textual list/task markers independently of animations and other inline Markdown styles. Raw output remains unchanged. Ordered numbering, hanging indent, code/escaped text and block/HTML starts are covered.

The current fork lacks upstream ListSpacing and Mermaid interfaces; neither is imported merely to satisfy adjacent patch context. Core configuration gains one resolved preference field, with the real generated schema. The current owner's local setting is seeded before startup previews and restored on widget replacement, even if legacy/server configuration differs. There are no new provider, credential, gateway, model-routing or fullscreen-default behaviors.

## Verified evidence

- Local full TUI/config: **4,758/4,758**, six skips, retries disabled.
- Apple Silicon full TUI/config: **4,764/4,764**, six skips, retries disabled.
- Three additional PTY cases per host pass without retries: Unicode defaults, disabled-list textual markers, and raw Markdown. Each drives a local streamed response through the real TUI process. Filtered-suite exclusions are not platform skip counts.
- Character-by-character streaming matches completed/emitted output for both list settings, rich/raw modes and widths 1/8/24/80, including Unicode. The source round-trip fixture is newline-terminated; no new claim is made about preexisting unterminated-input normalization.
- All five snapshot changes were reviewed. Existing raw transcript sections are identical; only rich marker presentation changes.
- Fresh `codex` and `airs-harness` debug builds on Linux and Apple Silicon pass all four version probes. Byte hashes are retained. These are development artifacts, not a new signed/notarized npm release.
- Local/native four-package lint has zero warnings, including the additional PTY tests. Formatting passed; 17 unrelated preexisting Python formatting changes were restored. All **7,191 tracked codex-rs files** match the Mac after lint/format.
- Full GNU run **3856 / Actions 311**, exact runtime/tooling `d07ec07431`: **18,612 passed, three failed, 34 skipped**, 2,027.984 seconds, no retry-pass flakes. Every assertion from both attempts of each failure matches the retained stable baseline after only timestamp/ANSI/indent/PID/fixture-path normalization. The suite is explicitly **not all green**.
- PTY test commit `3766c4dab8` follows the full-run source and changes only test code. Its separate local/native receipts are not folded into the earlier full-workspace count.

## Adversarial findings and closure

1. Initial local execution had 31 failures: 26 old marker assertions, three existing rich snapshots and two new snapshots. Each difference was inspected; the corrected full run passes. No source Markdown input was changed to manufacture the expected output.
2. The first native run reported three failures after their Rust assertions passed. Node helpers were still writing compile caches during temporary-root cleanup. The validation wrapper now retries only `ENOTEMPTY` for at most two seconds; other errors remain failures. The fresh complete native rerun passes. No product behavior changed for this correction.
3. Review caught a missing dedicated terminal acceptance check. The three PTY cases now cover actual terminal output, complementing renderer snapshots and streaming tests.
4. Review corrected an earlier audit label: the preceding terminal feature's 7,182-file count covered all tracked `codex-rs` files, not exclusively `.rs` files. Its receipts/count remain valid; the prose and field name are corrected.

A bare `- [x]` without trailing whitespace/content remains textual under the parser's GFM rules. Valid markers before empty/block content are recovered without duplicate marker events. No new dependencies or Cargo/Bazel lock changes are required for this slice.

## Remaining boundary

The list feature clears its four 9/10 source-integration gates. Math, connected fullscreen/search/mouse behavior, final signed/notarized Mac packaging, fresh registry acceptance and owner visual/real-account testing remain in the broader PRD. Linux distribution still waits for owner Mac acceptance.
