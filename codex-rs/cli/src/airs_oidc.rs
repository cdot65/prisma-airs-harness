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
use codex_login::auth::CredentialRecovery;
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
    SignInRequired,
}

#[derive(Clone, Copy)]
pub enum LoginFlow {
    Browser,
    BrowserManual,
    Device,
}

fn save(binding: &Binding, stored: &Stored) -> anyhow::Result<()> {
    save_in(binding, stored, &CredentialStore)
}

fn save_in(binding: &Binding, stored: &Stored, store: &impl KeyringStore) -> anyhow::Result<()> {
    store
        .save(
            SERVICE,
            &binding.id.to_string(),
            &serde_json::to_string(stored)?,
        )
        .map_err(|error| super::airs_storage_error::report(error, "save identity"))
}

pub(super) fn load_active(binding: &Binding) -> anyhow::Result<Tokens> {
    load_active_from(binding, &CredentialStore)
}

fn load_active_from(binding: &Binding, store: &impl KeyringStore) -> anyhow::Result<Tokens> {
    let Some(Source::Oidc { identity }) = &binding.source else {
        anyhow::bail!("not an OIDC binding");
    };
    let raw = store
        .load(SERVICE, &binding.id.to_string())
        .map_err(|error| super::airs_storage_error::report(error, "read identity"))?
        .context(CredentialRecovery::SignInRequired)?;
    anyhow::ensure!(raw.len() <= 131_072, "invalid stored identity record");
    let stored: Stored = serde_json::from_str(&raw)
        .map_err(|_| anyhow::anyhow!("invalid stored identity record"))?;
    let tokens = match stored {
        Stored::Active { tokens } => tokens,
        Stored::RefreshPending => return Err(CredentialRecovery::OutcomeUnknown.into()),
        Stored::SignInRequired => return Err(CredentialRecovery::SignInRequired.into()),
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

pub(super) async fn discover_provider(config: IdentityConfig) -> anyhow::Result<Provider> {
    let policy = super::airs_application_network::load()
        .await
        .map_err(|_| codex_http_client::NetworkPolicyDenied::Unavailable)?;
    Provider::discover(config, policy).await
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
    let provider = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        discover_provider(previous.identity.config.clone()),
    )
    .await
    .context(CredentialRecovery::TemporarilyUnavailable)?
    .map_err(|error| {
        if error
            .downcast_ref::<codex_http_client::NetworkPolicyDenied>()
            .is_some()
        {
            error.context(CredentialRecovery::PolicyDenied)
        } else {
            error.context(CredentialRecovery::TemporarilyUnavailable)
        }
    })?;
    // Durable pending state precedes the request. Cancellation, network failure,
    // or failed persistence requires login; never replay a possibly consumed token.
    refresh_after_admission(
        binding,
        provider
            .prepare_refresh(&previous)
            .map(codex_airs_identity::PreparedRefresh::complete),
        &CredentialStore,
    )
    .await
}

async fn refresh_after_admission(
    binding: &Binding,
    admitted: anyhow::Result<impl std::future::Future<Output = anyhow::Result<Tokens>>>,
    store: &impl KeyringStore,
) -> anyhow::Result<String> {
    // Admission errors prove no request started and preserve the original bytes.
    let exchange = admitted?;
    save_in(binding, &Stored::RefreshPending, store)?;
    // From this point even policy revocation is ambiguous: never restore a
    // rotating predecessor after the exchange could have been polled.
    complete_refresh(binding, exchange.await, store).await
}

async fn complete_refresh(
    binding: &Binding,
    refreshed: anyhow::Result<Tokens>,
    store: &impl KeyringStore,
) -> anyhow::Result<String> {
    let tokens = match refreshed {
        Ok(tokens) => tokens,
        Err(error) => {
            if matches!(
                error.downcast_ref::<codex_airs_identity::TokenExchangeError>(),
                Some(codex_airs_identity::TokenExchangeError::RefreshRejected)
            ) {
                // Retain a definitive rejection across later helper invocations.
                // This record contains no predecessor that could be replayed.
                save_in(binding, &Stored::SignInRequired, store)?;
                return Err(CredentialRecovery::SignInRequired.into());
            }
            return Err(CredentialRecovery::OutcomeUnknown.into());
        }
    };
    let access = tokens.access_token.clone();
    let returned = Stored::Active { tokens };
    // Retry persistence of this exact generation, never the consumed exchange.
    let mut persisted = save_in(binding, &returned, store);
    for delay in [100, 400] {
        if persisted.is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        persisted = save_in(binding, &returned, store);
    }
    persisted?;
    Ok(access)
}

/// Explicit recovery uses signed issuer/subject evidence and never changes the binding,
/// route or logout epoch. Ordinary login remains a separate session boundary.
pub(super) async fn restore_session(home: &Path, flow: LoginFlow) -> anyhow::Result<()> {
    let attempt = super::airs_auth_lifecycle::LoginAttempt::begin(home)?;
    let session = codex_utils_home_dir::airs_session::AirsSessionGuard::capture(home)?;
    let _lock = airs_environment::lock(home)?;
    session.check()?;
    let binding = super::airs_credentials::read_binding(home)?;
    let Some(Source::Oidc { identity }) = &binding.source else {
        anyhow::bail!(
            "This environment uses a workspace credential; inspect airs env status to repair its configured source"
        );
    };
    let tokens = authenticate(identity.config.clone(), flow).await?;
    ensure_restore_identity(&binding, &tokens.identity)?;
    session.check()?;
    super::airs_credentials::persist_oidc_binding(
        home,
        &binding,
        &serde_json::to_string(&Stored::Active { tokens })?,
        || {
            session.check()?;
            attempt.complete_restore()
        },
    )?;
    println!(
        "Sign-in restored for the same verified identity. Return to your open harness session and retry the paused request."
    );
    Ok(())
}

fn ensure_restore_identity(binding: &Binding, returned: &Identity) -> anyhow::Result<()> {
    let Some(Source::Oidc { identity }) = &binding.source else {
        anyhow::bail!("Expected an OIDC credential binding");
    };
    anyhow::ensure!(
        returned.config == identity.config
            && returned.subject == identity.subject
            && fingerprint(&binding.gateway_url, returned)? == binding.credential_fingerprint,
        "A different identity cannot restore this conversation; existing credentials were not changed. To switch this environment to another identity, run airs env auth"
    );
    Ok(())
}

#[cfg(test)]
#[path = "airs_oidc_restore_tests.rs"]
mod restore_tests;

pub async fn login(home: &Path, args: &LoginArgs, flow: LoginFlow) -> anyhow::Result<()> {
    login_inner(home, args, flow, /*progress*/ None).await
}

pub(super) async fn login_with_progress(
    home: &Path,
    args: &LoginArgs,
    flow: LoginFlow,
    progress: tokio::sync::watch::Sender<codex_tui::OnboardingProgress>,
) -> anyhow::Result<()> {
    login_inner(home, args, flow, Some(&progress)).await
}

async fn login_inner(
    home: &Path,
    args: &LoginArgs,
    flow: LoginFlow,
    progress: Option<&tokio::sync::watch::Sender<codex_tui::OnboardingProgress>>,
) -> anyhow::Result<()> {
    let attempt = super::airs_auth_lifecycle::LoginAttempt::begin(home)?;
    let _lock = airs_environment::lock(home)?;
    super::airs_credentials::recover_pending(home)?;
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
    let tokens = authenticate_with_progress(config, flow, progress).await?;
    report_progress(
        progress,
        "Saving sign-in",
        "Storing the verified identity in your OS credential store.",
        /*link*/ None,
    );
    let gateway_url = airs_environment::gateway(home)?;
    let credential_fingerprint = fingerprint(&gateway_url, &tokens.identity)?;
    let (existing, replaced) =
        super::airs_credentials::existing_for(home, &credential_fingerprint, args.replace)?;
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
    super::airs_credentials::persist_oidc_binding(
        home,
        &binding,
        &serde_json::to_string(&Stored::Active { tokens })?,
        || attempt.commit(|| super::airs_credentials::activate(home, &binding, args.replace)),
    )?;
    if let Some(previous) = &replaced {
        super::airs_credentials::retire_replaced(home, previous).await;
    }
    if progress.is_none() {
        println!(
            "Signed in through {issuer}. Verified subject: {subject}. Credentials stored in the OS store."
        );
        if replaced.is_some() {
            println!(
                "This sign-in replaced the environment's previous credential. Restart open sessions in this environment."
            );
        }
    }
    Ok(())
}

pub(super) async fn authenticate(
    config: IdentityConfig,
    flow: LoginFlow,
) -> anyhow::Result<Tokens> {
    authenticate_with_progress(config, flow, /*progress*/ None).await
}

fn report_progress(
    progress: Option<&tokio::sync::watch::Sender<codex_tui::OnboardingProgress>>,
    title: &str,
    detail: &str,
    link: Option<String>,
) {
    if let Some(progress) = progress {
        progress.send_replace(codex_tui::OnboardingProgress {
            title: title.into(),
            detail: detail.into(),
            link,
        });
    }
}

async fn authenticate_with_progress(
    config: IdentityConfig,
    flow: LoginFlow,
    progress: Option<&tokio::sync::watch::Sender<codex_tui::OnboardingProgress>>,
) -> anyhow::Result<Tokens> {
    report_progress(
        progress,
        "Checking credential storage",
        "Your sign-in will be saved in the OS credential store.",
        /*link*/ None,
    );
    // Fail before asking the user to authenticate if durable OS storage is unavailable.
    let probe = format!("oidc-probe-{}", Uuid::new_v4());
    CredentialStore
        .save(SERVICE, &probe, "storage-availability-check")
        .map_err(|error| super::airs_storage_error::report(error, "check credential storage"))?;
    CredentialStore
        .delete(SERVICE, &probe)
        .map_err(|error| super::airs_storage_error::report(error, "clean up storage check"))?;
    report_progress(
        progress,
        "Contacting company sign-in",
        "Discovering your organization's sign-in service.",
        /*link*/ None,
    );
    let provider = discover_provider(config).await?;
    let tokens = match flow {
        LoginFlow::Browser | LoginFlow::BrowserManual => {
            let login = provider.browser_login().await?;
            let url = login.authorization_url().to_string();
            if progress.is_none() {
                eprintln!("Open this URL to sign in:\n{url}");
            }
            report_progress(
                progress,
                "Waiting for company sign-in",
                "Complete sign-in in your browser, then return here. Your company password stays in the browser.",
                Some(url.clone()),
            );
            if matches!(flow, LoginFlow::Browser) {
                let browser_url = url.clone();
                let opened =
                    tokio::task::spawn_blocking(move || webbrowser::open(&browser_url)).await;
                if !matches!(opened, Ok(Ok(()))) {
                    report_progress(
                        progress,
                        "Open company sign-in",
                        "The browser could not open automatically. Open the link below, or cancel and choose device authorization for SSH.",
                        Some(url),
                    );
                }
            }
            if progress.is_some() {
                // The onboarding screen owns cancellation and its terminal signal guard.
                login.complete(&provider).await?
            } else {
                tokio::select! {
                    result = login.complete(&provider) => result?,
                    _ = tokio::signal::ctrl_c() => anyhow::bail!("login cancelled"),
                }
            }
        }
        LoginFlow::Device => {
            let login = provider.device_login().await?;
            if progress.is_none() {
                eprintln!(
                    "Open {} and enter code {}",
                    login.verification_uri(),
                    login.user_code()
                );
            }
            report_progress(
                progress,
                "Authorize this device",
                &format!(
                    "Enter code {} in your browser, then return here.",
                    login.user_code()
                ),
                Some(login.verification_uri().to_string()),
            );
            if progress.is_some() {
                login.complete(&provider).await?
            } else {
                tokio::select! {
                    result = login.complete(&provider) => result?,
                    _ = tokio::signal::ctrl_c() => anyhow::bail!("login cancelled"),
                }
            }
        }
    };
    Ok(tokens)
}

pub(super) fn store_tokens(binding: &Binding, tokens: Tokens) -> anyhow::Result<()> {
    save(binding, &Stored::Active { tokens })
}

#[cfg(test)]
#[path = "airs_oidc_refresh_tests.rs"]
mod refresh_tests;
