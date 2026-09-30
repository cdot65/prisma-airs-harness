//! Guided sign-in. Saved connection settings are public inputs, never identity claims.
use super::airs_credentials;
use super::airs_credentials::Binding;
use super::airs_credentials::LoginArgs;
use super::airs_credentials::Source;
use super::airs_environment;
use super::airs_oidc;
use anyhow::Context;
use codex_airs_identity::IdentityConfig;
use serde::Deserialize;
use serde::Serialize;
use std::io::BufRead;
use std::io::IsTerminal;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use url::Url;

const MAX_FIELD_BYTES: usize = 2_048;
const MAX_SETTINGS_BYTES: u64 = 16_384;
const SETTINGS_FILE: &str = "login-settings.json";
const SIGN_IN_MENU: &str = "Sign in to Prisma AIRS Harness\n\n  1. Company sign-in (opens your browser)\n  2. Workspace API key (hidden input)\n\nChoose 1 or 2 (Ctrl+C to cancel): ";
const DEVICE_SIGN_IN_MENU: &str = "Sign in to Prisma AIRS Harness\n\n  1. Company sign-in (device authorization)\n  2. Workspace API key (hidden input)\n\nChoose 1 or 2 (Ctrl+C to cancel): ";

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Settings {
    schema_version: u32,
    gateway_url: String,
    identity: IdentityConfig,
}

pub(super) fn public_field(value: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !value.is_empty()
            && value.len() <= MAX_FIELD_BYTES
            && value.bytes().all(|byte| byte.is_ascii_graphic()),
        "Connection settings must be nonempty text without whitespace or control characters (maximum 2048 bytes)"
    );
    Ok(())
}

pub(super) fn validate_identity(config: &IdentityConfig) -> anyhow::Result<()> {
    public_field(&config.issuer)?;
    public_field(&config.client_id)?;
    public_field(&config.audience)?;
    let issuer = Url::parse(&config.issuer).context("Invalid company issuer URL")?;
    anyhow::ensure!(
        issuer.scheme() == "https"
            && issuer.host_str().is_some()
            && issuer.username().is_empty()
            && issuer.password().is_none()
            && issuer.query().is_none()
            && issuer.fragment().is_none()
            && !config.issuer.ends_with('/'),
        "Company issuer must be an HTTPS URL without credentials, query, fragment or trailing slash"
    );
    Ok(())
}

pub(super) fn existing_binding(home: &Path) -> anyhow::Result<Option<Binding>> {
    match std::fs::symlink_metadata(home.join("credential-binding.json")) {
        Ok(_) => airs_credentials::read_binding(home).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("Cannot inspect the existing credential binding"),
    }
}

/// How an environment currently authenticates. Carries no secret values.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Method {
    WorkspaceKey { storage: String },
    Company { issuer: String, user: String },
}

impl Method {
    pub(super) fn describe(&self) -> String {
        match self {
            Self::WorkspaceKey { storage } => format!("a workspace API key ({storage})"),
            Self::Company { issuer, user } => format!("company SSO as {user} via {issuer}"),
        }
    }
}

/// Detect an established sign-in so login can offer to update it instead of failing.
/// Signed-out environments return None; their next login is an ordinary sign-in.
pub(super) fn current_method(home: &Path) -> anyhow::Result<Option<Method>> {
    let Some(binding) = existing_binding(home)? else {
        return Ok(None);
    };
    if home.join("logged-out").try_exists()? {
        return Ok(None);
    }
    Ok(binding.source.map(|source| match source {
        Source::Oidc { identity } => Method::Company {
            issuer: identity.config.issuer,
            user: identity.display_name.unwrap_or(identity.subject),
        },
        Source::Keyring | Source::KeyringV2 => Method::WorkspaceKey {
            storage: "OS credential store".into(),
        },
        Source::File { path } => Method::WorkspaceKey {
            storage: format!("file {}", path.display()),
        },
        Source::Environment { variable } => Method::WorkspaceKey {
            storage: format!("environment variable {variable}"),
        },
    }))
}

pub(super) const REPLACEMENT_NOTICE: &str = "Open AIRS sessions in this environment stop and must be restarted (airs resume). Saved conversations stay here and continue with the new credential. The previous key or sign-in is removed from this device; revoke old workspace keys in AIRS. MCP servers set up with company sign-in must be signed in again as the new user: repeat airs setup-mcp with the same options.";

/// Guided authentication change for an existing environment (airs env auth).
pub(super) async fn update(root: &Path, requested: Option<&str>) -> anyhow::Result<()> {
    anyhow::ensure!(
        std::io::stdin().is_terminal() && std::io::stderr().is_terminal(),
        "Guided authentication changes require an interactive terminal. Automation: airs --environment NAME login --replace with --with-api-key, --credential-file, --credential-env or --issuer-url; see airs env auth --help"
    );
    if codex_tui::AirsOnboarding::supported() {
        super::airs_welcome::run(root, requested, super::airs_welcome::Entry::UpdateAuth, &[])
            .await?;
        return Ok(());
    }
    airs_environment::select(root, requested)?;
    let home = codex_core::config::find_codex_home()?;
    let environment = airs_environment::name_for_home(root, home.as_path())?;
    interactive(
        home.as_path(),
        airs_oidc::LoginFlow::Browser,
        environment.as_deref(),
    )
    .await
}

/// Automation performs the same connectivity test as guided replacement.
pub(super) async fn verify_replacement(
    home: &Path,
    environment: Option<&str>,
) -> anyhow::Result<()> {
    eprintln!("{}", super::airs_access::DISCLOSURE);
    let access = super::airs_access::verify(home).await;
    let report = access.after_login(environment);
    anyhow::ensure!(access.outcome.is_ok(), "{report}");
    eprintln!("{report}");
    Ok(())
}

/// Classify missing sign-in without probing, replacing or suppressing store errors.
pub(super) fn needs_login(
    home: &Path,
    environment_available: impl Fn(&str) -> bool,
) -> anyhow::Result<bool> {
    // Parse an existing binding before the logout marker: damaged identity state
    // must never be mistaken for a fresh account that can overwrite it.
    let binding = existing_binding(home)?;
    let generation = codex_utils_home_dir::airs_session::read_auth_generation(home)?;
    if home.join("logged-out").try_exists()?
        || generation.is_some_and(|generation| {
            generation.state == codex_utils_home_dir::airs_session::AuthGenerationState::Revoked
        })
    {
        return Ok(true);
    }
    if let Some(binding) = binding {
        return Ok(binding.source.is_none());
    }
    let config: toml::Value = toml::from_str(&std::fs::read_to_string(home.join("config.toml"))?)?;
    let variable = config
        .get("model_providers")
        .and_then(|value| value.get("airs"))
        .and_then(|value| value.get("env_http_headers"))
        .and_then(|value| value.get("x-portkey-api-key"))
        .and_then(toml::Value::as_str)
        .context(
            "Credential configuration is invalid; inspect this environment with airs doctor",
        )?;
    Ok(!environment_available(variable))
}

pub(super) fn read_settings(home: &Path) -> anyhow::Result<Option<IdentityConfig>> {
    let path = home.join(SETTINGS_FILE);
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = match options.open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => anyhow::bail!("Cannot read saved company sign-in settings"),
    };
    anyhow::ensure!(
        std::fs::symlink_metadata(&path)?.is_file()
            && file.metadata()?.is_file()
            && file.metadata()?.len() <= MAX_SETTINGS_BYTES,
        "Saved company sign-in settings must be a regular file of at most 16 KiB"
    );
    let mut bytes = Vec::new();
    file.take(MAX_SETTINGS_BYTES + 1).read_to_end(&mut bytes)?;
    anyhow::ensure!(
        bytes.len() <= MAX_SETTINGS_BYTES as usize,
        "Saved company sign-in settings exceed 16 KiB"
    );
    let settings: Settings = serde_json::from_slice(&bytes)
        .map_err(|_| anyhow::anyhow!("Invalid saved company sign-in settings"))?;
    anyhow::ensure!(
        settings.schema_version == 1 && settings.gateway_url == airs_environment::gateway(home)?,
        "Saved company sign-in settings do not match this environment"
    );
    validate_identity(&settings.identity)?;
    Ok(Some(settings.identity))
}

/// Persist only public connection inputs before authentication so retries need no flags.
pub(super) fn remember_settings(home: &Path, args: &LoginArgs) -> anyhow::Result<()> {
    let identity = IdentityConfig {
        issuer: args
            .issuer_url
            .clone()
            .context("Company issuer is required")?,
        client_id: args
            .oidc_client_id
            .clone()
            .context("Company client ID is required")?,
        audience: args
            .audience
            .clone()
            .context("Company audience is required")?,
    };
    validate_identity(&identity)?;
    let _lock = airs_environment::lock(home)?;
    let settings = Settings {
        schema_version: 1,
        gateway_url: airs_environment::gateway(home)?,
        identity,
    };
    airs_environment::atomic_write(
        &home.join(SETTINGS_FILE),
        &serde_json::to_vec_pretty(&settings)?,
    )
    .context("Cannot save company sign-in settings")
}

pub(super) fn prompt(
    input: &mut impl BufRead,
    output: &mut impl Write,
    label: &str,
) -> anyhow::Result<String> {
    write!(output, "{label}")?;
    output.flush()?;
    let mut bytes = Vec::new();
    input
        .take((MAX_FIELD_BYTES + 3) as u64)
        .read_until(b'\n', &mut bytes)?;
    anyhow::ensure!(
        bytes.last() == Some(&b'\n'),
        "Sign-in cancelled: input ended or exceeded the input limit"
    );
    bytes.pop();
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    anyhow::ensure!(
        bytes.len() <= MAX_FIELD_BYTES && bytes.iter().all(u8::is_ascii_graphic),
        "Enter text without whitespace or control characters (maximum 2048 bytes)"
    );
    String::from_utf8(bytes).context("Invalid sign-in input")
}

fn company_settings(
    home: &Path,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> anyhow::Result<IdentityConfig> {
    // An established binding supplies the defaults; changing them is an explicit
    // replacement. Only damaged public settings trigger this recovery prompt.
    let bound_config = existing_binding(home)?.and_then(|binding| match binding.source {
        Some(Source::Oidc { identity }) => Some(identity.config),
        _ => None,
    });
    let config = if let Some(config) = bound_config {
        validate_identity(&config)?;
        Some(config)
    } else {
        match read_settings(home) {
            Ok(config) => config,
            Err(_) => {
                let choice = prompt(
                    input,
                    output,
                    "Saved company sign-in settings are invalid or unavailable.\nType 1 to enter replacement settings, or press Enter to cancel: ",
                )?;
                anyhow::ensure!(
                    choice == "1",
                    "Sign-in cancelled; saved settings were not changed"
                );
                None
            }
        }
    };
    if let Some(config) = config {
        writeln!(
            output,
            "\nCompany issuer: {}\nClient ID: {}\nAudience: {}",
            config.issuer, config.client_id, config.audience
        )?;
        match prompt(
            input,
            output,
            "Press Enter to continue with these settings, or type 2 to change them: ",
        )?
        .as_str()
        {
            "" => return Ok(config),
            "2" => {}
            _ => anyhow::bail!("Sign-in cancelled; choose Enter or 2 next time"),
        }
    }
    writeln!(
        output,
        "\nUse the public sign-in settings supplied by your organization.\nYour company password is entered only in your browser."
    )?;
    let config = IdentityConfig {
        issuer: prompt(input, output, "Company issuer URL (HTTPS): ")?,
        client_id: prompt(input, output, "Public client ID: ")?,
        audience: prompt(input, output, "Gateway audience: ")?,
    };
    validate_identity(&config)?;
    Ok(config)
}

pub(super) async fn interactive(
    home: &Path,
    preferred_flow: airs_oidc::LoginFlow,
    environment: Option<&str>,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        std::io::stdin().is_terminal() && std::io::stderr().is_terminal(),
        "Guided login requires an interactive terminal. Automation must select an explicit login method; see airs login --help"
    );
    let command = airs_environment::command(environment);
    // Check configuration before collecting input, without probing secret storage.
    airs_environment::gateway(home)?;
    let current = current_method(home)?;
    let choice = {
        let mut input = std::io::stdin().lock();
        let mut output = std::io::stderr().lock();
        if let Some(current) = &current {
            writeln!(
                output,
                "This environment signs in with {}.\nChoosing a different key or identity replaces it. {REPLACEMENT_NOTICE}\n",
                current.describe()
            )?;
        }
        let menu = match preferred_flow {
            airs_oidc::LoginFlow::Browser | airs_oidc::LoginFlow::BrowserManual => SIGN_IN_MENU,
            airs_oidc::LoginFlow::Device => DEVICE_SIGN_IN_MENU,
        };
        prompt(&mut input, &mut output, menu)
            .with_context(|| format!("Resume sign-in with: {command} login"))?
    };
    // Choosing a method while one is established is the explicit replacement consent.
    let replace = current.is_some();
    let result = match choice.as_str() {
        "1" => {
            let config = company_settings(
                home,
                &mut std::io::stdin().lock(),
                &mut std::io::stderr().lock(),
            )
            .with_context(|| format!("Resume sign-in with: {command} login"))?;
            let args = LoginArgs {
                issuer_url: Some(config.issuer),
                oidc_client_id: Some(config.client_id),
                audience: Some(config.audience),
                replace,
                ..Default::default()
            };
            remember_settings(home, &args)?;
            airs_oidc::login(home, &args, preferred_flow).await
        }
        "2" => {
            airs_credentials::login(
                home,
                &LoginArgs {
                    replace,
                    ..Default::default()
                },
                /*stdin_key*/ true,
            )
            .await
        }
        _ => Err(anyhow::anyhow!(
            "Sign-in cancelled; choose 1 or 2 next time"
        )),
    };
    result.with_context(|| format!("Resume sign-in with: {command} login"))?;
    eprintln!("{}", super::airs_access::DISCLOSURE);
    let access = super::airs_access::verify(home).await;
    eprintln!("{}", access.after_login(environment));
    Ok(())
}

#[cfg(test)]
#[path = "airs_login_tests.rs"]
mod tests;
