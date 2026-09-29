//! Branded startup before the process commits its immutable environment home.
mod forms;

use super::airs_access;
use super::airs_credentials;
use super::airs_environment;
use super::airs_harness;
use super::airs_login;
use super::airs_oidc;
use super::airs_typesafe;
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
    /// Change an environment's authentication (airs env auth).
    UpdateAuth,
}

/// Menu positions in the sign-in loop below.
const COMPANY: usize = 0;
const WORKSPACE_KEY: usize = 1;

/// What the user chose in the update menu for an already signed-in environment.
#[derive(Debug, PartialEq, Eq)]
enum Update {
    /// Run the sign-in loop with this action; `replace` is the confirmed consent.
    SignIn { action: usize, replace: bool },
    /// Send the connectivity test with the current credential only.
    Test,
}

fn update_items(current: &airs_login::Method) -> Vec<(OnboardingMenuItem, Update)> {
    let replace = |action| Update::SignIn {
        action,
        replace: true,
    };
    let mut items = match current {
        airs_login::Method::WorkspaceKey { .. } => vec![
            (
                item(
                    "Replace the workspace API key",
                    "Enter a new key from your administrator; input is hidden",
                ),
                replace(WORKSPACE_KEY),
            ),
            (
                item(
                    "Switch to company SSO",
                    "Sign in with your browser instead of a key",
                ),
                replace(COMPANY),
            ),
        ],
        airs_login::Method::Company { .. } => vec![
            (
                item("Sign in again", "Refresh the same company identity"),
                Update::SignIn {
                    action: COMPANY,
                    replace: false,
                },
            ),
            (
                item(
                    "Change company SSO user or settings",
                    "Another user, issuer, client ID or audience",
                ),
                replace(COMPANY),
            ),
            (
                item(
                    "Switch to a workspace API key",
                    "Replace company sign-in with a key; input is hidden",
                ),
                replace(WORKSPACE_KEY),
            ),
        ],
    };
    items.push((
        item(
            "Test the current credential",
            "Send one minimal inference request; nothing changes",
        ),
        Update::Test,
    ));
    items
}

/// Explain the consequences once, then require an explicit choice to replace.
fn confirm_replacement(ui: &mut AirsOnboarding, environment: Option<&str>) -> anyhow::Result<bool> {
    if ui.message(&OnboardingProgress {
        title: "Before you replace this credential".into(),
        detail: airs_login::REPLACEMENT_NOTICE.into(),
        link: None,
    })? == OnboardingResult::Cancelled
    {
        return Ok(false);
    }
    Ok(selected(
        ui.menu(
            "Replace this environment's credential?",
            &[
                item(
                    "Replace and test access",
                    "Sign in with the new credential, then send one test request",
                ),
                item("Keep the current credential", "Exit without changes"),
            ],
        )?,
        environment,
    )? == 0)
}

fn unchanged(environment: Option<&str>) -> anyhow::Error {
    let command = airs_environment::command(environment);
    anyhow::anyhow!(
        "No changes were made; this environment keeps its current credential. Update it later with {command} env auth."
    )
}

fn item(label: &str, detail: &str) -> OnboardingMenuItem {
    OnboardingMenuItem {
        label: label.into(),
        detail: detail.into(),
    }
}

fn cancelled(environment: Option<&str>) -> anyhow::Error {
    let command = airs_environment::command(environment);
    anyhow::anyhow!(
        "Sign-in cancelled. Existing environments are preserved; resume with {command} login."
    )
}

fn selected<T>(result: OnboardingResult<T>, environment: Option<&str>) -> anyhow::Result<T> {
    match result {
        OnboardingResult::Selected(value) => Ok(value),
        OnboardingResult::Cancelled => Err(cancelled(environment)),
    }
}

fn context(selection: &airs_environment::Selection) -> OnboardingContext {
    OnboardingContext {
        environment: selection.name.clone().unwrap_or_else(|| "default".into()),
        gateway: Some(selection.gateway.clone()),
    }
}

fn options(home: &Path, overrides: &[String]) -> OnboardingOptions {
    let animations_in = |path: &Path| {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| toml::from_str::<toml::Value>(&text).ok())
            .and_then(|config| config.get("tui")?.get("animations")?.as_bool())
    };
    // The environment's own config.toml overrides harness-wide settings.toml.
    let mut animations = animations_in(&home.join("config.toml"))
        .or_else(|| {
            codex_utils_home_dir::airs_settings_path()
                .and_then(|path| animations_in(path.as_path()))
        })
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
    let mut created_environment = false;
    if selection.is_none() {
        let choices = airs_environment::choices(root)?;
        selection = if !creating && !choices.is_empty() {
            Some(pick(&mut ui, root, &choices, None)?)
        } else {
            let defaults = match &entry {
                Entry::Create(args) => args.clone(),
                _ => airs_harness::SetupArgs::default(),
            };
            created_environment = true;
            Some(forms::create(&mut ui, root, requested, &defaults)?)
        };
    }
    let mut selection = selection.context("No environment selected")?;
    let mut retry_action = None;
    let mut replace = false;
    if matches!(entry, Entry::Login(_) | Entry::UpdateAuth)
        && let Some(current) = airs_login::current_method(&selection.home)?
    {
        ui.set_context(context(&selection));
        let (items, updates): (Vec<_>, Vec<_>) = update_items(&current).into_iter().unzip();
        let title = format!("Update authentication · now {}", current.describe());
        let update = match ui.menu(&title, &items)? {
            OnboardingResult::Selected(index) => &updates[index],
            OnboardingResult::Cancelled => return Err(unchanged(selection.name.as_deref())),
        };
        match update {
            Update::Test => {
                verify(&mut ui, &selection, /*note*/ None).await?;
                return Ok(selection.name);
            }
            Update::SignIn {
                action,
                replace: confirm,
            } => {
                if *confirm && !confirm_replacement(&mut ui, selection.name.as_deref())? {
                    return Err(unchanged(selection.name.as_deref()));
                }
                replace = *confirm;
                retry_action = Some(*action);
            }
        }
    }
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
            None => selected(
                ui.menu("Sign in to continue", &actions)?,
                selection.name.as_deref(),
            )?,
        };
        if action == 2 {
            selection = pick(&mut ui, root, &choices, selection.name.as_deref())?;
            if matches!(entry, Entry::Session)
                && !airs_login::needs_login(&selection.home, |name| {
                    std::env::var_os(name).is_some()
                })?
            {
                return Ok(selection.name);
            }
            continue;
        }
        let result = if action == WORKSPACE_KEY {
            // Secure key entry owns raw mode itself. Never copy a secret into render state.
            drop(ui);
            eprintln!(
                "PRISMA AIRS · {}\nGateway: {}",
                selection.name.as_deref().unwrap_or("default"),
                selection.gateway
            );
            let result = airs_credentials::login(
                &selection.home,
                &airs_credentials::LoginArgs {
                    replace,
                    ..Default::default()
                },
                /*stdin_key*/ true,
            )
            .await;
            ui = AirsOnboarding::open(context(&selection), display)?;
            result
        } else {
            let mut args = forms::company(&mut ui, &selection.home, selection.name.as_deref())?;
            args.replace = replace;
            if matches!(preferred, airs_oidc::LoginFlow::Browser) {
                preferred = match selected(
                    ui.menu(
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
                    )?,
                    selection.name.as_deref(),
                )? {
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
                    OnboardingResult::Cancelled => {
                        return Err(cancelled(selection.name.as_deref()));
                    }
                }
            }
        };
        if let Err(error) = result {
            // Signing in with another key or identity is an update, not a dead end.
            if !replace && error.is::<airs_credentials::CredentialChanged>() {
                if confirm_replacement(&mut ui, selection.name.as_deref())? {
                    replace = true;
                    retry_action = Some(action);
                    preferred = airs_oidc::LoginFlow::Browser;
                    continue;
                }
                return Err(unchanged(selection.name.as_deref()));
            }
            selected(
                ui.message(&OnboardingProgress {
                    title: "Sign-in needs your attention".into(),
                    detail: format!("{error:#}")
                        .replace("airs-harness ", "airs ")
                        .replace(".: ", ". "),
                    link: None,
                })?,
                selection.name.as_deref(),
            )?;
            if selected(
                ui.menu(
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
                )?,
                selection.name.as_deref(),
            )? == 0
            {
                retry_action = Some(action);
                preferred = airs_oidc::LoginFlow::Browser;
            }
            continue;
        }
        verify(
            &mut ui,
            &selection,
            replace.then_some(
                "This environment now uses the new credential. Restart any other open AIRS sessions in it.",
            ),
        )
        .await?;
        if created_environment
            && airs_typesafe::read(&selection.home).is_ok_and(|settings| settings.is_none())
            && std::env::var_os(airs_typesafe::KEY_VARIABLE).is_none()
            && selected(
                ui.menu(
                    "Optional: red-team judge",
                    &[
                        item(
                            "Continue to AIRS",
                            "Add a TypeSafe key later with airs env typesafe set",
                        ),
                        item(
                            "Add a TypeSafe judge API key",
                            "Enables red-team ASR scoring; input is hidden",
                        ),
                    ],
                )?,
                selection.name.as_deref(),
            )? == 1
        {
            // Secure key entry owns raw mode itself. Never copy a secret into render state.
            drop(ui);
            match airs_typesafe::set_interactive(&selection.home, None, None) {
                Ok(_) => eprintln!("TypeSafe key saved for this environment."),
                Err(error) => eprintln!("TypeSafe key not saved: {error}"),
            }
        }
        return Ok(selection.name);
    }
}

fn pick(
    ui: &mut AirsOnboarding,
    root: &Path,
    choices: &[(String, String)],
    environment: Option<&str>,
) -> anyhow::Result<airs_environment::Selection> {
    let actions: Vec<_> = choices
        .iter()
        .map(|(name, gateway)| item(name, gateway))
        .collect();
    let index = selected(ui.menu("Choose an environment", &actions)?, environment)?;
    airs_environment::resolve(root, Some(&choices[index].0))?
        .context("Selected environment is unavailable")
}

async fn verify(
    ui: &mut AirsOnboarding,
    selection: &airs_environment::Selection,
    note: Option<&str>,
) -> anyhow::Result<()> {
    let command = airs_environment::command(selection.name.as_deref());
    loop {
        let (_sender, receiver) = watch::channel(OnboardingProgress {
            title: "Credential saved · checking gateway access".into(),
            detail: airs_access::DISCLOSURE.into(),
            link: None,
        });
        let verification = match ui
            .wait(airs_access::verify(&selection.home), receiver)
            .await?
        {
            OnboardingResult::Selected(result) => result,
            OnboardingResult::Cancelled => anyhow::bail!(
                "Credential saved; access check cancelled. Resume with {command} or run {command} doctor --verify-access."
            ),
        };
        if ui.message(&OnboardingProgress {
            title: if verification.outcome.is_ok() {
                "You're ready to use AIRS"
            } else {
                "Credential saved · gateway access needs attention"
            }
            .into(),
            detail: match note {
                Some(note) => format!(
                    "{}\n{note}",
                    verification.after_login(selection.name.as_deref())
                ),
                None => verification.after_login(selection.name.as_deref()),
            },
            link: None,
        })? == OnboardingResult::Cancelled
        {
            anyhow::bail!(
                "Credential saved. Start {command} when ready, or inspect access with {command} doctor --verify-access."
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
                    &format!("Use {command} doctor --verify-access when ready"),
                ),
            ],
        )?;
        match next {
            OnboardingResult::Selected(0) => return Ok(()),
            OnboardingResult::Selected(1) => {}
            _ => anyhow::bail!("{}", verification.after_login(selection.name.as_deref())),
        }
    }
}

#[cfg(test)]
#[path = "welcome_tests.rs"]
mod tests;
