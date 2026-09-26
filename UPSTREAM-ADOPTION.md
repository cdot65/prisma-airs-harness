# AIRS selective Codex adoption

Implementation ledger for the September 22 PRD. Decisions are based on behavior,
not ancestry counts. Baseline: `1a6df30b5654ca8b892440b21e4bf1b5442b1f38`
(AIRS 0.1.2 / bundled CLI 7.1.5). Reviewed upstream:
`a1f40f3f1326eff7c81b860a4f4e27c1be186e10`. The prior Codex 0.154 import is
`719b317241`; some later features were independently adapted into AIRS.

## Candidate ledger

| Upstream commit | Decision | AIRS implementation and validation boundary |
| --- | --- | --- |
| `064e701b0fa4f5c0dcaf195a565d088f99bb6af8` | Adapt lifecycle semantics; defer RPC | `airs_status`, `airs_access`, `airs_auth_lifecycle` and TUI `airs_recovery` already share CLI-owned operations. Upstream secondary gateway OAuth would change AIRS SSO-or-key semantics. Audit status and cancellation using existing private adapters; add no protocol or credential service unless a concrete gap requires it. |
| `f8ab57359dde6b6d5de1aee613c18fe60b661aeb` | Already present; retained | `mcp_login`, `oauth_callback_input` and their tests implement hidden bounded callbacks. 320 OAuth tests and the native manager fixtures passed. AIRS private progress and gateway resource binding are retained. |
| `b33199b1fbcf96ea1e4035729158f6ecd1760192` | Adapt | AIRS `airs_doctor` is a separate entry point from upstream `doctor`. Bound filesystem probes in the AIRS path rather than importing unrelated desktop/daemon checks. |
| `0ad18769ad8dfb7a6cf16b1bdd26b6e5f8773f64` | Existing AIRS redaction; audit remaining edges | `airs_status::configuration` already discards TOML error payloads and `airs_doctor` emits a fixed configuration failure. Canary tests must exercise AIRS, not only upstream doctor. |
| `ef9f3d022aff6bd087c640392dc8608743aa411d` | Adapted | Limited recovery-command dispatch now covers embedded as well as remote replay-only threads. AIRS sign-in, doctor, MCP and TypeSafe are in the command allowlist. Doctor accepts an absent thread. Preserve drafts and never replay a failed turn. |
| `9ed1b7469921e056b7cdbe2b98d1b3251d08e9f9` | Defer whole patch; preserve existing draft behavior | The 32-file, 1,987-added-line patch combines deferred submission, fatal-startup recovery and owned-screen rendering. AIRS retains its existing non-submitting startup editor and tested protected-input handoff. This preview improves unavailable-thread recovery; it does not claim upstream's new deferred-submit/fatal-startup recovery behavior. |
| `c7828dd0109c75bcb5b07dc2d95c68947e56239a` | Defer executor refactor | 44 files, 872 additions, across StepInputs, Guardian and execution environment ownership. Executor selections are not AIRS environment UUIDs. The supported local AIRS boundary passes installed revocation/history/config fixtures; this is not acceptance of remote executor switching. Revisit with a separate remote-execution contract. |
| `568518357023708ecbeddb764ef8956ecaa076d7` | Defer with captured executor work | Eight files and 187 additions depending on the previous refactor. AIRS blocks upstream service commands and GuardianV2, and does not introduce managed daemon recovery. Do not enable these paths as a side effect of a reliability import. |
| `2b842962883f2e01526b9c64e383cb2375123a5d` | Defer OpenAI refresh; verify AIRS isolation | Upstream refresh is OpenAI-specific and explicitly a no-op for static catalogs. AIRS uses its local capability catalog; its gateway remains the authority. Credential generation changes invalidate the active session rather than permit adopting another identity. |
| `d5b29951aca5205bc8415120b56dd4bc295f5c83` | Narrow terminal-status fix; defer retry API refactor | AIRS doctor distinguished denials, but conversation requests replayed 401/403/446. Shared classification now rejects retries for these statuses, even with Retry-After. No retry timing API or schema change. |
| `3b50349e1a80930e6e9dcf6d9377d65ef55ef052` | Defer | Hosted app/account resource targets are not needed for gateway ServiceNow. No schema import. |

No dependency updates are currently selected. Each accepted patch carries only
its required code/tests; generated schemas and lockfiles follow owning changes.
Gateway-facing MCP OAuth uses upstream client machinery while the gateway retains
upstream OAuth credentials. CIE/CAS semantics, issuer/resource binding and workspace
keys remain AIRS-owned. TypeSafe remains an independent optional judge credential.

## Baseline and scoring

The initial `just test -p codex-cli -E 'test(airs_)'` baseline against unmodified
0.1.2 failed at linking because the persistent home volume filled; no tests ran.
The build cache was preserved on the larger local volume. Installed native acceptance
from `validation/2026-09-21/stable-0.1.2` is historical baseline evidence, not proof
that new code passes. Ubuntu acceptance host has only 431 MiB free; preserve it.

Each feature score uses correctness (3), boundary preservation (3), regression
evidence (2), maintainability (1) and user clarity (1). Below 9 requires revision;
a hard-fail condition overrides the arithmetic. A source/test score is not a
cross-platform release score. Pending evidence cannot receive passing points.

### P1 slice 1: attempt-bound MCP cancellation — 9/10 source score

At `074f00355d`, MCP wait/progress Cancel and Escape carry their originating attempt.
The shared recovery state rejects delayed/duplicate cancellations; doctor uses the
same guarded operation. A queued callback from every progress stage is exercised
after a newer recovery operation starts. No auth protocol, credential, schema or
visible copy changed.

Local full TUI library run: 4,351 passed, four cursor-color failures, two skips.
All 58 AIRS tests passed. The four cursor tests subsequently passed with the
inherited `NO_COLOR` removed, matching owned CI configuration. No snapshots were
accepted to hide the environment mismatch. Source score: correctness 3/3,
boundaries 3/3, regression evidence 1/2, maintainability 1/1, clarity 1/1.
The remaining evidence point requires installed candidate/platform checks.

Additional baseline evidence: installed 0.1.2 MCP/doctor terminal fixtures passed
11 checks; the opt-in five-minute expiry passed separately (301.355 seconds).
Existing OAuth client tests: 320 passed, eight skipped. These checks do not claim
a new real-account gateway or ServiceNow login. Full GNU workspace run 3817 is
testing `08fd6c2c12884a33c65318e02ca541536e707fa8` via the owned branch-push
workflow. It does not include subsequent lifecycle commits. Final result:
18,375 passed (one passed on retry), three failed, 34 skipped. The failures are
the same remote shell-snapshot cases recorded for stable 0.1.2:
`remote_pipe_recovery`, `remote_tty_recovery`, and `remote_sandbox`. They remain
failures, not an implicit waiver; the final source still needs its integrated run.

### P1 slice 2: company sign-in menu ownership — 9/10 source score

Recovery state records whether the active attempt belongs to company SSO,
doctor, MCP or TypeSafe. Opening a company sign-in menu captures only a company
attempt; its Cancel action cannot acquire ownership of another operation.
Pre-login MCP Cancel only dismisses its menu. Delayed cancel/finish and concurrent
start tests cover all four operation kinds, and UI callbacks retain the captured ID.

The full local TUI library suite passed: **4,357 passed, two skipped**, with
`NO_COLOR` unset as in owned CI. Existing visible snapshots remain unchanged.
Score uses the same 3+3+1+1+1 rubric as slice 1; installed candidate checks remain
pending. No credential policy, browser URL handling or agent API was added.

Other package scores and release readiness remain pending.

### P4 slice 1: recovery from an unavailable conversation — 9/10 source score

Adapted `ef9f3d022aff6bd087c640392dc8608743aa411d`, resolving five overlapping
files. Excluded unrelated permission-selection and sparkle dependencies. Retained
the existing AIRS input queue representation and event-channel rebinding.

The upstream remote-only availability guard would miss AIRS's embedded
history-read fallback. Availability now follows the actual listener attachment
for both targets. The regression runs recovery/local command dispatch and
new/clear creation against both, including stale review/MCP activity. Ordinary
prompts and unknown commands remain editable; uncertain submissions are retained
without autosend. Pasted commands require another explicit submit after expansion.
The doctor result remains bound to its original optional thread identity.

Full TUI library: **4,363 passed, two skipped** after the embedded adaptation.
`just fix -p codex-tui --lib --allow-staged` passed without warnings or rewrites.
The imported working-directory snapshot was reviewed and passed unchanged.
Score: 3+3+1+1+1; installed terminal/platform acceptance remains pending.

Follow-up review found report export still required a nonempty thread identity.
Corrected the report guard to accept its original optional identity while retaining
view invalidation and rejection after switching to a different thread. Actual
overview/export callbacks cover the threadless case. Full TUI run: **4,364 passed
(one background-exit test passed on retry), two skipped**. The feature returns to
9/10 after this correction; the first-pass failure is retained in the log.

### P1/P3 slice: verification belongs to its configuration — 9/10 source score

The verifier previously checked auth generations but could report old-route access
after config/catalog replacement. A private digest snapshot now covers both files
read under the configuration lock. It is checked after the credential helper,
before network use, after response headers and before accepting the response body.
Deleted, unreadable or changed files produce a fixed recovery message, not a retry
or a native-store mutation. The lock is not held across the helper that must acquire
it itself. No public state/schema or dependencies changed.

All **150 AIRS CLI unit tests passed** (277 unrelated tests filtered), including
request-count regressions for replacement/deletion before send and config/catalog
changes during a successful response. Existing 401/403/446/blocked-200 and native
credential tests passed. The new fixed user-visible message has snapshot coverage.
An initial snapshot-in-loop test setup error was corrected; the final run is green.
Score: 3+3+1+1+1, with installed candidate validation still pending.

### P4 slice 2: bounded catalog diagnostics — 9/10 source score

Adapted the filesystem-helper pattern from `b33199b1fb` into AIRS doctor. Catalog
validation runs in a private child with a two-second wait budget and null output;
timeout initiates termination without waiting on uninterruptible filesystem I/O.
The helper rejects links, nonregular files, invalid UTF-8/JSON and files over
1 MiB. Errors contain no catalog contents. This bounds the catalog check, not
OS process creation or every earlier configuration read.

All 154 AIRS CLI tests passed, including a stalled child and exit classification.
Seven native doctor/PTY tests passed against the locally built candidate, including
actual helper dispatch with zero gateway/credential work, FIFO/oversized catalogs,
cancel/retry and redacted reports. `just fmt` completed with the installed DotSlash
on PATH; unrelated baseline formatter changes were removed. Score: 3+3+1+1+1;
cross-platform installed package validation remains pending.

### P3 slice: terminal denial does not replay inference — 9/10 source score

An actual installed 0.1.2 fixture produced three identical requests for each of
401, 403 and 446 under a two-retry stream budget. The same fixture passes against
the rebuilt candidate with exactly one request per status and no local tool or
MCP execution. Server Retry-After cannot override these terminal decisions.
All 338 protocol tests passed; transient 429/503 retry behavior remains covered.

This is a narrow AIRS-motivated correction in the existing shared classifier,
not a claim to have imported `d5b29951ac`'s retry API refactor. No error variants,
wire schemas, dependencies, automatic login or credential fallback were added.
Score: 3+3+1+1+1; final integrated/platform tests remain pending.

Scoped protocol Clippy passed. Twelve rebuilt-native MCP/environment fixtures
passed, with the opt-in five-minute expiry fixture skipped in this run (its earlier
stable-baseline result is separate). They cover cancel/callback behavior, revocation
after logout/relogin, pinned history/catalog, credential destination changes and
rejection of unsupported upstream services. Warm metadata status made zero network
requests across ten measured launches; debug-vs-release timings do not establish
the release startup performance budget.

### P1 slice: time-stamped access observations — 9/10 source score

CLI login/doctor and the TUI's private doctor adapter share a completion timestamp
in the existing verification summary. Doctor JSON adds nullable
`gateway_access_checked_at` (Unix seconds); passive inspection leaves it null.
The timestamp records an attempted check, including denial, and is not an access
grant. Results remain transient and bound to the selected environment/auth
generation and config/catalog snapshot; no reusable success cache is introduced.
The generic busy-operation message no longer claims a browser login exists when
the shared owner is doctor, MCP or TypeSafe.

All 155 AIRS CLI tests passed after reviewing the timestamp-only inline and file
snapshot changes. Seven native doctor/PTY tests passed, including equal timestamps
in JSON and visible success/denial details, zero passive inference, exactly two
explicit probes and zero MCP requests. An initial run failed on the three expected
inline snapshot updates; corrected snapshots are retained in source. Score:
3+3+1+1+1; final integrated/package checks remain pending.

## Import maintenance and remaining gates

The unavailable-thread patch required five conflict resolutions; the adaptation
keeps AIRS handlers in private modules and avoids new app-server RPC schemas.
This count measures this patch, not a successful whole-upstream merge rehearsal.
Future weekly triage should compare behavior against the frozen baseline and record
newly accepted commits here. Rehearse larger merges in a disposable worktree;
do not manufacture ancestry for omitted code. Aim to keep the next comparable TUI
slice within five manually resolved runtime files; reassess for larger features.

No dependency, Cargo/Bazel lock or generated protocol/config schema update is
needed for the selected code. Fullscreen, remote executor/daemon activation and
hosted apps remain explicitly deferred. Existing OAuth trust/discovery, PKCE,
callback, scope and refresh transaction machinery is preserved; there is no new
MCP device grant or upstream ServiceNow credential path in the harness.

Owned native build workflows accept explicit `airs-preview-*` tag pushes. They produce candidates only; publication still uses installed platform
acceptance and signing/notarization gates. Inherited workflows remain disabled.
The final source needs full GNU validation, installed exact packages on all three
supported targets, comparable release-build latency and the documented upgrade/
rollback checks. Real-account browser consent and ServiceNow reads are separate
owner acceptance; fixtures do not claim them. Stable remains 0.1.2.

### Integrated follow-up: HTTP 200 policy envelopes

The actual-agent release probe exposed a remaining retry gap for JSON and SSE
`hooks_failed` denials returned under HTTP 200. The first preview candidate was
superseded before publication. A private API module now bounds JSON inspection
at 64 KiB and two seconds, reports explicit gateway policy fields without raw
body content, and rejects non-stream JSON terminally without pretending every
invalid response is a policy denial. Streamed invalid requests terminate
immediately; a later completion or stalled connection cannot override denial.
Normal SSE bytes are untouched. No auth fallback, dependency or wire-schema
change is involved. Six focused cases and the complete 195-test API suite pass;
the actual agent sends one request for each of 401/403/446 and JSON/SSE HTTP 200
policy denials, with no local tool or MCP execution. Source score returns to 9/10;
corrected installed-platform checks remain required. Version alpha.2 replaces the
unpublished alpha.1 candidate. The companion legacy CLI timestamp snapshot also
passes for both binary targets; original full-workspace failures remain recorded.

## Validated preview handoff: 0.1.3-alpha.2.mcp.1

Runtime/packaging source: `bb31dcdd813122f577efb1b8b647f2c06f3078c4`.
Frozen validation tooling: `766c9fb83a9bc32deacd2125cefcafd5b6527db7`.
The validation-only follow-up handles the explicit startup-busy response before
opening the passive MCP menu. It does not retry sign-in, reconnect or tool calls,
and did not require changing the runtime binaries. Original failed Mac CI evidence
is retained separately from the corrected installed checks.

All four packages are published under `mcp` at `https://npm.cdot.io`; stable
`latest` remains 0.1.2 and the bundled CLI remains 7.1.5. Exact candidate and fresh
anonymous registry acceptance passed on native Linux x64 (Alpine), native Linux
ARM64 and signed/notarized Apple Silicon. Each platform also passed Jev through
the actual agent approval path, native key handling, doctor checks and one-request
handling for 401/403/446 and explicit HTTP 200 JSON/SSE policy denials. Candidate
acceptance includes preserved upgrade/rollback from 0.1.2 and timed fixture refresh.

Full GNU workspace run 3826: **18,405 passed, 3 failed, 34 skipped**. The three
remote-shell snapshot failures have the same normalized assertion details as
the retained stable 0.1.2 run. They remain failures; no fully green workspace is
claimed. The final release-binary comparison met the ten-launch startup budget
and the cancellation budget, with zero inference/tool/token requests. This used
fresh application homes and a warm OS cache, not a cold-boot measurement.

| Implemented slice | Preview score |
| --- | --- |
| Attempt-owned MCP cancellation | 9/10 |
| Company sign-in operation ownership | 9/10 |
| Unavailable-conversation recovery and reports | 9/10 |
| Configuration-bound gateway verification | 9/10 |
| Bounded catalog diagnostics | 9/10 |
| Terminal denial handling without automatic replay | 9/10 |
| Time-stamped access observations | 9/10 |

Scores describe the implemented preview and controlled installed evidence. The
remaining confidence gap is real-account acceptance, not a claim that fixtures
prove company SSO, ServiceNow authorization or Jev accuracy. Owner Ubuntu and
real-account sign-in/consent/read checks remain separate; stable promotion is
not authorized by these scores. Evidence and immutable audit inputs are retained
in `validation/2026-09-22/selective-upstream-alpha2/`.

## Terminal integration — September 24, 2026

Frozen upstream review: `4891c4e35fe8b306433f6c0f04dd74bd9e4257e9`.
The following slices follow the prior alpha.2 preview. Each passed its source
phase before the next began; scores are implementation assessments, not owner
acceptance or a claim that all upstream features were imported.

| Phase | Upstream provenance | AIRS behavior and evidence | Source score |
| --- | --- | --- | --- |
| 1: transcript and copy | `095da4b7e8b70b01afb5c6131ef926dcb8c0d85d`, `dd9512c0008859f6c95fb2dae2290a2cb94b38fb` | Idempotent transcript entry, deferred inline reflow, rendered-height scrolling, latest completed assistant message for `/copy`; 4,367 TUI tests passed | 9/10 |
| 2: answer draft recovery | `811fe5fa4311d33a4d4626509025b6c61100245e`, `3032b387fa20fd782030587a305a27e6d02a7adc` | Recover typed answers when the live turn ends; preserve the composer, history search and gateway sign-in; no automatic answer submission or historical-event recovery; 4,375 tests passed | 9/10 |
| 3: terminal size and links | `7d2c58e6e0811d326accdfe9913417a44893dd2f`, `7db578fca663b3e0026996e9c50104784d420744` | Bounded asynchronous tmux size queries and full URL destinations across narrow/wide prompt wrapping; 4,382 tests passed | 9/10 |

All TUI counts exclude two skipped tests. The initial color-environment failure
and intermediate adaptation failures remain in the work record; final source
checks ran with the inherited `NO_COLOR` override removed. Scoped Clippy passed.

Phase 4 is explicitly deferred: upstream fullscreen selection/search introduces
a new logical-source renderer and broad input ownership changes. It is not a
small compatibility flag and is not represented as implemented or scored. The
current inline default and transcript overlay remain. Terminal.app SSH detection
also awaits its dependency/startup-probe review.

Phase 5 delivered **0.1.3-alpha.3.mcp.1**, Apple Silicon only, using the explicit
`owner-authorized-mac-preview` release scope and `mac-preview` registry channel.
This scope requires signed Mac evidence, restricts the launcher to Darwin ARM64,
and cannot satisfy an ordinary three-platform or stable release gate. Linux
distribution builds await the owner's Mac test. Stable and existing `mcp` tags
remain unchanged. Exact candidate and fresh anonymous registry acceptance passed;
the signed/notarized Mac native package and Mac-only launcher are published.
The immutable Mac-only version cannot later gain Linux packages; the eventual
all-platform handoff will use a new version.

No gateway authentication, MCP OAuth audience/resource, native-store credential
boundary, bundled CLI version, protocol schema or dependency upgrade is added by
these terminal slices. Jev remains behind the existing actual-agent approval
boundary and is included in regression acceptance.

### Terminal preview acceptance and assessment

Runtime/packaging: `7b177cc58a875f7f35ab91960cbc06f57784d7e1`.
Frozen validators: `a7488be6ed6f526717a7e3c970f38cdf51f35bd7`.
The tooling follow-ups corrected terminal fixture assumptions, without changing
runtime bytes: rendered text can be split by cursor movements, title generation
is separate from user submission, resize checks need observed layout, and active
question streams must be keyed by their originating prompt rather than request
arrival order. Every intermediate failure remains in the retained record.

Apple accepted notarization submission `ac1ee738-0157-42dd-8e7e-9906a9171019`.
Immediate online ticket verification initially failed; bounded rechecks of the
unchanged signed bytes passed. Final candidate and fresh registry checks verify
the Developer ID and explicit notarization requirement. No verification gate was
bypassed. The package binary SHA-256 is
`ba2e7656805bd21f3ad3718f982680df1743a03fb37776d90493a1ae3d772b99`.

Installed checks cover native-store access, transcript resize and preserved drafts,
question recovery without automatic submission, MCP management, policy denials,
diagnostics and Jev through the actual agent approval UI. An additional real tmux
3.7c pane test passed narrow/wide resizing and draft recovery. Stable 0.1.2 and
alpha.2 upgrade/rollback both preserve native credentials and conversation history
while completing three synthetic MCP turns. These are isolated fixtures, not
owner-account SSO, ServiceNow or paid Jev acceptance.

Full GNU run 3833: **18,423 passed, 3 failed, 34 skipped**. All three remote-shell
assertion blocks match the retained raw stable baseline after normalizing only
ANSI/timestamps, whitespace, process IDs and temporary paths. The complete original
log is retained; the workspace is not reported as fully green. Release contract
checks passed 82 tests, workflow/cache checks passed three, and scoped Clippy passed.
Ten measured warm starts met the 10% comparison budget; maximum cancellation was
11.7 ms. Measurements use fresh homes and warm OS caches, not cold boots.

| Implemented phase | Final preview score | Remaining confidence limit |
| --- | --- | --- |
| Transcript navigation and copying | 9/10 | Owner terminal and clipboard acceptance |
| Answer draft recovery and history search | 9/10 | Owner real-session workflow acceptance |
| Terminal size and hyperlink resilience | 9/10 | Broader terminal/remote combinations |
| Integrated signed Mac handoff | 9/10 | Owner real-account acceptance before Linux builds |

Scores use correctness 3, AIRS boundaries 2, tests 2 of 3, maintainability 1 and
clarity 1. Automated source and installed checks earn the preview gate; unrun
owner checks are not awarded the final test point. Fullscreen/search is a deferred
compatibility decision, not an implemented phase awarded a score. Stable remains
0.1.2; the prior cross-platform `mcp` tag remains alpha.2. Evidence is retained in
`validation/2026-09-24/terminal-upstream-alpha3/`.

## Selected menu fill — September 24 follow-up

The requested blue selection comes from upstream contrast/picker work
`132c2be239ecbc1f2a9bb22d9210fefe887986a5` and its shared-menu adoption in
`c9d13e8c757cd330a572e9a39e365866069aef07`. Selectively adapt the contrast resolver
and selected-row fill, preserving existing AIRS menu layout and authentication.
Known truecolor/256-color palettes use light/dark blue with at least 4.5:1 text
contrast after conversion; unknown/limited palettes use terminal-owned reverse
video. Wrapped rows and truncated ellipses retain the fill; disabled rows do not
acquire it. No fullscreen renderer, input-state or OAuth change is involved.

Source TUI gate: 4,388 passed, two skips. Initial failures exposed a lost style
on truncated lines; applying the style after truncation corrected it. Reviewed
snapshots reflect the full-width fill and styled padding, with unchanged visible
text. The source entry point is `codex-rs/tui/src/style.rs::selection_style`,
with contrast/palette resolution in `style/contrast.rs`. Shared menu rendering
applies it in `bottom_pane/selection_popup_common.rs::apply_row_state_style`,
including the single-line path after truncation. This keeps the fork difference
local to the style and shared renderer instead of copying individual AIRS menus.

Runtime/packaging source is `76d88c609556d1e00fbe1925ce34fe9521920364`;
frozen acceptance tooling is `061c3c73a3af0348520d3ef2daa6a77cbdb5f054`.
The initial Mac CI check assumed a selected option occupies exactly one screen
row. Wrapped descriptions invalidate that assumption; the tooling-only correction
checks contiguous selected rows and movement. Original CI failures are retained.
The corrected checks passed on the signed binary and the installed candidate,
with real PTY startup palette negotiation in light and dark themes.

Full GNU workspace: 18,429 passed, three failed, 34 skipped. Each failure matches
the raw stable baseline after normalizing only timestamps, ANSI, indentation,
process IDs and temporary paths. This is not an all-green suite. The exact signed/notarized candidate and fresh anonymous registry installation
passed all ten acceptance stages, plus native TypeSafe and actual-agent Jev
approval, policy retry and doctor checks. The installed suites each passed 53
tests with three Linux-only skips. Stable 0.1.2 and previous Mac alpha.3 upgrade/
rollback preserve configuration, native credentials and conversation history.
Warm startup/cancellation budgets passed against the exact stable binary.

`0.1.3-alpha.4.mcp.1` is published under `mac-preview`, Apple Silicon only,
with CLI 7.1.5. Stable and existing `mcp`/Linux channels remain unchanged. Source
rendering, native acceptance and signed Mac delivery each score **9/10**, using
the rubric above; owner visual/real-account acceptance reserves the final test
point. Linux distribution builds remain deferred. The public guide passed 23
browser checks. Retained evidence: `validation/2026-09-24/blue-selection-alpha4/`.


## September 26: reliability and fullscreen integration

Work proceeds in separately scored slices; source scores do not imply release readiness.

### Restricted macOS fcntl protection — 9/10 in each dimension

Adapted `04e4d2b40ffdeb7768319bec80bad6b2951c92e4` (PR 46500) to the existing Seatbelt generator. Restrict mutation through read-only descriptors under restricted filesystem profiles; unrestricted execution is retained. Separate unchanged upstream native canary tests prove denial and positive controls. Linux affected suite: 69 passed; native Mac: 104 passed, one ignored child fixture invoked by the parent. Both initial Homebrew Bash stderr failures reproduced on baseline; system Bash passes without weakened assertions. Formatting and scoped Clippy pass on applicable platforms. No native-test early skip was observed. Implementation, code quality, design and bounded feature completeness: 9/10 each. Evidence and limitations: `validation/2026-09-26/upstream-seatbelt/ACCEPTANCE.json`. Remaining integrations, full-workspace and signed-package acceptance are pending.

### Linux fork-safe descriptor cleanup prerequisite — 9/10 in each dimension

Adapted `b086ad5493` (PR 47654) to the existing PTY API without importing the new process launcher. Linux close_range/CLOEXEC with stack-based procfs fallback avoids allocation after fork and retains explicit descriptors and Rust spawn-error reporting. Thirty affected tests pass, including forced syscall failures and fallback. Scoped Clippy/format pass. Required native Mac Bazel lock regeneration succeeds and leaves MODULE.bazel.lock unchanged; Cargo adds only an existing test dependency edge. Cleanup remains best effort on platforms blocking both mechanisms. All four bounded prerequisite scores are 9/10; MCP integration remains pending. Evidence: `validation/2026-09-26/upstream-linux-fds/ACCEPTANCE.json`.

### Local MCP descriptor isolation — 9/10 in each dimension

Adapted `afe4249cfe` (PR 47094) to existing launchers: Tokio child cleanup on Linux/Mac and CLOEXEC_DEFAULT on macOS native relative-path spawning, whose native path bypasses pre_exec. Retain stdio, exec-error pipes, parent descriptors, cwd/env/argv and supported shell fallbacks. Integration tests qualify a leaking positive control and test both servers and descendants. Linux: 352 passes/eight ignored. Mac: 357 passes/eight ignored; three upstream remote-executor tests are explicitly excluded because the published AIRS fixture disables that service. Linux exercises those three using its ordinary codex fixture; final workspace uses freshly built binaries. Scoped lint completes on both platforms with one unchanged static-header expect_used warning; formatting passes. All six source/test hashes match native validation. Implementation, quality, design and bounded completeness: 9/10 each. Evidence: `validation/2026-09-26/upstream-mcp-descriptors/ACCEPTANCE.json`. Gateway HTTP/OAuth is untouched, and best-effort descriptor cleanup is not a sandbox replacement.

### Scoped rustls/AWS-LC refresh — 9/10 in each dimension

Adopted the dependency versions in `e5a2094817` (PR 45489): rustls 0.23.45, rustls-webpki 0.103.15, aws-lc-rs 1.18.1 and aws-lc-sys 0.45.0. No unrelated version changes. Cargo and native-regenerated Bazel locks agree; strict lock check passes. Linux affected suites: 458 passed/eight ignored with the existing CA-clean test runner extended to clear its musl wrapper variable. Initial six backend-selection failures resolve with unchanged binaries/assertions. Native Mac: 466 passed/eight ignored, with the same explicit remote-executor exclusion as the prior slice. Tests exercise custom-CA TLS, CONNECT proxy, negative trust, bounded/non-replayable fallback, WebSocket, identity and MCP OAuth. No Rust behavior changes in this slice. Four bounded scores: 9/10 each. Evidence: `validation/2026-09-26/upstream-tls/ACCEPTANCE.json`; full workspace and signed delivery remain pending.
