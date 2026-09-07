//! Endpoint-bound workspace credentials. Secret values never enter configuration.
use super::airs_environment;
use anyhow::Context;
use codex_keyring_store::DefaultKeyringStore;
use codex_keyring_store::KeyringStore;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use std::io::IsTerminal;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

// Stable credential namespace retained across the alpha.8 product rename.
// Changing it would orphan existing OS-store refresh tokens.
pub(super) const SERVICE: &str = "io.cdot.airs-terminal";

pub(super) fn credential_store_error() -> anyhow::Error {
    let recovery = if cfg!(target_os = "linux") {
        "Linux requires an unlocked Secret Service keyring on the current session D-Bus. Return to the shell where you unlocked the keyring. On a headless host, start `dbus-run-session -- bash`, unlock Secret Service with your existing keyring password, and run login/resume inside that same shell. See the README's Keycloak sign-in instructions."
    } else if cfg!(target_os = "macos") {
        "Unlock your login keychain in Keychain Access and allow AIRS Harness to access its credential item, then retry from the same macOS user account."
    } else if cfg!(windows) {
        "Run AIRS Harness from the same signed-in Windows user account and ensure Windows Credential Manager is available, then retry."
    } else {
        "Unlock the native credential store for your current user session, then retry."
    };
    anyhow::anyhow!("OS credential store unavailable. {recovery} No plaintext fallback is used.")
}

#[derive(Debug, Default, clap::Args)]
pub struct LoginArgs {
    /// HTTPS issuer for a public-client OIDC login.
    #[arg(long, requires_all = ["oidc_client_id", "audience"], conflicts_with_all = ["credential_file", "credential_env", "with_api_key"])]
    pub issuer_url: Option<String>,
    #[arg(long, requires = "issuer_url")]
    pub oidc_client_id: Option<String>,
    /// Resource audience required in the signed gateway access token.
    #[arg(long, requires = "issuer_url")]
    pub audience: Option<String>,
    /// Reference an existing owner-only credential file (explicit headless mode).
    #[arg(long, conflicts_with = "credential_env")]
    pub credential_file: Option<PathBuf>,
    /// Reference a credential environment variable; nothing secret is persisted.
    #[arg(long)]
    pub credential_env: Option<String>,
}

#[derive(Debug, clap::Args)]
pub struct HelperArgs {
    #[arg(long)]
    pub home: PathBuf,
    #[arg(long)]
    pub binding: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum Source {
    File {
        path: PathBuf,
    },
    Environment {
        variable: String,
    },
    Keyring,
    Oidc {
        identity: codex_airs_identity::Identity,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Binding {
    pub(super) schema_version: u32,
    pub(super) id: Uuid,
    pub(super) gateway_url: String,
    pub(super) credential_fingerprint: String,
    pub(super) source: Option<Source>,
}

pub(super) fn fingerprint(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn validate_token(token: &str) -> anyhow::Result<&str> {
    let token = token.trim();
    anyhow::ensure!(
        !token.is_empty() && token.len() <= 16_384 && token.bytes().all(|c| c.is_ascii_graphic()),
        "credential must be a nonempty token without whitespace or control characters"
    );
    Ok(token)
}

pub(super) fn file_token(path: &Path) -> anyhow::Result<String> {
    anyhow::ensure!(
        path.is_absolute(),
        "credential file must use an absolute path"
    );
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path).context("cannot open credential file")?;
    let metadata = file.metadata()?;
    anyhow::ensure!(
        metadata.is_file() && metadata.len() <= 16_384,
        "credential file must be a regular file of at most 16 KiB"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        anyhow::ensure!(
            metadata.uid() == unsafe { libc::geteuid() } && metadata.mode() & 0o077 == 0,
            "credential file must be owned by this user and inaccessible to group/others; use chmod 600"
        );
    }
    let mut token = String::new();
    file.take(16_385)
        .read_to_string(&mut token)
        .context("credential file is not valid text")?;
    Ok(validate_token(&token)?.to_owned())
}

pub(super) fn read_binding(home: &Path) -> anyhow::Result<Binding> {
    let binding: Binding = serde_json::from_slice(
        &std::fs::read(home.join("credential-binding.json"))
            .context("no credential binding; run airs-harness login")?,
    )?;
    anyhow::ensure!(
        binding.schema_version == 1,
        "unsupported credential binding schema"
    );
    anyhow::ensure!(
        airs_environment::gateway(home)? == binding.gateway_url,
        "gateway changed; create a new environment and authenticate explicitly"
    );
    Ok(binding)
}

fn resolve(binding: &Binding) -> anyhow::Result<String> {
    let token = match binding
        .source
        .as_ref()
        .context("logged out; run airs-harness login")?
    {
        Source::File { path } => file_token(path)?,
        Source::Environment { variable } => {
            std::env::var(variable).context("credential environment variable is unavailable")?
        }
        Source::Keyring => DefaultKeyringStore
            .load(SERVICE, &binding.id.to_string())
            .map_err(|_| credential_store_error())?
            .context("credential is missing from the OS store; run login")?,
        Source::Oidc { .. } => anyhow::bail!("OIDC tokens require the identity credential helper"),
    };
    let token = validate_token(&token)?.to_owned();
    anyhow::ensure!(
        fingerprint(&token) == binding.credential_fingerprint,
        "credential identity changed; create a new environment to keep histories separate"
    );
    Ok(token)
}

pub fn login(home: &Path, args: &LoginArgs, stdin_key: bool) -> anyhow::Result<()> {
    let _lock = airs_environment::lock(home)?;
    anyhow::ensure!(
        !(stdin_key && (args.credential_file.is_some() || args.credential_env.is_some())),
        "choose exactly one credential source"
    );
    let (source, token) = if let Some(path) = &args.credential_file {
        let token = file_token(path)?;
        (Source::File { path: path.clone() }, token)
    } else if let Some(variable) = &args.credential_env {
        anyhow::ensure!(
            !variable.is_empty()
                && variable.bytes().enumerate().all(|(i, c)| c == b'_'
                    || c.is_ascii_alphabetic()
                    || (i > 0 && c.is_ascii_digit())),
            "credential-env must be a variable name"
        );
        let token =
            std::env::var(variable).context("credential environment variable is unavailable")?;
        (
            Source::Environment {
                variable: variable.clone(),
            },
            validate_token(&token)?.to_owned(),
        )
    } else if stdin_key {
        anyhow::ensure!(
            !std::io::stdin().is_terminal(),
            "pipe the key on stdin; never place it in arguments"
        );
        let mut token = String::new();
        std::io::stdin().take(16_385).read_to_string(&mut token)?;
        (Source::Keyring, validate_token(&token)?.to_owned())
    } else {
        anyhow::bail!(
            "use --credential-file PATH, --credential-env NAME, or pipe a key to login --with-api-key for OS credential storage"
        );
    };
    let gateway_url = airs_environment::gateway(home)?;
    let credential_fingerprint = fingerprint(&token);
    let existing = if home.join("credential-binding.json").exists() {
        Some(read_binding(home)?)
    } else {
        None
    };
    if let Some(previous) = &existing {
        anyhow::ensure!(
            previous.credential_fingerprint == credential_fingerprint,
            "a different credential requires a new environment; existing sessions must not change identity"
        );
    }
    let binding = Binding {
        schema_version: 1,
        id: existing.map_or_else(Uuid::new_v4, |v| v.id),
        gateway_url,
        credential_fingerprint,
        source: Some(source),
    };
    if matches!(binding.source, Some(Source::Keyring)) {
        DefaultKeyringStore.save(SERVICE, &binding.id.to_string(), &token)
            .map_err(|_| anyhow::anyhow!("OS credential store is unavailable; no plaintext fallback was written. Use an explicit credential-file or credential-env reference."))?;
    }
    install_binding(home, &binding)?;
    println!(
        "Configured workspace credential for {}. No individual user identity is asserted.",
        binding.gateway_url
    );
    Ok(())
}

pub(super) fn install_binding(home: &Path, binding: &Binding) -> anyhow::Result<()> {
    let mut config: toml::Value =
        toml::from_str(&std::fs::read_to_string(home.join("config.toml"))?)?;
    let provider = config
        .get_mut("model_providers")
        .and_then(|v| v.get_mut("airs"))
        .and_then(toml::Value::as_table_mut)
        .context("AIRS provider is missing")?;
    for field in ["env_http_headers", "env_key", "experimental_bearer_token"] {
        provider.remove(field);
    }
    let helper = serde_json::json!({
        "command": std::env::current_exe()?,
        "args": ["credential", "--home", home, "--binding", binding.id],
        "timeout_ms": 60000, "refresh_interval_ms": 1000, "cwd": home
    });
    provider.insert("auth".into(), serde_json::from_value(helper)?);
    if let Some(Source::Environment { variable }) = &binding.source {
        let policy = config
            .as_table_mut()
            .context("invalid config")?
            .entry("shell_environment_policy")
            .or_insert_with(|| toml::Value::Table(Default::default()));
        let excluded = policy
            .as_table_mut()
            .context("invalid shell environment policy")?
            .entry("exclude")
            .or_insert_with(|| toml::Value::Array(Vec::new()))
            .as_array_mut()
            .context("invalid shell exclusion list")?;
        if !excluded.iter().any(|v| v.as_str() == Some(variable)) {
            excluded.push(toml::Value::String(variable.clone()));
        }
    }
    // Binding first: interruption fails closed until the matching helper is configured.
    airs_environment::atomic_write(
        &home.join("credential-binding.json"),
        &serde_json::to_vec_pretty(&binding)?,
    )?;
    airs_environment::atomic_write(
        &home.join("config.toml"),
        toml::to_string_pretty(&config)?.as_bytes(),
    )?;
    let logged_out = home.join("logged-out");
    if logged_out.exists() {
        std::fs::remove_file(logged_out)?;
    }
    Ok(())
}

pub async fn helper(args: &HelperArgs) -> anyhow::Result<()> {
    anyhow::ensure!(
        !std::io::stdout().is_terminal(),
        "credential helper may only write to a pipe"
    );
    anyhow::ensure!(args.home.is_absolute(), "credential home must be absolute");
    let _lock = airs_environment::lock(&args.home)?;
    let binding = read_binding(&args.home)?;
    anyhow::ensure!(
        binding.id == args.binding,
        "credential binding changed; restart in the intended environment"
    );
    anyhow::ensure!(
        !args.home.join("logged-out").exists(),
        "logged out; run airs-harness login"
    );
    let token = if matches!(binding.source, Some(Source::Oidc { .. })) {
        super::airs_oidc::credential(&binding).await?
    } else {
        resolve(&binding)?
    };
    println!("{token}");
    Ok(())
}

pub async fn logout(home: &Path) -> anyhow::Result<()> {
    let _lock = airs_environment::lock(home)?;
    airs_environment::atomic_write(
        &home.join("logged-out"),
        b"Local credentials disabled. Run login to reauthenticate.\n",
    )?;
    let mcp_logout = super::airs_mcp::logout(home).await;
    if !home.join("credential-binding.json").exists() {
        println!(
            "Logged out locally. New inference and MCP credential use is disabled until login; stop running sessions to discard cached credentials."
        );
        mcp_logout?;
        return Ok(());
    }
    let mut binding = read_binding(home)?;
    if binding.source.is_none() {
        mcp_logout?;
        println!(
            "Already logged out locally. Stop running sessions to discard cached access tokens."
        );
        return Ok(());
    }
    let oidc = matches!(binding.source, Some(Source::Oidc { .. }));
    let keyring = matches!(binding.source, Some(Source::Keyring | Source::Oidc { .. }));
    let revoke = if oidc {
        super::airs_oidc::load_active(&binding).ok()
    } else {
        None
    };
    binding.source = None;
    airs_environment::atomic_write(
        &home.join("credential-binding.json"),
        &serde_json::to_vec_pretty(&binding)?,
    )?;
    if oidc {
        codex_airs_identity::CredentialStore
            .delete(SERVICE, &binding.id.to_string())
            .map_err(|_| {
                anyhow::anyhow!("local binding disabled, but OS credential-store deletion failed")
            })?;
    } else if keyring {
        DefaultKeyringStore
            .delete(SERVICE, &binding.id.to_string())
            .map_err(|_| {
                anyhow::anyhow!("local binding disabled, but OS credential-store deletion failed")
            })?;
    }
    if oidc {
        if let Some(tokens) = revoke {
            let provider =
                codex_airs_identity::Provider::discover(tokens.identity.config.clone()).await;
            match provider {
                Ok(provider) if provider.revoke(&tokens).await.is_ok() => println!(
                    "Signed out locally and revoked the issuer refresh token. Stop running sessions to discard cached access tokens."
                ),
                _ => anyhow::bail!(
                    "signed out locally; issuer revocation could not be confirmed. Existing access tokens expire normally"
                ),
            }
        } else {
            println!(
                "Signed out locally. No usable refresh token remains; existing access tokens expire normally."
            );
        }
        mcp_logout?;
        return Ok(());
    }
    mcp_logout?;
    println!(
        "Logged out locally. Referenced files/environment variables are unchanged; stop running sessions to discard their cached token. Workspace-key revocation is managed in AIRS."
    );
    Ok(())
}

pub fn status(home: &Path) -> anyhow::Result<()> {
    println!("Prisma AIRS Harness {}", super::airs_harness::version());
    println!("State: {}", home.display());
    println!("Gateway: {}", airs_environment::gateway(home)?);
    anyhow::ensure!(
        !home.join("logged-out").exists(),
        "logged out; run airs-harness login"
    );
    if home.join("credential-binding.json").exists() {
        let binding = read_binding(home)?;
        if let Some(Source::Oidc { identity }) = &binding.source {
            super::airs_oidc::load_active(&binding)?;
            println!("Authentication: OIDC; issuer {}", identity.config.issuer);
            println!("Subject: {}", identity.subject);
            println!("Audience: {}", identity.config.audience);
            println!("Credential: available in OS store (local check)");
            return Ok(());
        }
        println!(
            "Authentication: workspace credential; binding {}",
            binding.id
        );
        resolve(&binding)?;
        println!("Credential: available (local check; not an inference test)");
    } else {
        let config: toml::Value =
            toml::from_str(&std::fs::read_to_string(home.join("config.toml"))?)?;
        let variable = config
            .get("model_providers")
            .and_then(|v| v.get("airs"))
            .and_then(|v| v.get("env_http_headers"))
            .and_then(|v| v.get("x-portkey-api-key"))
            .and_then(toml::Value::as_str)
            .context("run airs-harness login to configure credentials")?;
        println!("Authentication: workspace credential from {variable}");
        let token = std::env::var(variable).context("credential is missing; use login --credential-file PATH or login --credential-env NAME")?;
        validate_token(&token)?;
        println!("Credential: available from environment (local check)");
    }
    Ok(())
}

pub(super) fn check(home: &Path) -> anyhow::Result<()> {
    identity(home).map(|_| ())
}

pub(super) fn identity(home: &Path) -> anyhow::Result<String> {
    anyhow::ensure!(
        !home.join("logged-out").exists(),
        "logged out; run airs-harness login"
    );
    if home.join("credential-binding.json").exists() {
        let binding = read_binding(home)?;
        if matches!(binding.source, Some(Source::Oidc { .. })) {
            super::airs_oidc::load_active(&binding)?;
            Ok(binding.credential_fingerprint)
        } else {
            Ok(fingerprint(&resolve(&binding)?))
        }
    } else {
        let config: toml::Value =
            toml::from_str(&std::fs::read_to_string(home.join("config.toml"))?)?;
        let variable = config
            .get("model_providers")
            .and_then(|v| v.get("airs"))
            .and_then(|v| v.get("env_http_headers"))
            .and_then(|v| v.get("x-portkey-api-key"))
            .and_then(toml::Value::as_str)
            .context("run airs-harness login to configure credentials")?;
        let token = std::env::var(variable).with_context(|| {
            format!("credential environment variable {variable} is missing; run airs-harness login")
        })?;
        Ok(fingerprint(validate_token(&token)?))
    }
}

#[cfg(test)]
#[path = "airs_credentials_tests.rs"]
mod tests;
