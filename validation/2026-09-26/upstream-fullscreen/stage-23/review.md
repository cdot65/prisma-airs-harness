# Page projection and header placement

Adapted the presentation portion of 6ba6b73657: project each persisted turn
separately, split at hidden review prompts before rendering, retain native-history
ownership after removing a hidden cell, and prepend after either saved session
information or the initial session header. The existing Home-to-start, refill,
backtrack selection, overlap deduplication and live-turn merge paths remain.

The application orchestration is separated into projection, hidden-prompt
reconciliation, insertion and finishing methods. These are upstream-sized local
helpers; no new public protocol or core behavior is introduced. The turn-boundary
helper uses a set of final item IDs instead of upstream's completion-metadata map.
AIRS keeps its existing work dividers and runtime metrics. The unrelated upstream
completion-timestamp product change is not required by the selected fullscreen
feature and is not imported solely to satisfy an internal method dependency.
Historical timestamps are not invented, and replay does not trigger live actions.

Adversarial cases include identical visible/hidden prompts, groups on either side
of a hidden prompt, adjacent compatible commands across successful/failed/
interrupted/running turns, partial pages, overlapping turns arriving while newer
live state exists, and initial header placement with and without the overlay.
The reviewed compact snapshot shows three separate exploration groups and a review
marker, without exposing the internal prompt. Existing real app-server tests
cover multi-page Home navigation, stale completions and cross-page review markers.

Earlier test attempts and final validation will be retained below. The first
behavioral run had only the expected missing new snapshot. A later small map-to-set
cleanup exposed a type inference error, corrected by using the default set hasher.

This is a rendering prerequisite, not a completed fullscreen score or release.
Cross-page exploration/detail joins attach to the owned-view integration later;
the generic helpers are not imported unused at this stage. Gateway/OAuth/Jev and
model-context behavior are unchanged.

## Validation result

Final focused run: 132/132 passed, zero skips/failures/retries. All four turn
statuses and the reviewed snapshot passed, including the strengthened hidden-only
boundary and both initial-header views. Scoped lint completed in 77.52 seconds
with no warnings or fixes; formatting passed in 18.70 seconds. Unrelated formatter
churn was restored. No tests were rerun solely after lint/format. No native or
whole-feature passing score is claimed by these focused checks.
