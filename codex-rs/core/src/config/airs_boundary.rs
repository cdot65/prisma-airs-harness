//! Keep AIRS destinations and credential mechanisms in the selected user home.
use codex_config::config_toml::ConfigToml;
use codex_model_provider_info::ModelProviderInfo;
use std::path::Path;

pub(super) fn validate(home: &Path, effective: &ConfigToml) -> std::io::Result<ConfigToml> {
    let input = std::fs::read_to_string(home.join("config.toml")).map_err(|_| {
        invalid("cannot read AIRS environment configuration; run airs-terminal setup")
    })?;
    let user: ConfigToml =
        toml::from_str(&input).map_err(|_| invalid("invalid AIRS user configuration"))?;
    if effective.model_providers != user.model_providers
        || effective.model_catalog_json != user.model_catalog_json
        || effective.model_context_window != user.model_context_window
        || effective.mcp_servers != user.mcp_servers
        || effective.shell_environment_policy != user.shell_environment_policy
        || effective.allow_login_shell != user.allow_login_shell
    {
        return Err(invalid(
            "AIRS gateway, credential, capability, MCP and credential-exclusion settings must come from the selected environment; repository/profile/CLI overrides are not permitted",
        ));
    }
    Ok(user)
}

pub(super) fn validate_provider(
    user: &ConfigToml,
    provider: &ModelProviderInfo,
) -> std::io::Result<()> {
    if user.model_providers.get("airs") != Some(provider) {
        return Err(invalid(
            "selected AIRS provider does not match this environment's binding",
        ));
    }
    Ok(())
}

fn invalid(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}

#[cfg(test)]
#[path = "airs_boundary_tests.rs"]
mod tests;
