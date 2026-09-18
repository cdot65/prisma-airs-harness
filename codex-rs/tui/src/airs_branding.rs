//! Standalone product presentation without renaming upstream engine internals.
use crate::legacy_core::config::Config;
use codex_protocol::models::PermissionProfile;
use codex_utils_home_dir::AIRS_HARNESS_VERSION;
use codex_utils_home_dir::is_airs_harness;
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
        .env("AIRS_HARNESS_HOME", home)
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
    if is_airs_harness() {
        "Prisma AIRS Harness"
    } else {
        "OpenAI Codex"
    }
}

pub(crate) fn version(upstream: &'static str) -> &'static str {
    if is_airs_harness() {
        AIRS_HARNESS_VERSION
    } else {
        upstream
    }
}

pub(crate) fn placeholder() -> &'static str {
    if is_airs_harness() {
        "Ask AIRS Harness to work on your project"
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

/// Reopen the selected environment even if the saved default changes later.
pub(crate) fn command() -> Vec<String> {
    let environment = is_airs_harness()
        .then(|| codex_utils_home_dir::find_codex_home().ok())
        .flatten()
        .and_then(|home| environment_name(home.as_path()));
    command_for_environment(codex_utils_home_dir::command_name(), environment.as_deref())
}

pub(crate) fn command_for_environment(executable: &str, environment: Option<&str>) -> Vec<String> {
    let mut command = vec![executable.to_string()];
    if let Some(environment) = environment {
        command.extend(["--environment".to_string(), environment.to_string()]);
    }
    command
}

#[derive(Debug)]
pub(crate) struct HeaderContext {
    pub environment: String,
    pub gateway: String,
    pub identity: String,
    pub permissions: String,
}

fn identity_label(home: &Path) -> String {
    let oidc = || -> Option<String> {
        let path = home.join("credential-binding.json");
        if std::fs::metadata(&path).ok()?.len() > 16_384 {
            return None;
        }
        let binding: serde_json::Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
        let source = binding.get("source")?;
        if source.get("kind")?.as_str()? != "oidc" {
            return None;
        }
        let identity = source.get("identity")?;
        let label = identity
            .get("display_name")
            .and_then(serde_json::Value::as_str)
            .or_else(|| identity.get("subject")?.as_str())?;
        if label.is_empty() || label.len() > 128 || label.chars().any(char::is_control) {
            return Some("OIDC user".into());
        }
        Some(format!("OIDC · {label}"))
    };
    oidc().unwrap_or_else(|| "workspace credential".into())
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
            identity: identity_label(home),
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
