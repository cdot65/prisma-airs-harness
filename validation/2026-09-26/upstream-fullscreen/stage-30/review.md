# Compact browsing and bounded hydration — validation in progress

Selective adaptation of upstream 551844b3ef. Double Escape enters compact prompt
browsing; arrows and h/l choose prompts, j/k scroll, configured detail keys toggle
presentation, and Escape restores the original reading position and presentation.
Search, text selection, modal input and shortcut help retain priority. Pinned
prompt headers are presentation-only and excluded from transcript source text.

AIRS retains source-preserving branch-on-edit instead of upstream in-place revert.
Queued edit events hold the selected cell identity, so older history arriving
before dispatch cannot shift the target. A real embedded app-server integration
checks queued selection, canonical attachments, retained source rollout and branch
contents. Remote local-image paths and command-like prompts are rejected before
restoration, with unchanged draft and no thread start/fork requests.

Initial fullscreen hydration has independent three-screenful and 400-item scan
budgets, including unlimited terminal scrollback settings. Hidden items do not
force one-item paging. Loaded pages preserve prompt selection and cancellation,
and stale page responses cannot supersede a current request. AIRS uses its existing
local launch settings rather than reactivating the obsolete TranscriptV2 feature.
Gateway routing, CAS/OAuth, credential storage, protocol schemas and dependencies
are unchanged. The footer says “edit” to describe AIRS branching accurately.

The upstream mechanical interaction import exceeds the normal patch-size limit:
view bookmarks, browsing state, input dispatch, pagination and their fixtures use
coordinated APIs. New behavior is in dedicated modules; existing branch backend
and fork-specific tests remain. No daemon, voice, CUA or alternate auth service is
imported. Review is adversarial self-review, not independent certification.

Initial compile failed on two upstream style APIs, a changed paging signature and
older navigation fixtures; adapters now use AIRS styles and current APIs. The
first focused run passed 216/223, with five snapshots and two fork-specific fixture
expectations failing. The next passed 222/224; only the reviewed edit/shortcut
footer snapshots differed. The first full CLI/TUI/config/features run passed
6,186/6,195, with seven existing excluded tests and no retries. Seven failures
were snapshots of the new browse-prompts hint; two were assertions for that hint
and the pinned header row. Each was reviewed; only expectations changed afterward.
Receipts preserve all failures. The full rerun passed all 6,195 executed checks, seven existing skips and no retries (77.87 seconds execution, 159.94 including rebuild). Scoped CLI/core/TUI/config/features lint passed in 156.44 seconds with zero warnings or fixes. Formatting passed in 17.69 seconds; unrelated Python formatting was restored. Tests were not repeated solely for lint/format.

This is not a complete-feature score or a signed/published Mac handoff. Native Mac,
GNU, remaining interaction features, documentation and registry acceptance remain.
