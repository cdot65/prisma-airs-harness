//! Branded startup before the process commits its immutable environment home.
mod forms;

use super::airs_access;
use super::airs_credentials;
use super::airs_environment;
use super::airs_harness;
use super::airs_login;
use super::airs_oidc;
use anyhow::Context;
use codex_tui::AirsOnboarding;
use codex_tui::OnboardingContext;
use codex_tui::OnboardingMenuItem;
use codex_tui::OnboardingOptions;
use codex_tui::OnboardingProgress;
use codex_tui::OnboardingResult;
use std::path::Path;
use tokio::sync::watch;

pub(super) enum Entry {
    Session,
    Login(airs_oidc::LoginFlow),
    Create(airs_harness::SetupArgs),
}

fn item(label: &str, detail: &str) -> OnboardingMenuItem {
    OnboardingMenuItem {
        label: label.into(),
        detail: detail.into(),
    }
}

fn cancelled() -> anyhow::Error {
    anyhow::anyhow!(
        "Sign-in cancelled. Existing environments are preserved; resume with airs login or airs --environment NAME login."
    )
}

fn selected<T>(result: OnboardingResult<T>) -> anyhow::Result<T> {
    match result {
        OnboardingResult::Selected(value) => Ok(value),
        OnboardingResult::Cancelled => Err(cancelled()),
    }
}

fn context(selection: &airs_environment::Selection) -> OnboardingContext {
    OnboardingContext {
        environment: selection.name.clone().unwrap_or_else(|| "default".into()),
        gateway: Some(selection.gateway.clone()),
    }
}

fn options(home: &Path, overrides: &[String]) -> OnboardingOptions {
    let mut animations = std::fs::read_to_string(home.join("config.toml"))
        .ok()
        .and_then(|text| toml::from_str::<toml::Value>(&text).ok())
        .and_then(|config| config.get("tui")?.get("animations")?.as_bool())
        .unwrap_or(true);
    for entry in overrides {
        if let Some((key, value)) = entry.split_once('=')
            && key.trim() == "tui.animations"
            && let Ok(value) = value.trim().parse::<bool>()
        {
            animations = value;
        }
    }
    OnboardingOptions {
        animations,
        color: std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty()),
    }
}

/// Return the selected name; only the caller binds the process after this completes.
pub(super) async fn run(
    root: &Path,
    requested: Option<&str>,
    entry: Entry,
    overrides: &[String],
) -> anyhow::Result<Option<String>> {
    let creating = matches!(entry, Entry::Create(_));
    let mut selection = if creating
        || (!root.join("config.toml").exists() && !root.join("environments.json").exists())
    {
        None
    } else {
        airs_environment::resolve(root, requested)?
    };
    if matches!(entry, Entry::Session)
        && let Some(selection) = &selection
        && !airs_login::needs_login(&selection.home, |name| std::env::var_os(name).is_some())?
    {
        return Ok(selection.name.clone());
    }
    let mut display = options(
        selection.as_ref().map_or(root, |selection| &selection.home),
        overrides,
    );
    let mut ui =
        AirsOnboarding::open(selection.as_ref().map(context).unwrap_or_default(), display)?;
    let mut preferred = match entry {
        Entry::Login(flow) => flow,
        _ => airs_oidc::LoginFlow::Browser,
    };
    if selection.is_none() {
        let choices = airs_environment::choices(root)?;
        selection = if !creating && !choices.is_empty() {
            Some(pick(&mut ui, root, &choices)?)
        } else {
            let defaults = match &entry {
                Entry::Create(args) => args.clone(),
                _ => airs_harness::SetupArgs::default(),
            };
            Some(forms::create(&mut ui, root, requested, &defaults)?)
        };
    }
    let mut selection = selection.context("No environment selected")?;
    let mut retry_action = None;
    loop {
        ui.set_context(context(&selection));
        display = options(&selection.home, overrides);
        ui.set_options(display);
        let choices = airs_environment::choices(root)?;
        let mut actions = vec![
            item(
                "Sign in with company SSO",
                "Continue securely in your browser",
            ),
            item(
                "Use a workspace API key",
                "Enter a key from your administrator; input is hidden",
            ),
        ];
        if choices.len() > 1 {
            actions.push(item(
                "Choose another environment",
                "Use another environment for this session",
            ));
        }
        let action = match retry_action.take() {
            Some(action) => action,
            None => selected(ui.menu("Sign in to continue", &actions)?)?,
        };
        if action == 2 {
            selection = pick(&mut ui, root, &choices)?;
            if matches!(entry, Entry::Session)
                && !airs_login::needs_login(&selection.home, |name| {
                    std::env::var_os(name).is_some()
                })?
            {
                return Ok(selection.name);
            }
            continue;
        }
        let result = if action == 1 {
            // Secure key entry owns raw mode itself. Never copy a secret into render state.
            drop(ui);
            eprintln!(
                "PRISMA AIRS · {}\nGateway: {}",
                selection.name.as_deref().unwrap_or("default"),
                selection.gateway
            );
            let result = airs_credentials::login(
                &selection.home,
                &airs_credentials::LoginArgs::default(),
                /*stdin_key*/ true,
            );
            ui = AirsOnboarding::open(context(&selection), display)?;
            result
        } else {
            let args = forms::company(&mut ui, &selection.home)?;
            if matches!(preferred, airs_oidc::LoginFlow::Browser) {
                preferred = match selected(ui.menu(
                    "How would you like to sign in?",
                    &[
                        item(
                            "Open browser on this machine",
                            "Recommended for your desktop",
                        ),
                        item(
                            "Use device authorization",
                            "Sign in on another device; useful over SSH",
                        ),
                        item(
                            "Show the full browser URL",
                            "Manual browser flow with the existing text prompt",
                        ),
                    ],
                )?)? {
                    0 => airs_oidc::LoginFlow::Browser,
                    1 => airs_oidc::LoginFlow::Device,
                    _ => airs_oidc::LoginFlow::BrowserManual,
                };
            }
            if matches!(preferred, airs_oidc::LoginFlow::BrowserManual) {
                drop(ui);
                eprintln!(
                    "PRISMA AIRS · Manual company sign-in\nKeep this terminal open until sign-in completes. Ctrl+C cancels."
                );
                let result = airs_oidc::login(&selection.home, &args, preferred).await;
                ui = AirsOnboarding::open(context(&selection), display)?;
                result
            } else {
                let (sender, receiver) = watch::channel(OnboardingProgress {
                    title: "Preparing company sign-in".into(),
                    detail: "Checking the selected environment.".into(),
                    link: None,
                });
                match ui
                    .wait(
                        airs_oidc::login_with_progress(&selection.home, &args, preferred, sender),
                        receiver,
                    )
                    .await?
                {
                    OnboardingResult::Selected(result) => result,
                    OnboardingResult::Cancelled => return Err(cancelled()),
                }
            }
        };
        if let Err(error) = result {
            selected(
                ui.message(&OnboardingProgress {
                    title: "Sign-in needs your attention".into(),
                    detail: format!("{error:#}")
                        .replace("airs-harness ", "airs ")
                        .replace(".: ", ". "),
                    link: None,
                })?,
            )?;
            if selected(ui.menu(
                "Choose your next step",
                &[
                    item(
                        "Try sign-in again",
                        "Check storage access and connection settings first",
                    ),
                    item(
                        "Back to sign-in options",
                        "Use a workspace key or another environment",
                    ),
                ],
            )?)? == 0
            {
                retry_action = Some(action);
                preferred = airs_oidc::LoginFlow::Browser;
            }
            continue;
        }
        verify(&mut ui, &selection.home).await?;
        return Ok(selection.name);
    }
}

fn pick(
    ui: &mut AirsOnboarding,
    root: &Path,
    choices: &[(String, String)],
) -> anyhow::Result<airs_environment::Selection> {
    let actions: Vec<_> = choices
        .iter()
        .map(|(name, gateway)| item(name, gateway))
        .collect();
    let index = selected(ui.menu("Choose an environment", &actions)?)?;
    airs_environment::resolve(root, Some(&choices[index].0))?
        .context("Selected environment is unavailable")
}

async fn verify(ui: &mut AirsOnboarding, home: &Path) -> anyhow::Result<()> {
    loop {
        let (_sender, receiver) = watch::channel(OnboardingProgress {
            title: "Credential saved · checking gateway access".into(),
            detail: airs_access::DISCLOSURE.into(),
            link: None,
        });
        let verification = match ui.wait(airs_access::verify(home), receiver).await? {
            OnboardingResult::Selected(result) => result,
            OnboardingResult::Cancelled => anyhow::bail!(
                "Credential saved; access check cancelled. Resume with airs or run airs doctor --verify-access."
            ),
        };
        if ui.message(&OnboardingProgress {
            title: if verification.outcome.is_ok() {
                "You're ready to use AIRS"
            } else {
                "Credential saved · gateway access needs attention"
            }
            .into(),
            detail: verification.after_login(),
            link: None,
        })? == OnboardingResult::Cancelled
        {
            anyhow::bail!(
                "Credential saved. Start AIRS when ready, or inspect access with airs doctor --verify-access."
            );
        }
        if verification.outcome.is_ok() {
            return Ok(());
        }
        let next = ui.menu(
            "Your credential is saved",
            &[
                item(
                    "Continue to AIRS",
                    "Gateway permissions may still prevent requests",
                ),
                item(
                    "Retry access check",
                    "Send one more minimal inference request",
                ),
                item(
                    "Exit and fix gateway access",
                    "Use airs doctor --verify-access when ready",
                ),
            ],
        )?;
        match next {
            OnboardingResult::Selected(0) => return Ok(()),
            OnboardingResult::Selected(1) => {}
            _ => anyhow::bail!("{}", verification.after_login()),
        }
    }
}

#[cfg(test)]
#[path = "welcome_tests.rs"]
mod tests;
