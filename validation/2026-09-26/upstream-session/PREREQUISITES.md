# Session recovery integration — in progress

Stage 1 adapts `7f0ab95827`: history exposes the latest opaque compaction together with its recorded producer hash. Structural usability and reviewer compatibility are separate checks. Invalid newest checkpoints remain selected, so consumers cannot silently substitute an older grant. Missing metadata is not inferred from the active model.

Existing sync Guardian and asynchronous parent-compaction consumers share this view. Their legacy omission versus strict rejection behavior is retained, including serialized-size limits. The fork's immutable Guardian context-mode selection remains unchanged; upstream mode-migration methods absent from AIRS are not imported. The extension trait's fallback reports the latest checkpoint with unknown provenance rather than claiming a producer.

History tests cover absent/empty content, missing IDs, missing/empty/mismatched producer metadata and matching reviewer hashes. Existing snapshot immutability, rollback provenance and parent-compaction size/eligibility tests are updated to the shared view. Focused local and native checks each passed 169 selected tests, including full history and Guardian-v2 package suites and selected core consumers. Scoped lint passed on both platforms without warnings. Formatting passed; unrelated formatter churn was restored. This remains an unscored prerequisite, not completed session recovery.

Next: persisted resume metadata and bounded cold-replay reconstruction, preserving AIRS owning-thread config/model checkpoint order. Do not import managed-daemon automatic continuation solely to satisfy an upstream field. Full session-feature coverage and workspace validation are required before scores; exact signed Mac delivery remains a later gate.
