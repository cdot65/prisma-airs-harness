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
| `f8ab57359dde6b6d5de1aee613c18fe60b661aeb` | Already present, audit later gaps | `mcp_login`, `oauth_callback_input` and their tests already implement hidden bounded callbacks. AIRS extends this with private TUI progress; retain gateway resource binding. |
| `b33199b1fbcf96ea1e4035729158f6ecd1760192` | Adapt | AIRS `airs_doctor` is a separate entry point from upstream `doctor`. Bound filesystem probes in the AIRS path rather than importing unrelated desktop/daemon checks. |
| `0ad18769ad8dfb7a6cf16b1bdd26b6e5f8773f64` | Existing AIRS redaction; audit remaining edges | `airs_status::configuration` already discards TOML error payloads and `airs_doctor` emits a fixed configuration failure. Canary tests must exercise AIRS, not only upstream doctor. |
| `ef9f3d022aff6bd087c640392dc8608743aa411d` | Adapted | Limited recovery-command dispatch now covers embedded as well as remote replay-only threads. AIRS sign-in, doctor, MCP and TypeSafe are in the command allowlist. Doctor accepts an absent thread. Preserve drafts and never replay a failed turn. |
| `9ed1b7469921e056b7cdbe2b98d1b3251d08e9f9` | Compare existing startup draft handling | `startup_draft` already exists. Adopt only missing exactly-once behavior, not upstream fullscreen defaults. |
| `c7828dd0109c75bcb5b07dc2d95c68947e56239a` | Assess narrow correctness delta | Requires changes across StepInputs, Guardian and execution environment ownership. Executor contexts are not AIRS environment UUIDs. Do not import the 40+ file refactor without a demonstrated active-product gap. |
| `568518357023708ecbeddb764ef8956ecaa076d7` | Defer daemon portion; assess active permission path | Depends on captured executor state. AIRS does not enable managed daemon recovery. Check existing origin-context behavior separately. |
| `2b842962883f2e01526b9c64e383cb2375123a5d` | Defer OpenAI refresh; verify AIRS isolation | Upstream refresh is OpenAI-specific and explicitly a no-op for static catalogs. AIRS uses its local capability catalog; its gateway remains the authority. Credential generation changes invalidate the active session rather than permit adopting another identity. |
| `d5b29951aca5205bc8415120b56dd4bc295f5c83` | Adapt only missing classification | AIRS already distinguishes 401, 403, 446 and blocked-200 probes. Test transport/runtime paths for policy denial and unsafe replay before changing retry architecture. |
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
