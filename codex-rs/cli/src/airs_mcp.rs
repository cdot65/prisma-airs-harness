//! Explicit, endpoint-bound MCP workspace credentials, independent of inference.
use super::airs_credentials::file_token;
use super::airs_credentials::fingerprint;
use super::airs_environment;
use anyhow::Context;
use clap::Args;
use serde::Deserialize;
use serde::Serialize;
use std::io::IsTerminal;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Args)]
pub struct SetupArgs {
    /// Local server name, used by /mcp and mcp remove.
    #[arg(long)]
    pub name: String,
    /// Complete HTTPS endpoint of the AIRS MCP server.
    #[arg(long)]
    pub url: String,
    /// Existing owner-only file containing a separately authorized MCP API key.
    #[arg(long)]
    pub credential_file: PathBuf,
    /// Restrict the local tool catalog to these names. Repeat for more tools.
    #[arg(long)]
    pub tool: Vec<String>,
    /// Fail startup if this MCP server is unavailable.
    #[arg(long)]
    pub required: bool,
}

#[derive(Debug, Args)]
pub struct HelperArgs {
    #[arg(long)]
    pub home: PathBuf,
    #[arg(long)]
    pub binding: Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    schema_version: u32,
    id: Uuid,
    server: String,
    url: String,
    credential_file: PathBuf,
    credential_fingerprint: String,
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub fn setup(home: &Path, args: &SetupArgs) -> anyhow::Result<()> {
    anyhow::ensure!(
        !args.name.is_empty()
            && args.name.len() <= 64
            && args
                .name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_')),
        "MCP name must contain letters, digits, underscores or hyphens"
    );
    let url = url::Url::parse(&args.url).context("invalid MCP URL")?;
    anyhow::ensure!(
        url.scheme() == "https"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none(),
        "MCP requires HTTPS without URL credentials, query parameters or fragments"
    );
    let token = file_token(&args.credential_file)?;
    let _lock = airs_environment::lock(home)?;
    let mut config: toml::Value =
        toml::from_str(&std::fs::read_to_string(home.join("config.toml"))?)?;
    let servers = config
        .as_table_mut()
        .context("invalid configuration")?
        .entry("mcp_servers")
        .or_insert_with(|| toml::Value::Table(Default::default()))
        .as_table_mut()
        .context("invalid MCP configuration")?;
    anyhow::ensure!(
        !servers.contains_key(&args.name),
        "MCP server already exists; inspect it with mcp get and remove it before replacing its binding"
    );
    let binding = Binding {
        schema_version: 1,
        id: Uuid::new_v4(),
        server: args.name.clone(),
        url: url.to_string(),
        credential_file: args.credential_file.clone(),
        credential_fingerprint: fingerprint(&token),
    };
    let helper = format!(
        "{} mcp-credential --home {} --binding {}",
        shell_quote(
            std::env::current_exe()?
                .to_str()
                .context("executable path is not UTF-8")?
        ),
        shell_quote(home.to_str().context("state directory is not UTF-8")?),
        binding.id
    );
    let mut server = serde_json::json!({"url": binding.url, "http_headers_helper": helper,
        "required": args.required, "startup_timeout_sec": 30, "tool_timeout_sec": 60});
    if !args.tool.is_empty() {
        server["enabled_tools"] = serde_json::json!(args.tool);
    }
    servers.insert(args.name.clone(), serde_json::from_value(server)?);
    let directory = home.join("mcp-bindings");
    airs_environment::private_directory(&directory)?;
    airs_environment::atomic_write(
        &directory.join(format!("{}.json", binding.id)),
        &serde_json::to_vec_pretty(&binding)?,
    )?;
    airs_environment::atomic_write(
        &home.join("config.toml"),
        toml::to_string_pretty(&config)?.as_bytes(),
    )?;
    println!(
        "Configured MCP server {} at {} with a separate workspace credential.",
        args.name, binding.url
    );
    println!("Use mcp list or /mcp to inspect it. The gateway controls remote authorization.");
    Ok(())
}

pub fn helper(args: &HelperArgs) -> anyhow::Result<()> {
    anyhow::ensure!(
        !std::io::stdout().is_terminal(),
        "MCP credential helper may only write to a pipe"
    );
    anyhow::ensure!(args.home.is_absolute(), "state directory must be absolute");
    let _lock = airs_environment::lock(&args.home)?;
    anyhow::ensure!(
        !args.home.join("logged-out").exists(),
        "environment is logged out; run airs-terminal login"
    );
    let binding: Binding = serde_json::from_slice(&std::fs::read(
        args.home
            .join("mcp-bindings")
            .join(format!("{}.json", args.binding)),
    )?)?;
    anyhow::ensure!(
        binding.schema_version == 1 && binding.id == args.binding,
        "invalid MCP credential binding"
    );
    let config: toml::Value =
        toml::from_str(&std::fs::read_to_string(args.home.join("config.toml"))?)?;
    let destination = config
        .get("mcp_servers")
        .and_then(|v| v.get(&binding.server))
        .and_then(|v| v.get("url"))
        .and_then(toml::Value::as_str);
    anyhow::ensure!(
        destination == Some(binding.url.as_str()),
        "MCP destination changed or server was removed; configure a new binding"
    );
    let token = file_token(&binding.credential_file)?;
    anyhow::ensure!(
        fingerprint(&token) == binding.credential_fingerprint,
        "MCP credential identity changed; configure a new binding"
    );
    println!("{}", serde_json::json!({"x-portkey-api-key": token}));
    Ok(())
}

#[cfg(test)]
#[path = "airs_mcp_tests.rs"]
mod tests;
