# Session recovery integration — in progress

Stage 1 adapts `7f0ab95827`: history exposes the latest opaque compaction together with its recorded producer hash. Structural usability and reviewer compatibility are separate checks. Invalid newest checkpoints remain selected, so consumers cannot silently substitute an older grant. Missing metadata is not inferred from the active model.

Existing sync Guardian and asynchronous parent-compaction consumers share this view. Their legacy omission versus strict rejection behavior is retained, including serialized-size limits. The fork's immutable Guardian context-mode selection remains unchanged; upstream mode-migration methods absent from AIRS are not imported. The extension trait's fallback reports the latest checkpoint with unknown provenance rather than claiming a producer.

History tests cover absent/empty content, missing IDs, missing/empty/mismatched producer metadata and matching reviewer hashes. Existing snapshot immutability, rollback provenance and parent-compaction size/eligibility tests are updated to the shared view. Focused local and native checks each passed 169 selected tests, including full history and Guardian-v2 package suites and selected core consumers. Scoped lint passed on both platforms without warnings. Formatting passed; unrelated formatter churn was restored. This remains an unscored prerequisite, not completed session recovery.

Next: persisted resume metadata and bounded cold-replay reconstruction, preserving AIRS owning-thread config/model checkpoint order. Do not import managed-daemon automatic continuation solely to satisfy an upstream field. Full session-feature coverage and workspace validation are required before scores; exact signed Mac delivery remains a later gate.

## Stage 2: persisted resume metadata

Adapts `286d4ecf44` to the existing AIRS lifecycle. Optional metadata distinguishes a legacy record with absent metadata from a record explicitly storing empty values. Previous-turn settings are shared with history and captured with runtime and token-usage state under the existing compaction persistence/state locks. The owning thread's gateway settings event remains after the checkpoint.

AIRS does not have managed-daemon automatic continuation. Its optional upstream turn-identity field remains wire-compatible, but AIRS writes `None` and strips inherited parent identity on fork. No continuation workflow is enabled. Child checkpoints use the child's runtime; forks that rebuild context clear inherited previous-turn settings. Existing Guardian authorization sanitization remains intact.

The local broadened suite passed 649 cases, including history/rollout/thread-store packages and core compaction/gateway cases. Both schema generation modes passed, and 302 protocol tests passed. One generator fixture is ignored during ordinary tests and was explicitly run by regeneration. The first compile attempt found an ambiguous test macro import and one omitted optional field in an old fixture; both were corrected before the passing run. Native connected-feature validation is deferred until the reader integration is complete, to avoid duplicating the same transitive native rebuild; no native stage-2 result is claimed.

Stage 3 must preserve AIRS legacy rollback behavior: a post-checkpoint rollback marker prevents both scanner and core reconstruction from taking the bounded shortcut. Incomplete newest checkpoints also require full replay. Ordinary valid checkpoints should bound reconstruction without restoring stale pre-checkpoint settings. Full session scores remain unassigned.
