//! TUI content rendering preferences, independent of animation effects.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

/// Optional rich renderers. Disabled renderers retain their textual markers.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, JsonSchema)]
#[serde(default)]
#[schemars(deny_unknown_fields)]
pub struct TuiRendering {
    /// Render Markdown bullets and task checkboxes using Unicode symbols.
    pub lists: bool,
    /// Render supported math expressions using bounded Unicode layouts.
    pub math: bool,
}

impl Default for TuiRendering {
    fn default() -> Self {
        Self {
            lists: true,
            math: true,
        }
    }
}
