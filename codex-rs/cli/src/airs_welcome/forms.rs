use super::airs_credentials;
use super::airs_environment;
use super::airs_harness;
use super::airs_login;
use super::cancelled;
use super::item;
use super::selected;
use anyhow::Context;
use codex_airs_identity::IdentityConfig;
use codex_tui::AirsOnboarding;
use codex_tui::OnboardingContext;
use codex_tui::OnboardingInput;
use codex_tui::OnboardingProgress;
use std::path::Path;

pub(super) fn input(
    ui: &mut AirsOnboarding,
    label: &str,
    help: &str,
    initial: &str,
    environment: Option<&str>,
    validate: impl Fn(&str) -> anyhow::Result<()>,
) -> anyhow::Result<String> {
    let mut field = OnboardingInput {
        title: "Connect your environment".into(),
        label: label.into(),
        help: help.into(),
        value: initial.into(),
        error: None,
    };
    loop {
        let value = selected(ui.input(field.clone())?, environment)?;
        match validate(&value) {
            Ok(()) => return Ok(value),
            Err(error) => {
                field.value = value;
                field.error = Some(error.to_string());
            }
        }
    }
}

pub(super) fn create(
    ui: &mut AirsOnboarding,
    root: &Path,
    requested: Option<&str>,
    defaults: &airs_harness::SetupArgs,
) -> anyhow::Result<airs_environment::Selection> {
    selected(
        ui.menu(
            "Welcome to Prisma AIRS",
            &[
                item(
                    "Connect an environment",
                    "Use your organization's AI Gateway and company sign-in",
                ),
                item(
                    "Exit",
                    "Get connection details from your administrator first",
                ),
            ],
        )?,
        None,
    )
    .and_then(|choice| {
        if choice == 0 {
            Ok(())
        } else {
            Err(cancelled(None))
        }
    })?;
    let name = input(
        ui,
        "Environment name",
        "A short name such as work or staging.",
        requested.unwrap_or("work"),
        None,
        |name| {
            airs_environment::validate_name(name)?;
            anyhow::ensure!(
                !airs_environment::choices(root)?
                    .iter()
                    .any(|(existing, _)| existing == name),
                "That environment already exists. Choose another name, or use airs env use NAME."
            );
            Ok(())
        },
    )?;
    let mut args = defaults.clone();
    args.gateway_url = input(
        ui,
        "AI Gateway URL",
        "Use the inference API root supplied by your administrator.",
        &args.gateway_url,
        None,
        |value| {
            let candidate = airs_harness::SetupArgs {
                gateway_url: value.into(),
                ..defaults.clone()
            };
            airs_harness::configuration(&candidate, root).map(|_| ())
        },
    )?;
    ui.set_context(OnboardingContext {
        environment: name.clone(),
        gateway: Some(args.gateway_url.clone()),
    });
    if selected(
        ui.menu(
            "Create this environment?",
            &[
                item(
                    "Create environment and sign in",
                    "Keep its credentials and conversation history together",
                ),
                item("Cancel", "Nothing has been created yet"),
            ],
        )?,
        None,
    )? != 0
    {
        return Err(cancelled(None));
    }
    airs_environment::create(root, &name, &args)?;
    airs_environment::resolve(root, Some(&name))?
        .context("Created environment could not be resolved")
}

pub(super) fn company(
    ui: &mut AirsOnboarding,
    home: &Path,
    environment: Option<&str>,
) -> anyhow::Result<airs_credentials::LoginArgs> {
    let bound = airs_login::existing_binding(home)?.and_then(|binding| match binding.source {
        Some(airs_credentials::Source::Oidc { identity }) => Some(identity.config),
        _ => None,
    });
    let saved = if let Some(config) = bound {
        airs_login::validate_identity(&config)?;
        Some(config)
    } else {
        match airs_login::read_settings(home) {
            Ok(value) => value,
            Err(_) => {
                selected(ui.message(&OnboardingProgress {
                    title: "Check your company sign-in settings".into(),
                    detail: "Saved public connection settings are invalid or unavailable. Enter replacement settings from your administrator. Existing credentials and history are preserved.".into(), link: None,
                })?, environment)?;
                None
            }
        }
    };
    let config = if let Some(config) = saved {
        let choice = selected(
            ui.menu(
                "Company sign-in settings",
                &[
                    item("Continue with saved settings", &config.issuer),
                    item(
                        "Review or change public settings",
                        "A different identity requires a separate environment",
                    ),
                ],
            )?,
            environment,
        )?;
        if choice == 0 {
            config
        } else {
            collect_company(ui, Some(config), environment)?
        }
    } else {
        collect_company(ui, /*previous*/ None, environment)?
    };
    let args = airs_credentials::LoginArgs {
        issuer_url: Some(config.issuer),
        oidc_client_id: Some(config.client_id),
        audience: Some(config.audience),
        ..Default::default()
    };
    airs_login::remember_settings(home, &args)?;
    Ok(args)
}

fn collect_company(
    ui: &mut AirsOnboarding,
    previous: Option<IdentityConfig>,
    environment: Option<&str>,
) -> anyhow::Result<IdentityConfig> {
    let previous = previous.unwrap_or(IdentityConfig {
        issuer: String::new(),
        client_id: String::new(),
        audience: String::new(),
    });
    let issuer = input(
        ui,
        "Company issuer URL",
        "The HTTPS issuer from your administrator. Your password stays in the browser.",
        &previous.issuer,
        environment,
        |value| {
            airs_login::validate_identity(&IdentityConfig {
                issuer: value.into(),
                client_id: "public-client".into(),
                audience: "gateway".into(),
            })
        },
    )?;
    let client_id = input(
        ui,
        "Public client ID",
        "An administrator-provided public client ID, never a client secret.",
        &previous.client_id,
        environment,
        airs_login::public_field,
    )?;
    let audience = input(
        ui,
        "Gateway audience",
        "The audience registered for your AI Gateway.",
        &previous.audience,
        environment,
        airs_login::public_field,
    )?;
    let config = IdentityConfig {
        issuer,
        client_id,
        audience,
    };
    airs_login::validate_identity(&config)?;
    Ok(config)
}
