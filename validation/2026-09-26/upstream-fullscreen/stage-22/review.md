# Cursor-specific history completion and bounded metadata

Adapted the request-state portion of upstream 6ba6b73657. A stale response is
ignored before interpreting its error or touching presentation state. Cancellation
only clears the matching pending cursor, preserving a newer request for the same
thread. Turn metadata fetches are bounded by the missing item-backed turns in the
current page; empty intervening turns can require another bounded request.
Initial history, task overviews and safety-buffering retain the five-turn limit.

124 focused tests passed, zero skips/failures/retries. The real embedded history
API test reads one item at a time through twelve saved turns, checks every later
metadata limit is one, verifies the oldest output and absence of duplicates.
The existing Home-to-start test now injects a stale cancellation and stale error
while each page is pending, then a duplicate error after completion; it still
loads every page and reaches the oldest history. Existing hidden-review,
legacy-server compatibility, recovery and overlay tests passed.

An initial compiler attempt exposed a missing explicit test macro import; its
failure receipt is retained. Scoped lint passed with zero warnings/fixes and
formatting passed. No tests were repeated solely after lint/format. This slice
has no protocol/schema, credentials, network routing or model-context changes.

Adversarial review: the cursor guard precedes error handling; empty/missing thread
state is harmless; repeated cursors still use the existing bounded scan guard.
This does not create request-generation IDs for retries using the exact same
cursor. No automatic retry path was introduced. Native and connected fullscreen
gates remain pending; this is a prerequisite rather than a feature score.
