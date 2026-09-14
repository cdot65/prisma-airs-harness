//! Relocate only canonical, binding-owned authentication helpers during upgrades.
use super::airs_credentials;
use super::airs_environment;
use super::airs_mcp;
use anyhow::Context;
use std::io::Read;
use std::path::Path;

pub(super) fn read(path: &Path, limit: u64) -> anyhow::Result<Vec<u8>> {
    let metadata = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(metadata.is_file(), "helper metadata must be a regular file");
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        anyhow::ensure!(
            metadata.file_attributes() & 0x400 == 0,
            "helper metadata cannot be a reparse point"
        );
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        anyhow::ensure!(
            metadata.file_attributes() & 0x400 == 0,
            "opened helper metadata cannot be a reparse point"
        );
    }
    anyhow::ensure!(
        metadata.is_file() && metadata.len() <= limit,
        "helper metadata exceeds its regular-file limit"
    );
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    anyhow::ensure!(
        bytes.len() as u64 <= limit,
        "helper metadata exceeds its read limit"
    );
    Ok(bytes)
}

pub(super) fn owned_executable(path: &Path) -> bool {
    path.is_absolute()
        && matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("airs-harness" | "airs-harness.exe" | "airs-terminal" | "airs-terminal.exe")
        )
}

/// Normalize only helper execution location; all destinations and tool policies remain hashed.
pub(super) fn mcp_revision(home: &Path, config: &toml::Value) -> anyhow::Result<Option<String>> {
    let Some(servers) = config.get("mcp_servers").and_then(toml::Value::as_table) else {
        return Ok(None);
    };
    if servers.is_empty() {
        return Ok(None);
    }
    // Built-in OAuth servers use Codex's mutable, endpoint-bound credential store.
    // Keep legacy credential helpers pinned without making adding/removing a native
    // OAuth server invalidate the inference identity or rewrite existing history.
    // Dynamic registration (including the gateway) has no explicit oauth.client_id.
    let mut normalized = servers.clone();
    normalized.retain(|_, server| {
        let native_oauth = server
            .get("auth")
            .is_none_or(|auth| auth.as_str() == Some("oauth"))
            && server
                .get("url")
                .and_then(toml::Value::as_str)
                .is_some_and(|url| url.starts_with("https://"))
            && [
                "http_headers_helper",
                "http_headers",
                "env_http_headers",
                "bearer_token_env_var",
                "command",
            ]
            .iter()
            .all(|key| server.get(key).is_none());
        !native_oauth
    });
    if normalized.is_empty() {
        return Ok(None);
    }
    let raw = toml::to_string(&normalized)?;
    let mut owned = false;
    for (name, server) in servers {
        if !normalized.contains_key(name) {
            continue;
        }
        if let Some(binding) = airs_mcp::relocation::owned(home, name, server)? {
            // A typed value cannot collide with a valid raw shell-command string.
            normalized
                .get_mut(name)
                .and_then(toml::Value::as_table_mut)
                .context("invalid MCP configuration")?
                .insert(
                    "http_headers_helper".into(),
                    serde_json::from_value(serde_json::json!({
                        "airs_managed_helper_revision_v1": {"home": home, "binding": binding.id}
                    }))?,
                );
            owned = true;
        }
    }
    Ok(Some(airs_credentials::fingerprint(&if owned {
        format!(
            "airs-managed-mcp-revision-v1\0{}",
            toml::to_string(&normalized)?
        )
    } else {
        raw
    })))
}

/// Validate history first, then atomically rewrite owned paths under one configuration lock.
pub(super) fn refresh(home: &Path) -> anyhow::Result<()> {
    let executable = std::env::current_exe()?;
    let _lock = airs_environment::lock(home)?;
    super::airs_session_binding::validate_locked(home)?;
    rewrite_locked(home, &executable)
}

pub(super) fn rewrite_locked(home: &Path, executable: &Path) -> anyhow::Result<()> {
    let input = String::from_utf8(read(&home.join("config.toml"), 1024 * 1024)?)?;
    let mut config: toml::Value = toml::from_str(&input)?;
    let previous = config.clone();
    if let Some(auth) = config
        .get_mut("model_providers")
        .and_then(|v| v.get_mut("airs"))
        .and_then(|v| v.get_mut("auth"))
    {
        let command = auth.get("command").and_then(toml::Value::as_str);
        if let Some(command) = command.filter(|value| owned_executable(Path::new(value)))
            && home.join("credential-binding.json").exists()
        {
            let binding = airs_credentials::parse_binding(&read(
                &home.join("credential-binding.json"),
                16_384,
            )?)?;
            anyhow::ensure!(
                binding.schema_version == 1
                    && binding.gateway_url == airs_environment::gateway(home)?,
                "invalid managed inference binding"
            );
            let expected: toml::Value = serde_json::from_value(serde_json::json!({
                "command": command, "args": ["credential", "--home", home, "--binding", binding.id],
                "timeout_ms": 60000, "refresh_interval_ms": 1000, "cwd": home
            }))?;
            if *auth == expected {
                auth.as_table_mut()
                    .context("invalid helper configuration")?
                    .insert(
                        "command".into(),
                        toml::Value::String(
                            executable
                                .to_str()
                                .context("executable path is not UTF-8")?
                                .to_owned(),
                        ),
                    );
            }
        }
    }
    if let Some(servers) = config
        .get_mut("mcp_servers")
        .and_then(toml::Value::as_table_mut)
    {
        for (name, server) in servers {
            if let Some(binding) = airs_mcp::relocation::owned(home, name, server)? {
                server.as_table_mut().context("invalid MCP server")?.insert(
                    "http_headers_helper".into(),
                    toml::Value::String(airs_mcp::helper_command(executable, home, binding.id)?),
                );
            }
        }
    }
    if config != previous {
        airs_environment::atomic_write(
            &home.join("config.toml"),
            toml::to_string_pretty(&config)?.as_bytes(),
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "airs_helper_relocation_tests.rs"]
mod tests;
