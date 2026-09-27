# Source-aware diffs and patch details — fullscreen prerequisite

Adapted diff and patch portions of321c50fc2c. Diff rows retain logical source ranges and change signs while excluding number gutters, wrap at grapheme boundaries, and carry hard-wrap metadata. Compact patch headings retain visible path/count text without decorative bullets/tree gutters. Detailed patch output remains complete. Patch failures now retain complete diagnostics for transcript/raw display.

AIRS's existing syntax colors and deletion DIM overlay are preserved; the upstream foreground redesign was not part of this feature. The existing syntax-wrap snapshot was retained instead of accepting its upstream deletion. Most changes in the large diff module mechanically propagate HyperlinkLine and consolidate existing style arguments; keeping that structure avoids a separate unrelated renderer refactor. Tests and evidence are separate from production.

Review corrected the imported compact failure renderer: it no longer renders the entire error before truncating. It bounds selected input to16KiB before ANSI processing, shares a three-row preview, and retains only visible text in compact copy sources. This provides a real consumer of the source-aware preview API and removes its temporary dead-code allowance.

Initial323 checks passed; two added regression cases gave324/325 because the new heading expectation allowed one extra character. The reviewed expected clipping was corrected; final325/325 passed, zero retries. Existing diff/style snapshots and new compact/full patch snapshots pass. Tests cover CJK/combining/emoji, change signs versus gutters, long heading clipping, hidden fourth lines, genuine gutter-like diagnostics and dense combining input. Initial failed evidence remains retained.

This remains an intermediate source gate, with no standalone fullscreen score. Connected selection, replay, viewport and native/distribution validation remain pending. Scoped lint and formatting receipts accompany this review.
