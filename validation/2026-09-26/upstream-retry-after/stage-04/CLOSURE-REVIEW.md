# Native failure closure

Initial affected-package run: 4,956/4,972 passed, 16 failures, no retries; nextest also reported 24 skipped cases outside that executed count. Exact production code matches commit 0da5d68cf1; the native initial test still expected a display prefix corrected locally before that commit.

The first closure passed 217/221. Isolated test-process HOME/ZDOTDIR resolved shell-profile and RVM output contamination. A task-owned Python shim pointing to the installed Homebrew interpreter resolved the Apple launcher cache-write failure under the unchanged Seatbelt policy. Profiles and interpreter changes apply only to fixtures, never to AIRS product startup or the owner's account.

Three optional-MCP scenarios still expired their configured 250 ms/one-second startup deadline before the server received initialize. Tests now report the actual startup failure rather than a generic readiness timeout. Cold transport startup receives a separate 5/10-second fixture budget; catalog grace remains 50/250 ms or zero. The request stays held while a short-grace turn must complete within three seconds, below its ten-second server deadline; model tool presence/absence and zero-grace waiting/timeout remain asserted.

The mixed-tool test's total-turn 1.6-second cutoff did not directly establish concurrency. Its replacement holds a sync-tool barrier until a shell command creates a marker, then supplies the barrier's second participant. Both sync results must be successful. Serial execution cannot pass by merely being fast. The new assertion lives in a separate test module.

Final closure: 221/221, including every original failure and the complete API package. Local revised fixtures: 7/7. Three additional Mac repetitions: 7/7 each, all retries disabled. The initial fixture diagnostic compile failed on an ambiguous glob-imported assertion macro, fixed by explicit imports; no tests ran in that attempt. The local ad-hoc runner initially copied a stale JUnit file; that duplicate was identified by hash, discarded, and the runner now requires a fresh modification time.

Native lint first exhausted disk. Its failure is retained. Idle task-owned cache cleanup preserved production binaries, newest test targets, and compiled dependency libraries, reclaiming 9,553,540 KiB. Corrected ten-package native lint passed without warnings, with incremental compilation disabled to avoid duplicating large incremental caches. All 7,179 tracked Rust-workspace files match the committed source. Full GNU run 3852 remains pending. Scores are not yet assigned.
