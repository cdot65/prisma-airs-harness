# Dynamic tool restoration and terminal control filtering

Adapted the dynamic-tool and terminal-output parts of4abcb8d1da. Loaded dynamic calls now retain arguments, complete output, duration and truthful failed/unavailable states. Compact rows reuse explicit decorative-prefix metadata; detailed headers retain full source and existing AIRS cyan styling. Retained shared state supports later live updates without claiming that live integration is complete here.

Terminal scrollback now filters untrusted control-containing graphemes before adding trusted semantic hyperlink sequences, matching ratatui's behavior. Tests compare linked/unlinked output against sanitized output while leaving the original history line unchanged.

385 focused checks passed with zero retries, including current ordering/recap/session regressions and upstream dynamic compact/full/raw snapshots. Scoped lint passed without warnings or fixes. Formatting passed. This is a bounded prerequisite, not a completed fullscreen feature or a new release. Persisted command/MCP/patch restoration and connected live/view/selection validation remain next.
