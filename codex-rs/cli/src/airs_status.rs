//! Local configuration inspection. Native credentials are never opened here.
use super::airs_credentials;
use super::airs_credentials::Source;
use anyhow::Context;
use codex_utils_home_dir::airs_session::AirsSessionGuard;
use std::io::Read;
use std::path::Path;

const MAX_PUBLIC_BYTES: u64 = 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Inspection {
    pub(super) gateway: String,
    pub(super) authentication: String,
    pub(super) detail: &'static str,
}

fn public_file(path: &Path) -> anyhow::Result<String> {
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
    let file = options
        .open(path)
        .context("Cannot read saved authentication configuration")?;
    let metadata = file.metadata()?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        anyhow::ensure!(
            metadata.file_attributes() & 0x0400 == 0,
            "Invalid authentication configuration file"
        );
    }
    anyhow::ensure!(
        metadata.is_file() && metadata.len() <= MAX_PUBLIC_BYTES,
        "Authentication configuration must be a regular file of at most 1 MiB"
    );
    let mut value = String::new();
    file.take(MAX_PUBLIC_BYTES + 1)
        .read_to_string(&mut value)
        .context("Invalid authentication configuration text")?;
    anyhow::ensure!(
        value.len() <= MAX_PUBLIC_BYTES as usize,
        "Authentication configuration is too large"
    );
    Ok(value)
}

fn display_field(value: &str) -> anyhow::Result<&str> {
    // Reject controls, including bidi/format characters, instead of echoing saved
    // data into the terminal. JSON quoting also makes whitespace unambiguous.
    anyhow::ensure!(
        !value.is_empty() && value.len() <= 2048
            && value.chars().all(|character| !character.is_control()
                && !matches!(character, '\u{061c}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}')),
        "Invalid saved authentication metadata"
    );
    Ok(value)
}

pub(super) fn inspect(home: &Path) -> anyhow::Result<Inspection> {
    anyhow::ensure!(
        !home.join("logged-out").exists(),
        "logged out; run airs-harness login"
    );
    let session = AirsSessionGuard::capture(home)?;
    let config: toml::Value = toml::from_str(&public_file(&home.join("config.toml"))?)
        .map_err(|_| anyhow::anyhow!("Invalid saved environment configuration"))?;
    let provider = config
        .get("model_providers")
        .and_then(|value| value.get("airs"))
        .context("Missing AIRS gateway configuration")?;
    let gateway = provider
        .get("base_url")
        .and_then(toml::Value::as_str)
        .context("Missing AIRS gateway URL")?;
    display_field(gateway)?;
    anyhow::ensure!(
        gateway.bytes().all(|byte| byte.is_ascii_graphic()),
        "Invalid saved gateway URL"
    );
    let parsed =
        url::Url::parse(gateway).map_err(|_| anyhow::anyhow!("Invalid saved gateway URL"))?;
    let loopback = match parsed.host() {
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        Some(url::Host::Domain("localhost")) => true,
        _ => false,
    };
    anyhow::ensure!(
        (parsed.scheme() == "https" || parsed.scheme() == "http" && loopback)
            && parsed.host_str().is_some()
            && parsed.username().is_empty()
            && parsed.password().is_none()
            && parsed.query().is_none()
            && parsed.fragment().is_none(),
        "Invalid saved gateway URL"
    );
    let binding_path = home.join("credential-binding.json");
    let binding_exists = match std::fs::symlink_metadata(&binding_path) {
        Ok(_) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(_) => anyhow::bail!("Cannot inspect saved credential binding"),
    };
    let (authentication, detail) = if binding_exists {
        let binding = airs_credentials::parse_binding(public_file(&binding_path)?.as_bytes())?;
        anyhow::ensure!(
            binding.schema_version == 1,
            "unsupported credential binding schema"
        );
        anyhow::ensure!(
            binding.gateway_url == gateway,
            "gateway changed; create a new environment and authenticate explicitly"
        );
        anyhow::ensure!(
            binding.credential_fingerprint.len() == 64
                && binding
                    .credential_fingerprint
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit()),
            "Invalid saved credential fingerprint"
        );
        match binding
            .source
            .as_ref()
            .context("logged out; run airs-harness login")?
        {
            Source::Keyring | Source::KeyringV2 => (
                format!(
                    "Workspace credential; saved OS-store binding {}",
                    binding.id
                ),
                "Saved; availability and gateway access not checked. Run airs-harness doctor --verify-access to check access.",
            ),
            Source::Oidc { identity } => {
                let issuer = display_field(&identity.config.issuer)?;
                let subject = display_field(&identity.subject)?;
                let audience = display_field(&identity.config.audience)?;
                display_field(&identity.config.client_id)?;
                anyhow::ensure!(
                    super::airs_oidc::fingerprint(gateway, identity)?
                        == binding.credential_fingerprint,
                    "Invalid saved identity binding"
                );
                (
                    format!(
                        "OIDC; saved identity metadata (not freshly authenticated): issuer {issuer:?}; subject {subject:?}; audience {audience:?}"
                    ),
                    "Saved; availability and gateway access not checked. Run airs-harness doctor --verify-access to check access.",
                )
            }
            Source::File { .. } | Source::Environment { .. } => {
                airs_credentials::resolve(&binding)?;
                (
                    format!("Workspace credential; reference binding {}", binding.id),
                    "Reference credential available locally; gateway access not checked.",
                )
            }
        }
    } else {
        let variable = provider
            .get("env_http_headers")
            .and_then(|value| value.get("x-portkey-api-key"))
            .and_then(toml::Value::as_str)
            .context("run airs-harness login to configure credentials")?;
        anyhow::ensure!(
            !variable.is_empty()
                && variable.len() <= 128
                && variable
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'),
            "Invalid credential environment variable name"
        );
        let token = std::env::var(variable)
            .context("credential environment variable is missing; run airs-harness login")?;
        airs_credentials::validate_token(&token)?;
        (
            format!("Workspace credential from {variable}"),
            "Environment credential available locally; gateway access not checked.",
        )
    };
    session.check()?;
    Ok(Inspection {
        gateway: gateway.to_owned(),
        authentication,
        detail,
    })
}

#[cfg(test)]
#[path = "airs_status_tests.rs"]
mod tests;
