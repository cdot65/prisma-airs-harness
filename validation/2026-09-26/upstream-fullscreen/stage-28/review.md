# Owned fullscreen transcript integration — local closure

Adapts f1feda2299 and the final a2de8fedcc preference to AIRS. Fullscreen stays
opt-in. Main history and live output share source-anchored selection above the
composer, with return-to-latest feedback, prompt details, source-preserving reflow,
older-page handling, and clipboard outcomes. Modal/overlay ownership wins input.
Session transitions and local reloads preserve actual launch screen restrictions.
CLI startup futures are boxed. No gateway authentication or routing is changed.

This coordinated upstream application/UI import exceeds the normal patch-size
limit because main view, bottom-pane footer borrowing, frame ownership, history
mutation, and input routing must compile together. New logic lives in separate
owned_transcript, activity_presentation and transcript-view modules. Existing AIRS
large orchestration files receive call-site wiring rather than whole-file imports.

AIRS adaptations preserve boxed overlays, existing Ctrl-L regression, original
image-cell display, key-label spacing, contrast-aware accent styling, pending token
activity cells, and the existing archived-session confirmation interface. Missing
upstream daemon, voice, command-center and Computer/CUA features are not imported.
The inert Usage overlay marker is removed. A persisted-history test fixture is
extracted without importing completion-timestamp product behavior.

Startup resolves local bootstrap settings using the existing environment-aware cwd,
selected profile, normal loader and CLI overrides before acquiring/drawing the
terminal. The bootstrap result is reused. The prior local daemon socket probe is
already timeout-bounded; no remote session or cloud fetch is added before paint.
After local presentation is known, the provisional composer remains responsive
through cloud/session startup. Terminal.app-over-SSH and no-alt-screen restrictions
are applied before the first frame, with entry inside synchronized rendering.
Native behavior still requires validation; these are implementation statements,
not claims that all acceptance gates have passed.

Initial compile receipts retain missing fork-specific styles/header/API fixture
errors. The second compile retained fixture API/lifetime errors. The first executing
focused run passed 133/136 with zero retries. All three failures were reviewed
upstream snapshots: AIRS key spacing, existing image-cell rendering, and cyan accent
styles. Their actual expectations were accepted individually; no interaction
assertion was removed. Subsequent expanded validation and fixture closure are recorded below.

Remaining complete-feature work includes df7f717c85 search/activity details, then
551844b3ef compact prompt navigation and
its independent initial owned-history budget (three screenfuls/item scan cap),
/tui, mouse refinements, AIRS recap source fidelity, native/GNU acceptance,
documentation, and signed/notarized registry delivery. No complete-feature score
or publication is claimed. Reviews are adversarial self-review, not independent
certification.

The first full TUI/config/features execution passed 5,062/5,066 with six existing
skips and no retries (81.23 seconds executing; 166.81 seconds total). Four PTY
fixtures failed: three duplicated an explicit no-alt-screen option already added
by the shared launcher; the owned-screen fixture expected an upstream model label
instead of AIRS's retained model ID. No runtime assertion was removed. The helper
now adds the restriction only when absent; the model expectation matches the
configured fixture. A fresh CLI build and the expanded package suite are pending.

The actual first fullscreen PTY frame was visible during that failed model-label
wait. This is useful diagnostic evidence, not a passing first-frame/exit check.
The final primitive and PTY gates remain required.

## Expanded package and terminal closure

Adding codex-cli rebuilt its feature graph and found one borrow error confined to
the new private-dialog regression; its initial compile receipt is retained. After
releasing the borrowed renderable before input dispatch, the expanded full suite
ran 6,103 tests: 6,102 passed, one failed, seven existing skips, no retries
(73.28 seconds executing; 261.97 seconds including build). The failed fixture set
TERM_PROGRAM but did not answer the actual SSH terminal identity query; the
production policy correctly did not infer Terminal.app from that environment
variable. The fixture now supplies Apple's DA2 response. Production source was
unchanged after that full run.

All 17 focused closing checks pass without retries (2.736 seconds executing;
28.92 seconds total). They include actual first synchronized fullscreen frame,
clean exit, all three launch restrictions, existing SSH/other-terminal/tmux
identity cases, real CLI worktree/fork/cwd transitions, launch-mode reload
preservation, and the new private TypeSafe/MCP dialog test. Private fixture keys
and callbacks reach only the dialog channel, not transcript, agent operations or
AppEvent output; draft and selection ownership are preserved. This is an
expanded full run plus a test-only fixture closure, not an all-green full run.
Native Apple Silicon and final workspace/release validations remain pending.

Scoped CLI/core/config/TUI/features lint passed in 432.36 seconds with one
mechanical fix removing a single-element test loop after the unused Usage marker
was removed. Formatting passed in 18.33 seconds; unrelated Python formatter churn
was restored. No tests were rerun solely after lint/format. Local debug executables
were built and version-probed; debug-binaries.json records their pre-lint hashes.
These are unsigned local musl executables, not the promised Mac delivery.
