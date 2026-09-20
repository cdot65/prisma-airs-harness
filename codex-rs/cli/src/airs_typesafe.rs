//! Optional TypeSafe judge credential bound to one environment.
//!
//! The API key lives in the OS credential store under its own namespace; the
//! environment home keeps only public settings and a fingerprint. At session
//! invocation the harness passes the key only to an explicitly requested child
//! command through `airs env typesafe exec`. The managed CLI uses tenant settings. A
//! variable already present in the process always wins over the saved key.
use anyhow::Context;
use clap::Subcommand;
use codex_airs_identity::WorkspaceCredentialFormat;
use codex_airs_identity::WorkspaceCredentialStore;
use serde::Deserialize;
use serde::Serialize;
use std::io::IsTerminal;
use std::io::Read;
use std::path::Path;
use std::time::Duration;
use uuid::Uuid;

#[path = "airs_typesafe_storage.rs"]
mod storage;

/// Distinct OS-store namespace so judge keys never alias gateway credentials.
pub(super) const SERVICE: &str = "io.cdot.airs-terminal.typesafe";
const SETTINGS_FILE: &str = "typesafe.json";
const MAX_SETTINGS_BYTES: u64 = 16 * 1024;
pub(super) const KEY_VARIABLE: &str = "TYPESAFE_API_KEY";
pub(super) const BASE_URL_VARIABLE: &str = "TYPESAFE_BASE_URL";
pub(super) const MODEL_VARIABLE: &str = "TYPESAFE_DEFAULT_MODEL";
pub(super) const DEFAULT_BASE_URL: &str = "https://api.typesafe.ai";
pub(super) const DEFAULT_MODEL: &str = "jev-latest";

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Save a TypeSafe API key for this environment (hidden input, or --stdin).
    Set {
        /// Read the key from standard input instead of the hidden prompt.
        #[arg(long)]
        stdin: bool,
        /// Model alias exported as TYPESAFE_DEFAULT_MODEL (default jev-latest).
        #[arg(long)]
        model: Option<String>,
        /// HTTPS base URL exported as TYPESAFE_BASE_URL (default https://api.typesafe.ai).
        #[arg(long)]
        base_url: Option<String>,
    },
    /// Show whether a TypeSafe key is configured, without revealing it.
    Status,
    /// Run the bundled judge with TypeSafe variables scoped to its child process.
    Exec {
        /// Bind an installed skill to its owning environment, ignoring the default.
        #[arg(long, hide = true)]
        skill_home: Option<std::path::PathBuf>,
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<std::ffi::OsString>,
    },
    /// Remove the saved TypeSafe key and settings from this environment.
    Clear,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Settings {
    pub(super) schema_version: u32,
    pub(super) id: Uuid,
    pub(super) model: Option<String>,
    pub(super) base_url: Option<String>,
    pub(super) key_fingerprint: String,
}

/// Native store operations, mockable in tests.
pub(super) trait Store {
    fn save(&self, account: Uuid, token: &str) -> anyhow::Result<()>;
    fn load(&self, account: Uuid) -> anyhow::Result<Option<String>>;
    fn delete(&self, account: Uuid) -> anyhow::Result<()>;
}

pub(super) struct NativeStore;

impl Store for NativeStore {
    fn save(&self, account: Uuid, token: &str) -> anyhow::Result<()> {
        WorkspaceCredentialStore
            .save_new(SERVICE, account, token)
            .map_err(|error| super::airs_storage_error::report(error, "save"))
    }
    fn load(&self, account: Uuid) -> anyhow::Result<Option<String>> {
        WorkspaceCredentialStore
            .load(SERVICE, WorkspaceCredentialFormat::ChunkedV2, account)
            .map_err(|error| super::airs_storage_error::report(error, "read"))
    }
    fn delete(&self, account: Uuid) -> anyhow::Result<()> {
        WorkspaceCredentialStore
            .delete(SERVICE, WorkspaceCredentialFormat::ChunkedV2, account)
            .map(|_| ())
            .map_err(|error| super::airs_storage_error::report(error, "delete"))
    }
}

pub(super) fn validate_token(token: &str) -> anyhow::Result<&str> {
    anyhow::ensure!(
        !token.is_empty() && token.len() <= 16_384 && token.bytes().all(|b| b.is_ascii_graphic()),
        "A TypeSafe API key must be a non-empty ASCII token of at most 16 KiB without whitespace"
    );
    Ok(token)
}

pub(super) fn validate_base_url(value: &str) -> anyhow::Result<String> {
    let parsed =
        url::Url::parse(value).map_err(|_| anyhow::anyhow!("Invalid TypeSafe base URL"))?;
    let loopback = match parsed.host() {
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        Some(url::Host::Domain("localhost")) => true,
        _ => false,
    };
    anyhow::ensure!(
        value.len() <= 2048
            && value.bytes().all(|byte| byte.is_ascii_graphic())
            && (parsed.scheme() == "https" || parsed.scheme() == "http" && loopback)
            && parsed.host_str().is_some()
            && parsed.username().is_empty()
            && parsed.password().is_none()
            && parsed.query().is_none()
            && parsed.fragment().is_none(),
        "TypeSafe base URL must be HTTPS (HTTP only for loopback) without credentials, query or fragment"
    );
    Ok(value.trim_end_matches('/').to_owned())
}

fn validate_model(value: &str) -> anyhow::Result<String> {
    anyhow::ensure!(
        !value.is_empty()
            && value.len() <= 128
            && value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b':')),
        "Model alias must be 1 to 128 characters of letters, digits, '-', '.', '_' or ':'"
    );
    Ok(value.to_owned())
}

pub(super) fn read(home: &Path) -> anyhow::Result<Option<Settings>> {
    let path = home.join(SETTINGS_FILE);
    let file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("Unable to read TypeSafe settings"),
    };
    let metadata = file.metadata()?;
    anyhow::ensure!(
        metadata.is_file() && metadata.len() <= MAX_SETTINGS_BYTES,
        "TypeSafe settings must be a regular file of at most 16 KiB"
    );
    let mut bytes = Vec::new();
    file.take(MAX_SETTINGS_BYTES).read_to_end(&mut bytes)?;
    let settings: Settings = serde_json::from_slice(&bytes).map_err(|_| {
        anyhow::anyhow!("Invalid saved TypeSafe settings; run airs env typesafe clear")
    })?;
    anyhow::ensure!(
        settings.schema_version == 1,
        "Unsupported TypeSafe settings version"
    );
    if let Some(base_url) = &settings.base_url {
        validate_base_url(base_url)?;
    }
    if let Some(model) = &settings.model {
        validate_model(model)?;
    }
    Ok(Some(settings))
}

/// Save a new key, replacing any previous entry only after the new one is verified.
pub(super) fn save(
    home: &Path,
    store: &impl Store,
    token: &str,
    model: Option<&str>,
    base_url: Option<&str>,
) -> anyhow::Result<Settings> {
    validate_token(token)?;
    let settings = Settings {
        schema_version: 1,
        id: Uuid::new_v4(),
        model: model.map(validate_model).transpose()?,
        base_url: base_url.map(validate_base_url).transpose()?,
        key_fingerprint: super::airs_credentials::fingerprint(token),
    };
    let _lock = storage::lock(home)?;
    storage::recover(home, store)?;
    let previous = read(home)?;
    let mut pending = vec![settings.id];
    if let Some(previous) = previous {
        pending.push(previous.id);
    }
    storage::pending(home, &pending)?;
    let result = (|| -> anyhow::Result<()> {
        store.save(settings.id, token)?;
        let readback = store.load(settings.id)?;
        anyhow::ensure!(
            readback.as_deref() == Some(token),
            "Saved TypeSafe key could not be read back; the previous binding is preserved"
        );
        super::airs_environment::atomic_write(
            &home.join(SETTINGS_FILE),
            &serde_json::to_vec_pretty(&settings)?,
        )?;
        Ok(())
    })();
    let cleanup = storage::recover(home, store);
    cleanup?;
    result?;
    Ok(settings)
}

pub(super) fn clear(home: &Path, store: &impl Store) -> anyhow::Result<bool> {
    let _lock = storage::lock(home)?;
    storage::recover(home, store)?;
    let Some(settings) = read(home)? else {
        return Ok(false);
    };
    store.delete(settings.id)?;
    std::fs::remove_file(home.join(SETTINGS_FILE))?;
    Ok(true)
}

fn load_key(store: &impl Store, settings: &Settings) -> anyhow::Result<String> {
    let access_context = "This process could not read the saved TypeSafe credential. \
        A shell sandbox or locked credential store can prevent access even after successful setup. \
        Inside AIRS, retry the judge using per-command approval for credential-store and network access. \
        If the approved command still fails, check /typesafe and unlock the native credential store before replacing the key.";
    let token = store
        .load(settings.id)
        .context(access_context)?
        .context(access_context)?;
    anyhow::ensure!(
        super::airs_credentials::fingerprint(&token) == settings.key_fingerprint,
        "Saved TypeSafe key does not match its recorded fingerprint; run airs env typesafe set"
    );
    Ok(token)
}

/// Where the exported key came from, for diagnostics that never show the key.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Origin {
    NotConfigured,
    ProcessVariable,
    Environment,
}

/// Variables the session should export. Process variables always take precedence.
pub(super) fn exports(
    home: &Path,
    store: &impl Store,
    present: impl Fn(&str) -> bool,
) -> anyhow::Result<(Origin, Vec<(&'static str, String)>)> {
    if present(KEY_VARIABLE) {
        return Ok((Origin::ProcessVariable, Vec::new()));
    }
    let Some(settings) = read(home)? else {
        return Ok((Origin::NotConfigured, Vec::new()));
    };
    let mut variables = vec![(KEY_VARIABLE, load_key(store, &settings)?)];
    if let Some(base_url) = settings.base_url
        && !present(BASE_URL_VARIABLE)
    {
        variables.push((BASE_URL_VARIABLE, base_url));
    }
    if let Some(model) = settings.model
        && !present(MODEL_VARIABLE)
    {
        variables.push((MODEL_VARIABLE, model));
    }
    Ok((Origin::Environment, variables))
}

/// Build a child command without mutating the multithreaded harness environment.
pub(super) fn child_command(
    home: &Path,
    store: &impl Store,
    arguments: &[std::ffi::OsString],
) -> anyhow::Result<std::process::Command> {
    let (program, args) = arguments.split_first().context("Supply a judge command")?;
    let (_, variables) = exports(home, store, |name| std::env::var_os(name).is_some())?;
    let mut command = std::process::Command::new(program);
    command.args(args).envs(variables);
    Ok(command)
}

/// Public status line; never includes the key.
pub(super) fn status_detail(home: &Path, present: impl Fn(&str) -> bool) -> String {
    if present(KEY_VARIABLE) {
        return format!(
            "Configured from process variable {KEY_VARIABLE}; the environment setting is not used"
        );
    }
    match read(home) {
        Ok(None) => "Not configured (optional). Set with airs env typesafe set".into(),
        Ok(Some(settings)) => format!(
            "Saved OS-store binding {}; model {}; base URL {}",
            settings.id,
            settings.model.as_deref().unwrap_or(DEFAULT_MODEL),
            settings.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL)
        ),
        Err(error) => error.to_string(),
    }
}

/// Authenticated model listing, the same endpoint the official SDK uses. No state
/// or question is sent, so the probe consumes no judgment tokens.
pub(super) async fn probe(home: &Path) -> anyhow::Result<String> {
    let (key, base_url) = match std::env::var(KEY_VARIABLE) {
        Ok(key) => (
            key,
            std::env::var(BASE_URL_VARIABLE).unwrap_or_else(|_| DEFAULT_BASE_URL.into()),
        ),
        Err(_) => {
            let settings = read(home)?.context("Not configured")?;
            (
                load_key(&NativeStore, &settings)?,
                std::env::var(BASE_URL_VARIABLE)
                    .ok()
                    .or(settings.base_url)
                    .unwrap_or_else(|| DEFAULT_BASE_URL.into()),
            )
        }
    };
    probe_key(&key, &base_url).await
}

async fn probe_key(key: &str, base_url: &str) -> anyhow::Result<String> {
    validate_token(key)?;
    let endpoint = url::Url::parse(&format!("{}/v1/models", validate_base_url(base_url)?))?;
    let mut response = codex_http_client::HttpClientBuilder::new()
        .without_request_logging()
        .without_redirects()
        .build_respecting_outbound_proxy_policy(
            &codex_http_client::HttpClientFactory::new(
                codex_http_client::OutboundProxyPolicy::ReqwestDefault,
            ),
            endpoint.as_str(),
            codex_http_client::ClientRouteClass::Other,
        )?
        .get(endpoint.as_str())
        .bearer_auth(key)
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .map_err(|_| {
            anyhow::anyhow!("TypeSafe API unreachable; check network access to {base_url}")
        })?;
    match response.status().as_u16() {
        200..=299 => {
            let mut bytes = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .context("Cannot read TypeSafe models response")?
            {
                anyhow::ensure!(
                    bytes.len() + chunk.len() <= 65_536,
                    "TypeSafe models response exceeds size limit"
                );
                bytes.extend_from_slice(&chunk);
            }
            let payload: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|_| anyhow::anyhow!("Invalid TypeSafe models response"))?;
            anyhow::ensure!(
                payload
                    .get("models")
                    .is_some_and(serde_json::Value::is_array),
                "TypeSafe models response has no models array"
            );
            Ok("Key accepted by the TypeSafe models endpoint; no judgment request sent".into())
        }
        401 | 403 => anyhow::bail!(
            "TypeSafe rejected the key (HTTP {}); run airs env typesafe set",
            response.status().as_u16()
        ),
        status => anyhow::bail!("TypeSafe models endpoint returned HTTP {status}"),
    }
}

fn read_stdin_key() -> anyhow::Result<String> {
    anyhow::ensure!(
        !std::io::stdin().is_terminal(),
        "--stdin expects a piped key; omit it for the hidden prompt"
    );
    let mut buffer = String::new();
    std::io::stdin()
        .lock()
        .take(16_384 + 2)
        .read_to_string(&mut buffer)?;
    Ok(buffer.trim_end_matches(['\r', '\n']).to_owned())
}

/// Hidden-prompt entry used by setup and by `env typesafe set`.
pub(super) fn set_interactive(
    home: &Path,
    model: Option<&str>,
    base_url: Option<&str>,
) -> anyhow::Result<Settings> {
    let token = super::airs_secret_prompt::typesafe_key()?;
    save(home, &NativeStore, &token, model, base_url)
}

pub(super) async fn run(
    root: &Path,
    command: &Command,
    requested: Option<&str>,
) -> anyhow::Result<()> {
    let bound_name = match command {
        Command::Exec {
            skill_home: Some(home),
            ..
        } => skill_environment(root, home, requested)?,
        _ => requested.map(str::to_owned),
    };
    super::airs_environment::select(root, bound_name.as_deref())?;
    let home = codex_core::config::find_codex_home()?;
    let home = home.as_path();
    let label = super::airs_environment::command(requested);
    match command {
        Command::Set {
            stdin,
            model,
            base_url,
        } => {
            let settings = if *stdin {
                let token = read_stdin_key()?;
                save(
                    home,
                    &NativeStore,
                    &token,
                    model.as_deref(),
                    base_url.as_deref(),
                )?
            } else {
                set_interactive(home, model.as_deref(), base_url.as_deref())?
            };
            println!(
                "Saved TypeSafe key for this environment (binding {}). `airs env typesafe exec -- COMMAND` passes {KEY_VARIABLE} only to that child; shell variables take precedence.",
                settings.id
            );
            println!("Check it: {label} doctor --verify-access");
        }
        Command::Status => println!(
            "TypeSafe judge: {}",
            status_detail(home, |name| std::env::var_os(name).is_some())
        ),
        Command::Exec { command, .. } => {
            let status = child_command(home, &NativeStore, command)?.status()?;
            std::process::exit(status.code().unwrap_or(1));
        }
        Command::Clear => {
            if clear(home, &NativeStore)? {
                println!("Removed the saved TypeSafe key and settings from this environment.");
            } else {
                println!("No TypeSafe key is saved for this environment.");
            }
        }
    }
    Ok(())
}

/// Bind credential access to the installed skill, including after default changes.
fn skill_environment(
    root: &Path,
    home: &Path,
    requested: Option<&str>,
) -> anyhow::Result<Option<String>> {
    let root = root.canonicalize()?;
    let home = home
        .canonicalize()
        .context("The skill's environment home is unavailable")?;
    let name = super::airs_environment::name_for_home(&root, &home)?;
    anyhow::ensure!(
        name.is_some() || (home == root && !root.join("environments.json").exists()),
        "The skill's environment is no longer registered; start airs in a registered environment"
    );
    anyhow::ensure!(
        requested.is_none() || requested == name.as_deref(),
        "The requested environment does not own this installed skill"
    );
    Ok(name)
}

#[cfg(test)]
#[path = "airs_typesafe_tests.rs"]
mod tests;
