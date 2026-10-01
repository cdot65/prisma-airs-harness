# Nested writable metadata mount ordering

Adapted upstream 8bbe8f8702471b90c75df20104bf91d20001b0d7 (#47623) on source f42eec46e1. A parent writable bind previously mounted protected child metadata before a deeper writable bind, creating duplicate metadata mounts and preventing some sandbox startups. Defer only shared mounts until the deeper writable bind; retain validation and symlink remapping for every path before deferring it. Preserve AIRS synthetic mount registry protections. No dependency, schema, routing, identity or OAuth changes.

The extended mount-order regression fails on original production code (duplicate metadata mounts) and passes with the fix. The four unchanged upstream applied-policy cases exercise directory and symlink roots from parent and nested working directories. They write allowed files, prove protected metadata remains unwritable, and check temporary mountpoint cleanup. No new case returned early for unavailable sandbox prerequisites.

Linux sandbox unit suite: 111 passes. Four focused applied-policy cases: four passes; 41 unrelated tests filtered out. Full owning crate: 155 passes, one explicit skip, no retry-only passes. Existing process-reaper cases printed four early-return notices for unavailable namespaced proc mounts; these are not acceptance of those cases. The separate sandboxing baseline passed 69 cases.

A preexisting codex-client redundant closure blocked Clippy. Replace it with the identical RetryAfter method reference; this is a separate one-line lint correction with no retry timing change. Both standard and strict scoped Clippy subsequently remain blocked by the previously documented static product-header expect in rmcp-client/src/utils.rs:18. Preserve the strict log and do not claim a passing lint gate. Standard scoped lint status is recorded separately in ACCEPTANCE.json.

The repository formatter ran successfully. Its unrelated Python/evidence rewrites were restored, preserving historical release evidence. Per repository guidance, tests were not rerun after final lint/format cleanup.

This is source-level Linux evidence, not exact-package or cross-platform release acceptance. Installed-binary upgrade/rollback, native-host/platform release checks and the remaining adoption slices are pending. No release readiness score or publication is claimed.
