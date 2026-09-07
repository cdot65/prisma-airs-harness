//! OIDC storage and CLI orchestration. Callers hold the environment lock.
use super::airs_credentials::Binding;
use super::airs_credentials::LoginArgs;
use super::airs_credentials::SERVICE;
use super::airs_credentials::Source;
use super::airs_environment;
use anyhow::Context;
use codex_airs_identity::CredentialStore;
use codex_airs_identity::Identity;
use codex_airs_identity::IdentityConfig;
use codex_airs_identity::Provider;
use codex_airs_identity::Tokens;
use codex_keyring_store::KeyringStore;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
enum Stored {
    Active { tokens: Tokens },
    RefreshPending,
}

pub enum LoginFlow {
    Browser,
    Device,
}

fn save(binding: &Binding, stored: &Stored) -> anyhow::Result<()> {
    CredentialStore
        .save(
            SERVICE,
            &binding.id.to_string(),
            &serde_json::to_string(stored)?,
        )
        .map_err(|_| super::airs_credentials::credential_store_error())
}

pub(super) fn load_active(binding: &Binding) -> anyhow::Result<Tokens> {
    let Some(Source::Oidc { identity }) = &binding.source else {
        anyhow::bail!("not an OIDC binding");
    };
    let raw = CredentialStore
        .load(SERVICE, &binding.id.to_string())
        .map_err(|_| super::airs_credentials::credential_store_error())?
        .context("OIDC credential missing; sign in again")?;
    anyhow::ensure!(raw.len() <= 131_072, "invalid stored identity record");
    let stored: Stored = serde_json::from_str(&raw)
        .map_err(|_| anyhow::anyhow!("invalid stored identity record"))?;
    let Stored::Active { tokens } = stored else {
        anyhow::bail!(
            "previous refresh was interrupted; sign in again. The consumed token will not be retried"
        );
    };
    anyhow::ensure!(
        tokens.identity.config == identity.config
            && tokens.identity.subject == identity.subject
            && fingerprint(&binding.gateway_url, identity)? == binding.credential_fingerprint,
        "stored identity does not match this environment"
    );
    Ok(tokens)
}

pub(super) fn fingerprint(gateway: &str, identity: &Identity) -> anyhow::Result<String> {
    Ok(super::airs_credentials::fingerprint(
        &serde_json::to_string(&("oidc-v1", gateway, &identity.config, &identity.subject))?,
    ))
}

pub(super) async fn credential(binding: &Binding) -> anyhow::Result<String> {
    let previous = load_active(binding)?;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    if previous.expires_at > now + 30 {
        return Ok(previous.access_token);
    }
    // Discovery does not consume the refresh token. Finish it before marking
    // pending so an IdP metadata outage can be retried without losing a session.
    let provider = Provider::discover(previous.identity.config.clone()).await?;
    // Durable pending state precedes the request. Cancellation, network failure,
    // or failed persistence requires login; never replay a possibly consumed token.
    save(binding, &Stored::RefreshPending)?;
    let tokens = provider.refresh(&previous).await?;
    let access = tokens.access_token.clone();
    save(binding, &Stored::Active { tokens })?;
    Ok(access)
}

pub async fn login(home: &Path, args: &LoginArgs, flow: LoginFlow) -> anyhow::Result<()> {
    let _lock = airs_environment::lock(home)?;
    let config = IdentityConfig {
        issuer: args
            .issuer_url
            .clone()
            .context("OIDC login requires --issuer-url")?,
        client_id: args
            .oidc_client_id
            .clone()
            .context("OIDC login requires --oidc-client-id")?,
        audience: args
            .audience
            .clone()
            .context("OIDC login requires --audience")?,
    };
    let tokens = authenticate(config, flow).await?;
    let gateway_url = airs_environment::gateway(home)?;
    let credential_fingerprint = fingerprint(&gateway_url, &tokens.identity)?;
    let existing = if home.join("credential-binding.json").exists() {
        Some(super::airs_credentials::read_binding(home)?)
    } else {
        None
    };
    if let Some(previous) = &existing {
        anyhow::ensure!(
            previous.credential_fingerprint == credential_fingerprint,
            "this identity requires a new environment; existing history belongs to another credential"
        );
    }
    let subject = tokens.identity.subject.clone();
    let issuer = tokens.identity.config.issuer.clone();
    let binding = Binding {
        schema_version: 1,
        id: existing.map_or_else(Uuid::new_v4, |b| b.id),
        gateway_url,
        credential_fingerprint,
        source: Some(Source::Oidc {
            identity: tokens.identity.clone(),
        }),
    };
    save(&binding, &Stored::Active { tokens })?;
    super::airs_credentials::install_binding(home, &binding)?;
    println!(
        "Signed in through {issuer}. Verified subject: {subject}. Credentials stored in the OS store."
    );
    Ok(())
}

pub(super) async fn authenticate(
    config: IdentityConfig,
    flow: LoginFlow,
) -> anyhow::Result<Tokens> {
    // Fail before asking the user to authenticate if durable OS storage is unavailable.
    let probe = format!("oidc-probe-{}", Uuid::new_v4());
    CredentialStore
        .save(SERVICE, &probe, "storage-availability-check")
        .map_err(|_| super::airs_credentials::credential_store_error())?;
    CredentialStore
        .delete(SERVICE, &probe)
        .map_err(|_| anyhow::anyhow!("OS credential store cleanup failed"))?;
    let provider = Provider::discover(config).await?;
    let tokens = match flow {
        LoginFlow::Browser => {
            let login = provider.browser_login().await?;
            eprintln!("Open this URL to sign in:\n{}", login.authorization_url());
            let _ = webbrowser::open(login.authorization_url().as_str());
            tokio::select! {
                result = login.complete(&provider) => result?,
                _ = tokio::signal::ctrl_c() => anyhow::bail!("login cancelled"),
            }
        }
        LoginFlow::Device => {
            let login = provider.device_login().await?;
            eprintln!(
                "Open {} and enter code {}",
                login.verification_uri(),
                login.user_code()
            );
            tokio::select! {
                result = login.complete(&provider) => result?,
                _ = tokio::signal::ctrl_c() => anyhow::bail!("login cancelled"),
            }
        }
    };
    Ok(tokens)
}

pub(super) fn store_tokens(binding: &Binding, tokens: Tokens) -> anyhow::Result<()> {
    save(binding, &Stored::Active { tokens })
}
