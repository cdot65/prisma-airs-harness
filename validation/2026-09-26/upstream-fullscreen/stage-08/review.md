# Source-aware bounded tool previews — fullscreen prerequisite

Adapted the tool-output portion of `321c50fc2c`. Bounded wrapping retains original logical source ranges/styles and hyperlink destinations. Omission hints remain synthetic rows with no source or link. The existing command/MCP display callers project visible lines until the compact renderers adopt the richer interface. Existing hint wording and snapshots remain unchanged in this stage.

The first compile exposed an attempted borrowed-to-static MCP conversion. Kept the existing borrowed input path instead of allocating entire long lines before bounding them. The richer hyperlink-input method has an explicit temporary dead-code allowance until the following compact MCP stage; remove that allowance when its production consumer lands. The initial compile receipt is retained.

Focused history/exec/replay/tool-output tests:241/241 passed, zero retries. Added upstream metadata comparisons check original whitespace, styles, link mapping and omitted synthetic rows; combining-character limits and 100,000-line bounded iteration remain covered. No new snapshot acceptance or user-visible rendering change. Connected fullscreen/native/release gates remain pending; no standalone feature score.

Scoped config/core/TUI lint passed with zero warnings and no fixes. Formatting passed, with unrelated Python churn restored.
