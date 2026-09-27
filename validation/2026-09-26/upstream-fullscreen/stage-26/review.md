# Clipboard delivery certainty

Adapts the CopyOutcome/CopyStatus contract from d7d9f2b9b5 before connecting
transcript selection. Native/WSL acceptance is confirmed; OSC 52/tmux transmission
is an unconfirmed request. Empty content is rejected before any clipboard backend.
A returned native lease replaces the prior owner; a terminal request or leaseless
success preserves it. The small contract lives in its own module.

AIRS intentionally retains its SSH routing: no native or WSL clipboard write on
the remote machine. Local native copying still wins without unsolicited terminal
forwarding. Existing OSC 52 size limits, tmux checks, macOS stderr suppression and
Markdown/HTML behavior remain. Upstream's different parallel native/terminal
routing is not imported as a selection dependency.

Consumers include /copy, last reply, transcript export, private diagnostic report
and the MCP authorization dialog. General copies show the upstream export fallback;
diagnostics and authorization show their own appropriate local fallback. No raw
clipboard error or callback is added to the MCP/doctor notice. The MCP input and
receiver are untouched by copying. This does not alter gateway OAuth or inference.

Tests inject all clipboard backends and exercise the successful backend/status
matrix, retained native ownership, empty payload rejection, existing SSH routing,
private callback masking and receiver state, draft preservation and no inference
operations. Reviewed UI snapshots are required before closing this slice.

The first compile caught a duplicate module declaration accidentally inserted in
an existing test by the edit script; corrected without changing production API.
Its original receipt is retained. Results will be recorded after validation.

No fullscreen feature score or native delivery claim is made by this prerequisite.
A session-lived owner for transcript selection belongs to the following Tui/input
integration; this contract does not by itself connect mouse or fullscreen mode.

## Validation result

The first executing focused run passed 128/131; all three failures were the new
or changed snapshots. Review confirmed masked callback text and an unconfirmed
copy notice, unchanged generic copy fallback, and the confirmed report caption.
After accepting only those three snapshots, the full TUI/config/features run
passed all 4,966 tests, with six preexisting skips and zero retries (59.48 seconds
test execution; 61.17 seconds total). This also closes the prior source-layout
slice against a full all-green package run. No platform or full-workspace claim.

Scoped lint passed in 120.96 seconds with no warnings/fixes; formatting passed in
18.08 seconds. Adversarial self-review confirms existing native/WSL/SSH routing,
size limits and private callback behavior through actual consumer tests; no
model/context or gateway authentication change. Existing failed attempts are
retained. Native clipboard ownership after closing transcript overlays remains
part of the following input/terminal-lifecycle slice, not claimed complete here.
