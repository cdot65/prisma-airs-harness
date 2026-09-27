# Transcript search and local activity details — local closure

Selective adaptation of df7f717c85. Case-insensitive literal Find uses source
positions, a 4 KiB query cap, at most eight entries and bounded 16 KiB text
windows per scanning frame (plus bounded query overlap). Older-history loading
uses the existing pagination contract. Closing Find restores prior presentation
and position. Individual activity disclosure follows retained tool identities
without mutating execution or model-visible context.

F3/global.find_transcript, F4/global.focus_activity and pager.find accept existing
keymap remaps/chords/unbinding. New defaults yield to configured keys. Search
owns editor input; selection's fixed keys retain priority. Pending chords expose
available completions and retain the existing one-second expiry. Resume previews
receive the complete resolved keymap so Find honors the same editor bindings.

The coordinated upstream interaction import exceeds the usual patch-size limit
because its viewport, disclosure, search, keymap and app adapters depend on each
other's APIs. New behavior lives in separate modules; AIRS orchestration is
adapted at call sites. No voice, CUA, daemon or authentication feature is imported.
Existing TUI copy ownership is retained instead of adding upstream's duplicate
widget clipboard helper. Existing contrast-aware accent and shortcut spacing
remain. The fork's launch/private-dialog regressions are preserved.

Validation and individual snapshot review are pending. This is not a completed
feature score, an independent certification, or a new published binary.

The initial compile (149.70 seconds) found the fork's shortcut-label API and older
cache fixtures; both were adapted. The first executing focused run passed
511/535 with no retries (13.98 seconds execution, 238.67 seconds including
compile/link). Nineteen failures were individually reviewed snapshot changes:
Find/activity inventory, new chord completions, pager Find hints, existing AIRS
spacing and dim/cyan styles. One upstream F8 debug fixture was rejected and the
existing Ctrl-O matched-action test retained. Remaining string expectations were
adapted to AIRS labels. The page-folding fixture used an unimplemented Computer
cell; it now exercises actual AIRS exploration commands with identical
source-identity/reasoning/copy assertions. Both private-dialog regressions passed,
including Find already active. Complete validation after these changes is pending.

## Full package run and focused closure

Full CLI/TUI/config/features validation ran 6,152 checks: 6,149 passed, three
viewer snapshots failed, seven existing skips, no retries (72.65 seconds execution;
242.21 total). The three failures showed only the new `f3 find` footer in existing
inline/backtrack, visualization and status scenes; each was reviewed and accepted.
No production changes followed this full run. All 538 subsequent behavior/snapshot
checks passed. The one new real-terminal check initially sent F3 during provisional
startup and timed out; its fixture now waits for the final model footer. The
terminal check then passed for both inline and fullscreen modes, including actual
F3 decoding, query input, Escape and preservation of the unsent draft (2/2 selected
checks pass, no retries, 1.242 seconds execution; 27.13 total). This is a full run
plus focused/test-only closure, not an all-green full-suite claim.

Actual config schema generation passed in 18.66 seconds. The changes add only the
three keymap fields. Lint, formatting and native/final delivery remain pending.

Scoped CLI/core/TUI/config/features lint passed in 286.50 seconds with zero
warnings or fixes. Formatting passed in 20.82 seconds; unrelated Python churn
was restored. Tests were not repeated solely after lint/format. Complete-feature
scoring and native/release acceptance remain outstanding.
