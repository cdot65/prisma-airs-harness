# Session recovery adversarial review — feature gate passed

This is a self-review with reproducible tests, not an independent certification. The bounded session feature meets all four 9/10 engineering gates. Exact signed-artifact acceptance remains a later delivery gate; this is not a release-completion claim.

| Dimension | Score | Evidence and limits |
| --- | --- | --- |
| Implementation | 9/10 | Atomic metadata, bounded/full reconstruction, surviving and discarded rollback checkpoints, actual resumed gateway requests, first resumed prompt and draft reconnect pass. |
| Code quality | 9/10 | Shared metadata and narrow helpers, generated schemas, reviewed snapshots, clean scoped lint on both platforms and formatting; 7,173 tracked Rust-workspace files match native source. |
| Design | 9/10 | Canonical ownership, absent/empty distinction, stripped parent continuation identity, current route/model/effort authority, read-only quiet resume and old-server fallback are preserved. |
| Feature completeness | 9/10 | Storage/core/app-server/TUI coverage is connected; full GNU has 18,548 passes and only three exactly matched baseline failures. Native failures/flakes are closed by 175 no-retry case executions. Signed package acceptance remains pending. |

## Challenges that changed the implementation

- Upstream's simplified storage scanner omitted legacy rollback handling that AIRS still needs. Both storage selection and core bounded replay now reject a shortcut across a newer rollback marker.
- Full replay initially lost previous-turn settings on a surviving current checkpoint after rollback. A failing regression reproduced the loss. Reconstruction now honors that checkpoint's baseline, including intentionally empty values, before consulting older turns.
- A genuine cold-resume integration reached the new checkpoint but lost Plan mode. The missing `91d54f1667` prerequisite is adapted end-to-end. Saved mode/instructions do not override current model/reasoning or permission settings; only the owning thread's snapshot wins.
- The integration initially used an OpenAI-authenticated test provider by accident. Its gateway fixture now matches existing AIRS test conventions and fails immediately on inference errors.
- An old collaboration test implicitly changed models on restart because its hardcoded model differed from the builder default. It now uses the actual initial model and still asserts exactly one instruction occurrence; a separate regression validates deliberate current-model overrides.

## Boundaries retained

No managed-daemon continuation, voice activation, remote exec-server defaults, direct upstream MCP route, new authentication service, hosted account fallback or sandbox-environment bypass is introduced. Existing Guardian context-mode selection and checkpoint provenance remain intact. The gateway owns inference and upstream MCP OAuth; the new history metadata contains no credential material. Canonical thread metadata is reattached by the storage owner rather than copied from a parent segment.

A signed/notarized npm Mac preview is still a later delivery gate. No package or latest-source full-workspace pass is claimed by this review. The Python SDK remains tied to its separately pinned runtime; this harness change regenerates its authoritative Rust/TypeScript/JSON app-server schema fixtures.


## Rejected prerequisite behavior

A diagnostic adoption of the resume-time checkpoint write from `6515a72db7` failed four existing AIRS lifecycle assertions: quiet resume must not rewrite rollout timestamps or replace persisted cwd with a transient override. That write was removed. The mode response still restores saved Plan instructions and applies current model/effort overrides, while the accepted persisted snapshot remains unchanged until a turn or explicit settings update. Tests assert both effective response values and original durable values. The broader workspace-root migration is not imported merely to satisfy an upstream test expectation.

The connected initial Linux run passed 5,633/5,636; one new reviewed TUI snapshot and two adapted assertions remained. The diagnostic checkpoint write then passed 5,632/5,636 and demonstrated the four incompatibilities above; it is not retained. Native CLI executables built, but native test compilation exhausted disk. The failed run is retained. Only stale changed-package test executable outputs and subsequently idle task incremental caches were reclaimed; all compiled library dependencies, production binaries and user files were retained. Native validation must finish before any score.

## Compatibility closure

The retained production source matches the initial connected run (5,633 passes) byte for byte. Its three remaining failures were two imported fixture assumptions and the first new TUI golden. After adapting those expectations to AIRS and accepting the reviewed golden, all 75 targeted resume/reconnect tests passed in 17.305 seconds. These include the initial three failures and the four lifecycle checks that rejected the temporary persistence write. This is a targeted closure, not a claim that the entire workspace passed. Connected lint, latest-source native validation and full GNU workspace comparison remain required.

The native diagnostic still in flight contains the rejected persistence write. Preserve its results as diagnostics and synchronize the final source before counting native acceptance. Neither that diagnostic nor the existing transport workspace receipt can satisfy this feature's gate.

Scoped lint completed in 6m57s without warnings. Formatting and `git diff --check` passed; 17 unrelated formatter changes were restored. No Rust dependency changed. The patch is committed in two review slices: storage/core reconstruction and mode restoration, followed by app-server/TUI propagation with generated protocol fixtures. Native and full-workspace gates remain pending.

## Native timeout investigation

The rejected-write diagnostic completed 5,641 cases: 5,635 passed, the expected four lifecycle assertions failed, and two TUI tests timed out on both attempts. The prior transport receipt shows `background_task_reads_server_defaults_for_actual_destination` taking 58.884 seconds against the unchanged 60-second deadline. `unavailable_thread_new_and_clear_start_a_writable_session` passed in 29.503 seconds only after a failed attempt. These are not treated as clean baseline passes.

Both tests aggregate independent server scenarios. Their nine destination scenarios and twelve recovery combinations are now isolated into named tests using local macros and the existing Tokio harness. Source comparison proves the scenario bodies and assertions are unchanged. No deadline increase, skipped scenario or new dependency is introduced. An initial compile attempt used an unavailable parameterization helper; the local macro implementation replaces it. Targeted validation runs with retries disabled, followed by native confirmation. Production session code is unchanged by this test correction.

All 21 isolated cases passed locally with retries disabled (1.288 seconds). Scoped lint passed without warnings (3m18s); formatting passed with unrelated churn restored. Native validation is pending after the current production-source suite finishes.

## Broader native closure still in progress

The committed production-source native suite passed 5,638/5,641 cases in 785.768 seconds. The only final failures were aggregate deadline overruns (background destinations, replacement defaults, unavailable-thread recovery). Startup precedence and MCP registration passed on retry. All four quiet-resume regressions passed. Seven replacement and six startup scenarios are now also isolated, with every body/assertion retained; those 13 plus the unchanged MCP registration fixture passed locally without retries. Lint and formatting passed.

The first 21 isolated native cases passed five consecutive repetitions without retries (105 passes). Subsequent lint failed because the disk filled. Investigation found `chatwidget/tests/helpers.rs::test_config` deliberately retains each temporary home with `TempDir::keep()`. Cleanup removed 10,832 closed fixture directories created in this task's time window before the latest broad run, excluding open files; the receipt is retained. Current binaries and validation evidence remain. Future task-native checks use a private temporary root and clean it after successful runs. Native validation of the additional cases and lint remain outstanding; no score is assigned yet.

## MCP fixture startup bound

The first controlled 14-case native run passed three repetitions, then reproduced the MCP registration failure on repetition four. Its five-second event deadline included complete child configuration/state-database startup. The existing shared `core_test_support::startup::STARTUP_TIMEOUT` is 30 seconds expressly for loaded startup fixtures. This one test now uses that shared bound; production limits and the outer 60-second test limit are unchanged. Registration must still arrive before creation completes, and approval acknowledgment is still required. This is distinct from the scenario splits, whose deadlines were not changed.

The corrected native run passed five repetitions of all 14 cases without retries (70 passes). The local MCP case, lint and formatting passed. Tests run in an owned temporary root that is cleared after each successful repetition. Native lint is still required before assigning the connected feature scores.

## Connected gate decision

Full GNU run 3851 validated production source `639fda5722`: 18,548 passes, three failures and 34 skips in 1,994.583 seconds. Every assertion from both attempts of all three disabled exec-server failures matches the stable baseline. There were no additional retry passes. The suite is explicitly not all green. Subsequent Rust changes are limited to five test modules; production code is unchanged.

The original native suite's three failures and two flaky cases all map to the final repeated case sets: nine background destinations, twelve unavailable-session recovery combinations, seven replacement configurations, six startup precedence cases, and the MCP registration fixture. Each final case passed five times with retries disabled (175 executions). Native scoped lint passed without warnings in 5m18s. The full tracked `codex-rs` source audit matches all 7,173 files after two formatting-only syncs; those changes do not require another test run.

Implementation, code quality, design and bounded feature completeness each score 9/10. Retain the baseline failures, diagnostic attempts, fixture deadline correction and temporary-home cleanup receipts. No signed package, new npm version, whole-goal completion, live company sign-in or owner acceptance is implied by this gate. Retry-After is the next separate feature.
