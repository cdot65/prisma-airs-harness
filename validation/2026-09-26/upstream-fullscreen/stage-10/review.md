# Source-aware commands and compact disclosure — fullscreen prerequisite

Adapted the command portion of `321c50fc2c`: separate compact renderer, stable exec activity identities, retained expanded output/source ranges, user-shell truncation accounting and older-page prepend support. Removed the prior ActivityGroup compatibility methods once real command consumers adopted the richer source/detail interface. Existing cyan styling and transcript hint wording remain intact.

Review found upstream counted search exit1 as a failure in the compact group header although the existing display treats it neutrally (it can mean no matches). The count now follows that existing contract; a new 0/1/2 status regression/snapshot verifies it. Preserved the existing long-URL transcript-height regression until the later layout-cache stage actually supersedes it. The initial 294/296 pass run had two expected snapshots (new search regression and singular 1 line); both were inspected and accepted. Final focused suite:297/297 passed, zero retries.

This coherent conversion changes existing Line projections to HyperlinkLine and isolates new compact logic in a 113-line module. Most growth in the already-large render module is mechanical metadata propagation; retaining upstream structure here avoids an unrelated refactor. This is an intermediate prerequisite, not a standalone feature score. Connected native/fullscreen/release checks remain pending.

Open connected-gate review item: upstream clipped compact previews can fall back to plain source containing synthetic gutters. Add collapsed command/MCP/patch copy tests and retain explicit prefix-aware metadata without revealing clipped hidden text before scoring the complete selection feature. Details are retained in the implementation checkpoint; current compact display is not yet enabled as a product mode.

Scoped config/core/TUI lint passed without warnings or fixes. Formatting passed, with unrelated Python churn restored.
