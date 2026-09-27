# Interactive viewport and terminal ownership (in progress)

Connects d7d9f2b9b5's source-anchored selection/view modules to the transcript
pager, input dispatch, session-lived selection clipboard and terminal screen/mouse
lifecycle. The copy-outcome contract is already retained in stage26. This is a
coherent mechanical adaptation across separate upstream modules, exceeding the
normal total patch-size guideline because view, pager, event enum and terminal
ownership must compile together. No whole-feature score until connected acceptance.

AIRS adaptations: retain boxed overlays and omit the absent analytics product;
reuse AIRS accent styling and existing keyboard labels. The terminal currently
re-emits cursor styles each frame, so invalidation resets its hidden-cursor belief
without importing upstream's separate style-cache optimization. Repeated alternate
screen entry remains guarded. Existing startup screens explicitly ignore mouse
reports while preserving pending paste/input. Gateway/OAuth paths are unchanged.

All existing overlay and viewport regression tests are retained in legacy modules
alongside the imported interaction tests. New tests cover pointer/keyboard
selection, copy confirmation, clipboard ownership after overlay close, resize,
live updates, pagination, terminal restoration and failed writes. Existing goldens
are not blanket accepted. Missing upstream dependencies are adapted to actual
AIRS consumers, including TextLayout measurement visibility and the footer type.

First compile (49.36 seconds) found the missing accent-color helper, preserved
TextLayout re-export and exhaustive startup mouse cases. These were corrected;
initial failure receipts are retained. The results and exact-identity pagination consumers are recorded below.
AIRS recap provenance, fullscreen launch mode, search and final Mac/GNU acceptance
remain required. No signed artifact or registry publication is claimed here.

## Connected validation

The first executing run passed 350/365. Eleven external snapshots and one inline
snapshot reflected the new selection/status footer: removed tildes and percentage
chrome, complete copy/navigation hints, retained AIRS key-label spacing and accent
style. Review confirmed unchanged transcript, diff, hyperlink, status and
visualization content. These changes were accepted individually; the original
legacy snapshot namespace was retained rather than leaving orphan goldens.

Three older behavior assertions needed adaptation: scrolling into live output now
holds its displayed revision; Home compares the retained content area rather than
an independently changing loading footer; and the footer no longer shows 100%.
The legacy render-count assertions remain, proving offscreen history is not
formatted for status. Explicit prompt-navigation review found that a frozen live
snapshot could hide a newer canonical prompt: ensure_entry_visible now resolves
the canonical cell identity and releases the frozen selection only when revealing
that prompt. The retained anchor regression exercises the newer queued prompt.

Exact persisted-identity exploration joins are now connected through history
pagination, including transparent reasoning details and live pending calls.
Four adapted upstream scenarios verify pending completion, identical text from a
different turn, every reasoning/answer boundary, and failed commands. Computer/CUA
activity grouping is deliberately excluded; ordinary gateway MCP remains separate.
A missing web-search summary helper was adapted by sharing AIRS's unchanged
styled summary between display and source-aware transcript rendering; the failed
compile is retained as diagnostic evidence.

The next focused run passed 369/371. Both remaining failures were legacy test
adaptations: the revised retained-text expectation had not been applied to one
assertion, and tail visibility had been incorrectly translated as following mode.
After correcting those assertions, the complete TUI/config/features run passed
all 5,046 tests, six existing skips, zero retries (59.99 seconds executing tests;
138.53 seconds including compilation). The current source includes all prior
AIRS regression tests plus selection, clipboard lifecycle, screen restoration and
page-join coverage. This is local package acceptance, not full workspace, native
Mac, signing or registry acceptance.

Remaining complete-feature gates: connect opt-in fullscreen startup/session
ownership, search and /tui, later mouse refinements, AIRS recap source/copy behavior,
final GNU/native validations and signed Apple Silicon package acceptance. The
unused upstream Usage overlay input marker remains inert pending final scope
cleanup; it does not enable an analytics/usage product.

Scoped lint passed in 151.86 seconds. Its sole automatic fix moved the shared
footer type before the preexisting test module; no runtime behavior changed.
No warnings remain. Required formatting passed in 18.55 seconds; unrelated
formatter-only Python changes were restored. Tests were not rerun solely for
formatting/lint. The original failing receipts remain in this directory.

Adversarial self-review: no unresolved blocker within the connected overlay
selection/page-join slice, but no independent certification or complete fullscreen
score is claimed. Config selection, credentials and conversation context are not
modified by this slice; complete-feature native/workspace gates remain mandatory.
