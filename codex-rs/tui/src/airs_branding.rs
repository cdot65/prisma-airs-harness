//! Standalone product presentation without renaming upstream engine internals.
use crate::legacy_core::config::Config;
use codex_protocol::models::PermissionProfile;
use codex_utils_home_dir::AIRS_TERMINAL_VERSION;
use codex_utils_home_dir::is_airs_terminal;
use codex_utils_sandbox_summary::summarize_permission_profile;
use std::path::Path;

pub(crate) async fn logout(home: &Path) -> color_eyre::Result<()> {
    // Linux keeps this inode executable after an atomic binary replacement.
    // current_exe() may instead return a path ending in " (deleted)".
    #[cfg(target_os = "linux")]
    let executable = std::path::PathBuf::from("/proc/self/exe");
    #[cfg(not(target_os = "linux"))]
    let executable = std::env::current_exe()?;
    let output = tokio::process::Command::new(executable)
        .arg("logout")
        // Bind to this running session even if another process selected a new default.
        .env("AIRS_TERMINAL_HOME", home)
        .output()
        .await?;
    if !output.status.success() {
        return Err(color_eyre::eyre::eyre!(
            "{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub(crate) fn product_name() -> &'static str {
    if is_airs_terminal() {
        "Prisma AIRS Terminal"
    } else {
        "OpenAI Codex"
    }
}

pub(crate) fn version(upstream: &'static str) -> &'static str {
    if is_airs_terminal() {
        AIRS_TERMINAL_VERSION
    } else {
        upstream
    }
}

pub(crate) fn placeholder() -> &'static str {
    if is_airs_terminal() {
        "Ask AIRS Terminal to work on your project"
    } else {
        "Ask Codex to do anything"
    }
}

pub(crate) fn environment_name(home: &Path) -> Option<String> {
    home.parent()
        .and_then(|p| p.parent())
        .and_then(|root| std::fs::read(root.join("environments.json")).ok())
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|registry| {
            let id = home.file_name()?.to_str()?;
            registry
                .get("environments")?
                .as_object()?
                .iter()
                .find(|(_, value)| value.get("id").and_then(serde_json::Value::as_str) == Some(id))
                .map(|(name, _)| name.clone())
        })
}

#[derive(Debug)]
pub(crate) struct HeaderContext {
    pub environment: String,
    pub gateway: String,
    pub identity: String,
    pub permissions: String,
}

impl HeaderContext {
    pub(crate) fn from_config(config: &Config, permissions: &PermissionProfile) -> Self {
        let home = config.codex_home.as_path();
        let environment = environment_name(home).unwrap_or_else(|| "default".into());
        Self {
            environment,
            gateway: config
                .model_provider
                .base_url
                .clone()
                .unwrap_or_else(|| "unconfigured".into()),
            identity: "workspace credential".into(),
            permissions: summarize_permission_profile(
                permissions,
                &config.cwd,
                &config.effective_workspace_roots(),
            ),
        }
    }

    pub(crate) fn rows(&self) -> [(&'static str, &str); 4] {
        [
            ("environment:", &self.environment),
            ("gateway:", &self.gateway),
            ("identity:", &self.identity),
            ("permissions:", &self.permissions),
        ]
    }
}
