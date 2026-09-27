# Selection boundaries and export followup

Completes three small d7d9f2b9b5 application changes left out of stage27:
exclude the unconfirmed-copy notice from exported conversation markdown, and
exercise shortcut precedence and selection immediately after live output commits.
The only production change is the export notice filter. Tests use AIRS boxed
overlays and actual app-server/TUI dispatch, retaining draft/shortcut behavior.

All 21 focused key-chord/export tests passed with zero skips or retries
(1.111 seconds executing; 122.063 seconds including compilation). Stage27's
preceding complete TUI/config/features run remains 5,046 passes with six existing
skips; this followup does not claim a new complete-suite run.

Adversarial self-review checked that fixed selection keys bypass chord prefixes,
a committed stream is rendered exactly once before selection begins, and export
omits only the same transient status classes already excluded from markdown.
No credentials, gateway routing, model context, dependencies or schemas changed.
No whole-feature score, native Mac acceptance or new publication is claimed.
The connected fullscreen/search implementation and signed preview remain pending.

Scoped lint passed in 122.29 seconds with no warnings or automatic fixes.
Formatting passed in 18.10 seconds; unrelated Python formatting churn was
restored. Tests were not rerun solely after lint/format.
