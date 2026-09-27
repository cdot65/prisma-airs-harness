# Background interaction details and compact session tips

Adapted321c50fc2c terminal bookkeeping and compact session delegation. Polling/stdin notices stay available in rich transcript and raw history; normal chat relies on actual command output and background status. Transcript rows now preserve input sources separately from indentation. Compact session cards delegate to child cells, and optional tips remain in detailed history.

365 focused checks passed with zero retries, including exec-flow ordering, current AIRS recap behavior, session branding and a direct chat/transcript/raw input boundary. Four directly affected history tests now exercise transcript rendering; ordering tests use AIRS's existing transcript-drain helper. No upstream timestamp helper or alternate recap product layout was imported. All existing snapshots pass.

Scoped lint and formatting passed. This is still a fullscreen prerequisite, not an independently scored product feature. Connected selection/replay/native and release checks remain pending; no gateway/authentication or model-context behavior changed.
