//! Harness-wide user interface preferences shared by every AIRS environment.
//!
//! `<application home>/settings.toml` (normally `~/.airs-harness/settings.toml`) sits
//! just above the packaged defaults, so each environment's own `config.toml`, project
//! configuration and command-line overrides still take precedence. It accepts only
//! presentation settings: gateway, credential, model, MCP and sandbox configuration
//! stay bound to the selected environment.

use super::layer_io;
use crate::ConfigLayerSource;
use crate::merge::merge_toml_values;
use crate::state::ConfigLayerEntry;
use codex_file_system::ExecutorFileSystem;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::io;
use toml::Value as TomlValue;

/// Top-level keys that only change how the terminal presents a session.
pub(super) const ALLOWED_KEYS: &[&str] = &[
    "tui",
    "file_opener",
    "hide_agent_reasoning",
    "show_raw_agent_reasoning",
    "disable_paste_burst",
    "notice",
    "history",
];

/// Harness defaults that differ from upstream Codex. Users override them in settings.toml.
const HARNESS_DEFAULTS: &str = "[tui]\nfullscreen_transcript = true\n";

pub(super) fn validate(settings: &TomlValue, path: &AbsolutePathBuf) -> io::Result<()> {
    let table = settings.as_table().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} must be a TOML table", path.as_path().display()),
        )
    })?;
    let mut rejected: Vec<&str> = table
        .keys()
        .map(String::as_str)
        .filter(|key| !ALLOWED_KEYS.contains(key))
        .collect();
    if rejected.is_empty() {
        return Ok(());
    }
    rejected.sort_unstable();
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "{} accepts only interface settings ({}); move {} to an environment's config.toml",
            path.as_path().display(),
            ALLOWED_KEYS.join(", "),
            rejected.join(", ")
        ),
    ))
}

/// Harness defaults merged with the user's settings file, or None outside the harness.
pub(super) async fn layer(
    fs: &dyn ExecutorFileSystem,
    strict_config: bool,
) -> io::Result<Option<ConfigLayerEntry>> {
    let (Some(root), Some(path)) = (
        codex_utils_home_dir::airs_application_home(),
        codex_utils_home_dir::airs_settings_path(),
    ) else {
        return Ok(None);
    };
    let mut merged: TomlValue = toml::from_str(HARNESS_DEFAULTS).map_err(io::Error::other)?;
    if let Some(settings) =
        layer_io::read_config_from_path(fs, &path, /*log_missing_as_info*/ true, strict_config)
            .await?
    {
        validate(&settings, &path)?;
        merge_toml_values(&mut merged, &settings);
    }
    Ok(Some(ConfigLayerEntry::new(
        ConfigLayerSource::PackagedDefaults { file: path },
        super::resolve_relative_paths_in_config_toml(merged, root.as_path())?,
    )))
}

#[cfg(test)]
#[path = "airs_settings_tests.rs"]
mod tests;
