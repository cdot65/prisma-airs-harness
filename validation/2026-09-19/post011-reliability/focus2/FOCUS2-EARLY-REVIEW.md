# Focus 2 early independent review

Status: read-only design/code review; focused Rust compilation and installed acceptance are still pending. No completion score awarded.
Reviewed worktree: `/home/cdot/development/cdot65/airs-post011-reliability-20260919` after PRD1 commit `873c42bfab`, with uncommitted diagnostic-report changes.
Reviewer did not modify product code, run Rust, access credentials, use a real clipboard or alter publication tags.

## Assessment

No demonstrated runtime privacy or state-preservation blocker found in the current diff. The design is small, deterministic and isolated to AIRS-owned TUI modules. Two meaningful verification gaps should be closed before a 9/10 completion decision, and final tests/snapshots/docs are still required.

## Findings requiring attention

### R1 — No-action assertion is vacuous after draining events

`codex-rs/tui/src/chatwidget/tests/airs_doctor.rs`, new `report_copy_failure_keeps_lease_draft_and_local_fallback`, calls `drain_insert_history(&mut rx)` and then loops over the receiver to reject `CodexOp`, `AirsDoctor` and MCP events. The helper in `chatwidget/tests/helpers.rs:347` drains **every** event and discards non-history events. Consequently the latter loop can never detect an accidentally emitted model/network action.

Replace this with one receiver drain asserting absence of prohibited operation variants and `InsertHistoryCell`, or collect once then inspect every event. The related new actions/status snapshot test also only checks history and should use the same honest event inspection when claiming no model/operation effects. This is a test defect, not evidence that current production handlers submit anything.

### R2 — Event-guard wiring has no behavioral integration coverage yet

`report_tests.rs` proves `Session::allows` and view-drop invalidation, but no test exercises `App::handle_airs_doctor` rejecting a stale/wrong-thread `Event::Report`. If the actual guard at `app/airs_doctor.rs` were removed, all these lifetime tests would still pass. The installed new test covers only a valid current-view path.

Add one bounded App-level behavioral case (reuse existing App fixtures) that sends a Save action from an invalidated/replaced view or another thread and verifies no report file is created. A valid current-thread/current-view Save can serve as its positive control. This avoids a real clipboard and verifies the capability is actually enforced at the side-effect boundary. If existing App construction makes this disproportionate, explicitly retain the wiring coverage gap instead of claiming the thread-switch acceptance row executed.

## Positive code findings

- `report::render` accepts only known authentication labels and fixed check names/booleans. It never renders check details, report product, gateway/environment/path values or connection metadata. Duplicate known checks consolidate pessimistically. Unknown checks cannot inject labels. The final text is structurally bounded by ten static checks; the 8192-byte assertion is therefore adequate for the current structure, with an explicit save-time limit as a second boundary.
- `gateway_access` absent is `Not verified`. Gateway health, saved credentials and completed tool use remain distinct. Credential-service failure explicitly does not establish a locked collection.
- Safe text is captured in `Arc<str>` once per inspection. Preview/copy/save consume that snapshot. The report action branch contains no doctor subprocess, new inference/MCP request, credentials access, configuration write or model/history submission.
- Each view header owns a Session capability and invalidates it on Drop. Report actions use `dismiss_on_select: false`, preventing their own view from expiring before dispatch. App checks matching displayed thread, active capability and the doctor view before any side effect. Replacing report actions invalidates previously queued actions. Preview preserves the underlying view and therefore supports returning with Esc.
- `Event` Debug remains payload-redacted. Preview uses literal text lines through the existing pager, not markdown/link expansion. Saved path appears locally in a private popup, not in the shareable report or transcript.
- Clipboard tests inject success/failure closures. Existing clipboard leases are replaced on successful copy and preserved on failure. User-facing failure text excludes backend errors. The success message acknowledges terminal clipboard support may prevent actual acceptance; it does not claim observed native paste success.
- The installed PTY case forces SSH/OSC52 with tmux disabled, decodes the exact emitted bytes and compares them to the saved file. It verifies unchanged health/inference/MCP request counts, config/binding preservation and no report in rollout files. This is meaningful end-to-end coverage once executed successfully.

## File creation / cleanup review

`report::save` rejects oversized text before writing, creates an unpredictable file in the selected home, writes/flushed bounded text, then keeps the completed file. The installed tempfile3.27.0 implementation was inspected:

- `file/imp/unix.rs:18` uses `create_new(true)` and mode0600; existing filename/symlink targets cannot be overwritten.
- Failed write or flush drops the NamedTempFile, retaining normal unlink-on-drop cleanup.
- `NamedTempFile::keep` returns an owning PersistError on failure; mapping/dropping that error also drops its NamedTempFile.
- Generated filenames contain neither the local environment name nor server identifiers.

Current tests prove unique saves, actual file mode, exact contents, preserved sentinel, invalid destination and pre-write size rejection. They do **not** inject a partial-write failure or force a random-name collision. Those properties currently rest on reviewed tempfile semantics, not executed AIRS fault injection. Do not describe the tests as covering those exact failure modes. No new production abstraction is necessary merely to retest an unchanged tempfile primitive.

## Remaining completion gates

1. Resolve R1; preferably add R2's behavioral dispatch case.
2. Complete focused TUI tests and inspect generated snapshot diffs, especially 55-column disclosure/action usability and pager return behavior.
3. Execute the installed PTY report case and existing doctor cases. Preserve exact no-extra-request counts and byte equality evidence.
4. Keep clipboard failure explicitly unit-injected and SSH copy actually terminal-mediated; native clipboard UI acceptance remains the later native release check, not proven by a fake lease.
5. Update the matched user guide with privacy limitations and explicit local save location/behavior; no automatic uploads.
6. Preserve npm `latest` at0.1.1 until a separately authorized stable promotion. This diff contains no tag/version mutations.

No Rust tests were run by this reviewer. `git diff --check` passed for the inspected working diff.
