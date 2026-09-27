# Transcript overlay extraction

Applied1ac4b6973b as a mechanical extraction: the detailed overlay moves from pager_overlay.rs into a504-line module, with its tests beside it. The implementation block comparison against AIRS differs only in one explanatory comment; public callers and runtime behavior are preserved. Existing highlight tests move with the owning module. Source/snapshot files are covered by existing Bazel globs.

The function-name audit found one prior footer-hint regression absent from the upstream extraction. It was restored in the new test module. Initial159/159 and final160/160 focused pager/transcript/recap checks passed with zero retries. Snapshots remain visually identical. This intentionally exceeds the normal patch-size target because it moves the existing implementation and tests rather than adding that volume of logic.

Scoped lint and formatting receipts accompany this prerequisite. No standalone fullscreen score or distribution claim is made; cache/anchor/selection/search and connected validation are still required.
