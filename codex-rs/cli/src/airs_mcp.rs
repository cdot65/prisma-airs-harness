//! Explicit, endpoint-bound MCP workspace credentials, independent of inference.
use super::airs_credentials::Binding as IdentityBinding;
use super::airs_credentials::Source;
use super::airs_credentials::file_token;
use super::airs_credentials::fingerprint;
use super::airs_environment;
use super::airs_oidc;
use anyhow::Context;
use clap::Args;
use codex_airs_identity::IdentityConfig;
use serde::Deserialize;
use serde::Serialize;
use std::io::IsTerminal;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

#[path = "airs_mcp_relocation.rs"]
pub(super) mod relocation;

#[derive(Debug, Args)]
pub struct SetupArgs {
    /// Local server name, used by /mcp and mcp remove.
    #[arg(long)]
    pub name: String,
    /// Complete HTTPS endpoint of the AIRS MCP server.
    #[arg(long)]
    pub url: String,
    /// Existing owner-only file containing a separately authorized MCP API key.
    #[arg(
        long,
        required_unless_present = "issuer_url",
        conflicts_with = "issuer_url"
    )]
    pub credential_file: Option<PathBuf>,
    /// Issuer for the separate MCP resource login; must match the inference identity.
    #[arg(long, requires_all = ["oidc_client_id", "audience"])]
    pub issuer_url: Option<String>,
    #[arg(long, requires = "issuer_url")]
    pub oidc_client_id: Option<String>,
    #[arg(long, requires = "issuer_url")]
    pub audience: Option<String>,
    /// Direct OAuth resource URL, matching --url and --audience. Enables Bearer auth.
    #[arg(long, requires = "issuer_url", conflicts_with = "device_auth")]
    pub resource: Option<String>,
    /// Requested OAuth permission. Repeat for each scope.
    #[arg(long, requires = "resource")]
    pub scope: Vec<String>,
    /// Use a device code instead of a local browser callback.
    #[arg(long, requires = "issuer_url")]
    pub device_auth: bool,
    /// Print the login URL without opening the desktop browser.
    #[arg(long, requires = "issuer_url", conflicts_with = "device_auth")]
    pub no_browser: bool,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    credential_file: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    oidc: Option<IdentityBinding>,
    #[serde(default)]
    logged_out: bool,
    credential_fingerprint: String,
}

#[cfg(not(windows))]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub async fn setup(home: &Path, args: &SetupArgs) -> anyhow::Result<()> {
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
    anyhow::ensure!(
        args.resource.as_ref().is_none_or(
            |resource| resource == url.as_str() && args.audience.as_ref() == Some(resource)
        ),
        "direct OAuth resource must match the MCP URL and audience"
    );
    let _lock = airs_environment::lock(home)?;
    let mut config: toml::Value =
        toml::from_str(&std::fs::read_to_string(home.join("config.toml"))?)?;
    if config
        .get("mcp_servers")
        .and_then(|servers| servers.get(&args.name))
        .is_some()
        && home.join("session-binding.json").try_exists()?
    {
        super::airs_session_binding::validate_locked(home)?;
        super::airs_helper_relocation::rewrite_locked(home, &std::env::current_exe()?)?;
        config = toml::from_str(&std::fs::read_to_string(home.join("config.toml"))?)?;
    }
    let servers = config
        .as_table_mut()
        .context("invalid configuration")?
        .entry("mcp_servers")
        .or_insert_with(|| toml::Value::Table(Default::default()))
        .as_table_mut()
        .context("invalid MCP configuration")?;
    // Reauthentication preserves the helper/binding ID, keeping existing history stable.
    let existing = if servers.contains_key(&args.name) {
        let directory = home.join("mcp-bindings");
        let mut found = None;
        for entry in std::fs::read_dir(&directory)? {
            let previous = read_binding(&entry?.path())?;
            if previous.server == args.name {
                anyhow::ensure!(
                    found.is_none(),
                    "ambiguous MCP bindings; use a new environment"
                );
                found = Some(previous);
            }
        }
        let previous =
            found.context("MCP server has no managed binding; remove it before setup")?;
        anyhow::ensure!(
            previous.oidc.is_some() && args.issuer_url.is_some() && previous.url == url.as_str(),
            "MCP server already exists; only the same OIDC resource can reauthenticate in place"
        );
        Some(previous)
    } else {
        None
    };
    let mut binding = Binding {
        schema_version: 1,
        id: existing.as_ref().map_or_else(Uuid::new_v4, |b| b.id),
        server: args.name.clone(),
        url: url.to_string(),
        credential_file: args.credential_file.clone(),
        credential_fingerprint: String::new(),
        oidc: None,
        logged_out: false,
    };
    let tokens = if let Some(issuer) = &args.issuer_url {
        let inference = super::airs_credentials::read_binding(home)?;
        anyhow::ensure!(
            !home.join("logged-out").exists(),
            "sign in to inference before MCP"
        );
        let Some(Source::Oidc { identity }) = &inference.source else {
            anyhow::bail!("MCP OIDC requires an OIDC inference identity in this environment");
        };
        let config = IdentityConfig {
            resource: args.resource.clone(),
            scopes: {
                let mut scopes = args.scope.clone();
                scopes.sort();
                scopes.dedup();
                scopes
            },
            issuer: issuer.clone(),
            client_id: args
                .oidc_client_id
                .clone()
                .context("missing MCP OIDC client")?,
            audience: args
                .audience
                .clone()
                .context("missing MCP resource audience")?,
        };
        anyhow::ensure!(
            config.issuer == identity.config.issuer
                && config.client_id != identity.config.client_id
                && config.audience != identity.config.audience,
            "MCP requires the same issuer and a distinct client and resource audience"
        );
        let flow = if args.device_auth {
            airs_oidc::LoginFlow::Device
        } else if args.no_browser {
            airs_oidc::LoginFlow::BrowserManual
        } else {
            airs_oidc::LoginFlow::Browser
        };
        let tokens = airs_oidc::authenticate(config, flow).await?;
        anyhow::ensure!(
            tokens.identity.subject == identity.subject,
            "MCP login belongs to a different user; authenticate with the inference account"
        );
        binding.credential_fingerprint = airs_oidc::fingerprint(&binding.url, &tokens.identity)?;
        binding.oidc = Some(IdentityBinding {
            schema_version: 1,
            id: binding.id,
            gateway_url: binding.url.clone(),
            credential_fingerprint: binding.credential_fingerprint.clone(),
            source: Some(Source::Oidc {
                identity: tokens.identity.clone(),
            }),
        });
        Some(tokens)
    } else {
        binding.credential_fingerprint = fingerprint(&file_token(
            args.credential_file
                .as_deref()
                .context("MCP requires a credential file or OIDC login")?,
        )?);
        None
    };
    if let Some(previous) = &existing {
        anyhow::ensure!(
            previous.credential_fingerprint == binding.credential_fingerprint,
            "MCP identity changed; create a new environment for this resource identity"
        );
    }
    let helper = helper_command(&std::env::current_exe()?, home, binding.id)?;
    let mut server = serde_json::json!({"url": binding.url, "http_headers_helper": helper,
        "required": args.required, "startup_timeout_sec": 90, "tool_timeout_sec": 60});
    if !args.tool.is_empty() {
        server["enabled_tools"] = serde_json::json!(args.tool);
    }
    let server: toml::Value = serde_json::from_value(server)?;
    if existing.is_some() {
        anyhow::ensure!(
            servers.get(&args.name) == Some(&server),
            "MCP configuration changed; repeat the original setup options or use a new environment"
        );
    }
    if let (Some(identity), Some(tokens)) = (&binding.oidc, tokens) {
        airs_oidc::store_tokens(identity, tokens)?;
    }
    servers.insert(args.name.clone(), server);
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
        "Configured MCP server {} at {} with a separate resource credential.",
        args.name, binding.url
    );
    println!("Use mcp list or /mcp to inspect it. The MCP resource controls remote authorization.");
    Ok(())
}

pub async fn helper(args: &HelperArgs) -> anyhow::Result<()> {
    anyhow::ensure!(
        !std::io::stdout().is_terminal(),
        "MCP credential helper may only write to a pipe"
    );
    anyhow::ensure!(args.home.is_absolute(), "state directory must be absolute");
    let session = codex_utils_home_dir::airs_session::AirsSessionGuard::capture(&args.home)?;
    let _lock = airs_environment::lock(&args.home)?;
    anyhow::ensure!(
        !args.home.join("logged-out").exists(),
        "environment is logged out; run airs-harness login"
    );
    let binding = read_binding(
        &args
            .home
            .join("mcp-bindings")
            .join(format!("{}.json", args.binding)),
    )?;
    anyhow::ensure!(
        binding.schema_version == 1 && binding.id == args.binding && !binding.logged_out,
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
    let token = if let Some(identity) = &binding.oidc {
        anyhow::ensure!(
            identity.id == binding.id
                && identity.gateway_url == binding.url
                && identity.credential_fingerprint == binding.credential_fingerprint,
            "MCP identity binding is inconsistent"
        );
        let inference = super::airs_credentials::read_binding(&args.home)?;
        let (Some(Source::Oidc { identity: user }), Some(Source::Oidc { identity: mcp })) =
            (&inference.source, &identity.source)
        else {
            anyhow::bail!("OIDC identity missing; sign in again");
        };
        anyhow::ensure!(
            user.subject == mcp.subject
                && user.config.issuer == mcp.config.issuer
                && user.config.audience != mcp.config.audience
                && user.config.client_id != mcp.config.client_id,
            "MCP identity no longer matches the signed-in user and resource"
        );
        airs_oidc::credential(identity).await?
    } else {
        let token = file_token(
            binding
                .credential_file
                .as_deref()
                .context("missing MCP credential source")?,
        )?;
        anyhow::ensure!(
            fingerprint(&token) == binding.credential_fingerprint,
            "MCP credential identity changed; configure a new binding"
        );
        token
    };
    session.check()?;
    let direct = binding.oidc.as_ref().and_then(|identity| identity.source.as_ref())
        .is_some_and(|source| matches!(source, Source::Oidc { identity } if identity.config.resource.as_deref() == Some(binding.url.as_str())));
    let headers = if direct {
        serde_json::json!({"Authorization": format!("Bearer {token}")})
    } else {
        serde_json::json!({"x-portkey-api-key": token})
    };
    println!("{headers}");
    Ok(())
}

fn read_binding(path: &Path) -> anyhow::Result<Binding> {
    anyhow::ensure!(
        std::fs::metadata(path)?.len() <= 16_384,
        "MCP binding is too large"
    );
    let binding: Binding = serde_json::from_slice(&std::fs::read(path)?)?;
    anyhow::ensure!(
        binding.schema_version == 1
            && (binding.oidc.is_some() != binding.credential_file.is_some()),
        "invalid MCP credential source"
    );
    Ok(binding)
}

pub(super) fn helper_command(executable: &Path, home: &Path, id: Uuid) -> anyhow::Result<String> {
    let executable = executable
        .to_str()
        .context("executable path is not UTF-8")?;
    let home = home.to_str().context("state directory is not UTF-8")?;
    #[cfg(not(windows))]
    {
        Ok(format!(
            "{} mcp-credential --home {} --binding {id}",
            shell_quote(executable),
            shell_quote(home)
        ))
    }
    #[cfg(windows)]
    {
        use base64::Engine;
        // Encoded PowerShell bypasses cmd.exe's percent expansion and quoting of
        // executable paths. Only nonsecret paths/UUID are encoded, never tokens.
        let quote = |s: &str| format!("'{}'", s.replace('\'', "''"));
        let script = format!(
            "& {} mcp-credential --home {} --binding '{id}'; exit $LASTEXITCODE",
            quote(executable),
            quote(home)
        );
        let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
        Ok(format!(
            "powershell.exe -NoProfile -NonInteractive -EncodedCommand {}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        ))
    }
}

/// Disable every local MCP binding before attempting remote revocation. Caller holds the environment lock.
pub(super) async fn logout(home: &Path) -> anyhow::Result<()> {
    use codex_keyring_store::KeyringStore;
    let directory = home.join("mcp-bindings");
    if !directory.exists() {
        return Ok(());
    }
    let mut failures = false;
    let mut revoke = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        let mut binding = read_binding(&path)?;
        binding.logged_out = binding.oidc.is_some();
        airs_environment::atomic_write(&path, &serde_json::to_vec_pretty(&binding)?)?;
        if let Some(identity) = &binding.oidc {
            if let Ok(tokens) = airs_oidc::load_active(identity) {
                revoke.push(tokens);
            }
            failures |= codex_airs_identity::CredentialStore
                .delete(super::airs_credentials::SERVICE, &identity.id.to_string())
                .is_err();
        }
    }
    for tokens in revoke {
        match codex_airs_identity::Provider::discover(tokens.identity.config.clone()).await {
            Ok(provider) => failures |= provider.revoke(&tokens).await.is_err(),
            Err(_) => failures = true,
        }
    }
    anyhow::ensure!(
        !failures,
        "MCP disabled locally; credential deletion or issuer revocation could not be confirmed"
    );
    Ok(())
}

#[cfg(test)]
#[path = "airs_mcp_tests.rs"]
mod tests;
