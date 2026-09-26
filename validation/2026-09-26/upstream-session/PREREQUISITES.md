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


## Stage 3 — bounded cold resume (in progress)

Adapts `e7bbc79f482acf285e50a09a9f898aeb2ce3881c`. The reverse storage reader stops at the newest complete checkpoint; its caller prepends canonical owning-thread metadata. Missing replacement history/window information or a later rollback requires full replay. Core reconstruction uses checkpoint resume settings and the later companion records, without resurrecting pre-checkpoint context. AIRS retains its existing Guardian mode and normal turn submission API; the upstream daemon continuation API and absent source-runtime fork scanner are not imported.

The patch replaces a 200-line storage scanner and updates its existing tests. This connected change spans scanner, migration and core consumption; it must not be shipped with only the storage boundary changed. The larger diff is dominated by deletion of the old scanner/tests and the integration fixture. Separate stage-1 and stage-2 prerequisites already isolated the data representation and atomic persistence changes.

Initial local adaptation compiled after replacing unavailable upstream test helpers. The broader run passed 669/670 tests; the new cold-resume fixture did not reach its expected bounded checkpoint and remains under investigation. An additional adversarial test checks that rollback also retains metadata on a surviving checkpoint, not only that it discards a rolled-back checkpoint. Native validation is running on the synchronized source; these are diagnostic runs, not passing gates or a release claim. Scores remain unassigned.


Further adversarial checks found a real rollback interaction: full replay discarded previous-turn metadata on a surviving current checkpoint. Reconstruction now uses that checkpoint's metadata, including explicit empty values, and stops searching older turns once the surviving current baseline is known. The initial regression failed before the fix and passed in the subsequent broader run (670/671; remaining failure was collaboration-mode restoration).

The cold-resume integration also revealed upstream prerequisite `91d54f1667`: saved Plan mode and developer instructions were reset to Default. Core restoration is isolated in `resume_settings.rs`, keeps current model/reasoning overrides, ignores foreign owning-thread snapshots and does not inherit a parent mode into a fresh fork. App-server/TUI propagation is the next required connected slice; no core-only score will claim complete user-visible mode restoration. The initial Mac diagnostic run matched Linux at 669/670 before these later fixes; it is not validation of the latest source.


## Stage 4 — restored mode through app-server/TUI (in progress)

Adapts the remaining `91d54f1667` surface: optional `collaborationMode` in resume responses, propagation to the TUI, first resumed prompt preservation, and server-provided mode taking precedence over recovered local draft state. Older servers that omit the field keep local mode selection. The absent upstream realtime replay path is not imported; the mode application stays inside AIRS's existing replay/queue ownership path.

The prior expanded local run passed 701/703. The new cold-resume fixture reached all behavioral assertions and only needed its first reviewed snapshot; that snapshot preserves gateway-owned inference, the original Plan instructions, checkpoint summary, subsequent assistant output and the new input. The other failure exposed a test fixture's implicit model change on restart: its hardcoded collaboration model differed from the builder default, legitimately creating another world-state instruction fragment. The fixture now uses its actual initial model and asserts restored Plan mode explicitly. It still requires exactly one instruction occurrence. A separate regression verifies that explicit current model/reasoning overrides win over saved values.

Both schema generators and connected package tests are running. No final session score, latest-source native pass, workspace pass or signed package is claimed yet. The separately pinned Python SDK runtime/generator is outside this harness release; authoritative Rust/TypeScript/JSON schema fixtures are regenerated here.
