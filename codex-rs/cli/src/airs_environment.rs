//! Named AIRS environments. Each UUID owns its complete upstream runtime home.
use anyhow::Context;
use clap::Subcommand;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create an environment and sign in; supply --gateway-url for automation.
    Create {
        /// Environment name. Guided setup prompts when omitted.
        name: Option<String>,
        #[command(flatten)]
        args: super::airs_harness::SetupArgs,
    },
    /// List environments; an asterisk marks the default for new processes.
    List,
    /// Select the default environment for new processes.
    Use { name: String },
    /// Inspect a named or selected environment without showing credentials.
    Show { name: Option<String> },
    /// Show the gateway and local credential availability for an environment.
    Status { name: Option<String> },
    /// Rename an environment while preserving its credentials and history.
    Rename { name: String, new_name: String },
    /// Unregister an environment, preserving its local history on disk.
    Remove { name: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    schema_version: u32,
    active: Option<String>,
    environments: BTreeMap<String, Environment>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            schema_version: 1,
            active: None,
            environments: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Environment {
    id: Uuid,
    gateway_url: String,
}

pub fn private_directory(path: &Path) -> anyhow::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)?;
    anyhow::ensure!(
        std::fs::symlink_metadata(path)?.is_dir(),
        "state path must be a real directory"
    );
    Ok(())
}

pub fn atomic_write(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    let parent = path.parent().context("file has no parent directory")?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(contents)?;
    file.as_file().sync_all()?;
    file.persist(path)?;
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
}

pub fn lock(home: &Path) -> anyhow::Result<File> {
    let file = open_lock(home)?;
    file.lock()?;
    Ok(file)
}

/// Renewal waits briefly for an existing configuration owner without blocking the runtime.
pub(super) async fn credential_lock(home: &Path) -> anyhow::Result<File> {
    let file = open_lock(home)?;
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(std::fs::TryLockError::WouldBlock) => {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await
                }
                Err(std::fs::TryLockError::Error(error)) => return Err(anyhow::Error::from(error)),
            }
        }
    })
    .await
    .context("Authentication is busy in another process; retry when it finishes")?
}

fn open_lock(home: &Path) -> anyhow::Result<File> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(home.join(".configuration.lock"))?;
    anyhow::ensure!(file.metadata()?.is_file(), "invalid configuration lock");
    Ok(file)
}

fn read(root: &Path) -> anyhow::Result<Registry> {
    let path = root.join("environments.json");
    if !path.exists() {
        return Ok(Registry::default());
    }
    let registry: Registry =
        serde_json::from_slice(&std::fs::read(path)?).context("invalid environment registry")?;
    anyhow::ensure!(
        registry.schema_version == 1,
        "unsupported environment schema version"
    );
    if let Some(name) = &registry.active {
        anyhow::ensure!(
            registry.environments.contains_key(name),
            "active environment is missing"
        );
    }
    Ok(registry)
}

fn write(root: &Path, registry: &Registry) -> anyhow::Result<()> {
    atomic_write(
        &root.join("environments.json"),
        &serde_json::to_vec_pretty(registry)?,
    )
}

fn environment_home(root: &Path, environment: &Environment) -> PathBuf {
    root.join("environments").join(environment.id.to_string())
}

pub fn gateway(home: &Path) -> anyhow::Result<String> {
    let config: toml::Value = toml::from_str(&std::fs::read_to_string(home.join("config.toml"))?)?;
    config
        .get("model_providers")
        .and_then(|v| v.get("airs"))
        .and_then(|v| v.get("base_url"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .context("run airs env create to configure the gateway")
}

pub fn select(root: &Path, requested: Option<&str>) -> anyhow::Result<()> {
    let registry = read(root)?;
    let name = requested.or(registry.active.as_deref());
    let Some(name) = name else {
        anyhow::ensure!(
            !root.join("environments.json").exists(),
            "select an environment with --environment NAME or env use NAME"
        );
        return Ok(());
    };
    let environment = registry
        .environments
        .get(name)
        .context("unknown environment; use env list")?;
    let home = environment_home(root, environment);
    anyhow::ensure!(
        gateway(&home)? == environment.gateway_url,
        "gateway binding changed; create a new environment and authenticate explicitly"
    );
    codex_utils_home_dir::select_airs_environment_home(home)?;
    Ok(())
}

pub(super) fn validate_name(name: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !name.is_empty()
            && name.len() <= 64
            && name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.')),
        "environment name must contain 1–64 letters, digits, dots, underscores or hyphens"
    );
    Ok(())
}

pub fn setup(root: &Path, name: &str, args: &super::airs_harness::SetupArgs) -> anyhow::Result<()> {
    let home = create(root, name, args)?;
    println!("Configured Prisma AIRS Harness in {}", home.display());
    println!("Run airs login to sign in with your company account or workspace API key.");
    println!("Selected environment {name}. Its sessions and credentials are independent.");
    Ok(())
}

/// Resolve public context before committing this process's immutable selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Selection {
    pub name: Option<String>,
    pub home: PathBuf,
    pub gateway: String,
}

pub(super) fn choices(root: &Path) -> anyhow::Result<Vec<(String, String)>> {
    Ok(read(root)?
        .environments
        .into_iter()
        .map(|(name, env)| (name, env.gateway_url))
        .collect())
}

pub(super) fn resolve(root: &Path, requested: Option<&str>) -> anyhow::Result<Option<Selection>> {
    let registry = read(root)?;
    let name = requested.or(registry.active.as_deref());
    let Some(name) = name else {
        return if root.join("config.toml").exists() && !root.join("environments.json").exists() {
            Ok(Some(Selection {
                name: None,
                home: root.to_owned(),
                gateway: gateway(root)?,
            }))
        } else {
            Ok(None)
        };
    };
    let environment = registry
        .environments
        .get(name)
        .context("unknown environment; use env list")?;
    let home = environment_home(root, environment);
    anyhow::ensure!(
        gateway(&home)? == environment.gateway_url,
        "gateway binding changed; create a new environment and authenticate explicitly"
    );
    Ok(Some(Selection {
        name: Some(name.to_owned()),
        home,
        gateway: environment.gateway_url.clone(),
    }))
}

pub(super) fn create(
    root: &Path,
    name: &str,
    args: &super::airs_harness::SetupArgs,
) -> anyhow::Result<PathBuf> {
    validate_name(name)?;
    let _lock = lock(root)?;
    let mut registry = read(root)?;
    anyhow::ensure!(
        !registry.environments.contains_key(name),
        "environment already exists; setup never overwrites it"
    );
    let mut environment = Environment {
        id: Uuid::new_v4(),
        gateway_url: String::new(),
    };
    let home = environment_home(root, &environment);
    private_directory(&home)?;
    super::airs_harness::setup_in(args, &home)?;
    environment.gateway_url = gateway(&home)?;
    registry.environments.insert(name.to_owned(), environment);
    registry.active = Some(name.to_owned());
    write(root, &registry)?;
    Ok(home)
}

pub async fn run(root: &Path, command: &Command, requested: Option<&str>) -> anyhow::Result<()> {
    if let Command::Status { name } = command {
        select(root, name.as_deref().or(requested))?;
        let home = codex_core::config::find_codex_home()?;
        return super::airs_credentials::status(home.as_path());
    }
    // The wizard owns its registry lock and releases it before browser sign-in.
    if let Command::Create { name, args } = command {
        anyhow::ensure!(
            name.as_deref()
                .zip(requested)
                .is_none_or(|(name, requested)| name == requested),
            "environment name conflicts with --environment"
        );
        let name = name.as_deref().or(requested);
        if args.gateway_url.is_empty() {
            return super::airs_setup::interactive(root, name, args).await;
        }
        let name = name.context("supply a name: airs env create NAME --gateway-url URL")?;
        setup(root, name, args)?;
        println!("Next: airs --environment {name} login");
        println!("Then: airs --environment {name} doctor --verify-access");
        return Ok(());
    }
    let _lock = lock(root)?;
    let mut registry = read(root)?;
    match command {
        Command::Create { .. } | Command::Status { .. } => {
            unreachable!("creation and status are handled before locking the registry")
        }
        Command::List => {
            for (name, environment) in &registry.environments {
                let marker = if registry.active.as_ref() == Some(name) {
                    "*"
                } else {
                    " "
                };
                println!("{marker} {name}\t{}", environment.gateway_url);
            }
            if registry.environments.is_empty() {
                println!("No named environments. Run airs env create to get started.");
            }
        }
        Command::Show { name } => {
            let name = name
                .as_deref()
                .or(requested)
                .or(registry.active.as_deref())
                .context("no environment selected; use env list and env use NAME")?;
            let environment = registry
                .environments
                .get(name)
                .context("unknown environment")?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "name": name, "id": environment.id, "gateway_url": environment.gateway_url,
                    "state_directory": environment_home(root, environment)
                }))?
            );
        }
        Command::Use { name } => {
            anyhow::ensure!(
                registry.environments.contains_key(name),
                "unknown environment"
            );
            registry.active = Some(name.clone());
            write(root, &registry)?;
            println!("Selected {name} for new processes. Running sessions keep their environment.");
        }
        Command::Rename { name, new_name } => {
            validate_name(new_name)?;
            anyhow::ensure!(
                !registry.environments.contains_key(new_name),
                "environment already exists; choose another name"
            );
            let environment = registry
                .environments
                .remove(name)
                .context("unknown environment")?;
            registry.environments.insert(new_name.clone(), environment);
            if registry.active.as_ref() == Some(name) {
                registry.active = Some(new_name.clone());
            }
            write(root, &registry)?;
            println!("Renamed {name} to {new_name}. Credentials and history are unchanged.");
        }
        Command::Remove { name } => {
            let environment = registry
                .environments
                .remove(name)
                .context("unknown environment")?;
            if registry.active.as_ref() == Some(name) {
                registry.active = None;
            }
            write(root, &registry)?;
            println!(
                "Unregistered {name}. History remains at {}.",
                environment_home(root, &environment).display()
            );
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "airs_environment_tests.rs"]
mod tests;
