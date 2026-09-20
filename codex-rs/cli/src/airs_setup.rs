//! First-run setup for the standalone harness. Explicit gateway flags stay noninteractive.
use super::airs_environment;
use super::airs_harness::SetupArgs;
use super::airs_login;
use super::airs_oidc::LoginFlow;
use anyhow::Context;
use std::io::BufRead;
use std::io::IsTerminal;
use std::io::Write;
use std::path::Path;

fn collect(
    requested_name: Option<&str>,
    args: &SetupArgs,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> anyhow::Result<(String, SetupArgs)> {
    writeln!(
        output,
        "Welcome to Prisma AIRS Harness\n\nConnect to your organization's AI Gateway, then sign in.\nAsk your administrator for the gateway URL if you don't have it."
    )?;
    let name = match requested_name {
        Some(name) => name.to_owned(),
        None => {
            let name = airs_login::prompt(input, output, "Environment name [work]: ")?;
            if name.is_empty() {
                "work".to_owned()
            } else {
                name
            }
        }
    };
    airs_environment::validate_name(&name)?;
    let gateway_url = airs_login::prompt(input, output, "AI Gateway URL: ")?;
    anyhow::ensure!(!gateway_url.is_empty(), "An AI Gateway URL is required");
    let setup = SetupArgs {
        gateway_url,
        ..args.clone()
    };
    Ok((name, setup))
}

pub(super) async fn interactive(
    root: &Path,
    requested_name: Option<&str>,
    args: &SetupArgs,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        std::io::stdin().is_terminal() && std::io::stderr().is_terminal(),
        "Guided setup requires an interactive terminal. Automation must supply env create NAME --gateway-url URL"
    );
    if codex_tui::AirsOnboarding::supported() {
        let name = super::airs_welcome::run(
            root,
            requested_name,
            super::airs_welcome::Entry::Create(args.clone()),
            &[],
        )
        .await?;
        airs_environment::select(root, name.as_deref())?;
        return Ok(());
    }
    let (name, args) = collect(
        requested_name,
        args,
        &mut std::io::stdin().lock(),
        &mut std::io::stderr().lock(),
    )?;
    // setup holds the registry lock, refuses existing names and atomically
    // publishes the new environment. Never mutate a previous configuration.
    airs_environment::setup(root, &name, &args)?;
    airs_environment::select(root, Some(&name))?;
    let home = codex_core::config::find_codex_home()?;
    let command = airs_environment::command(Some(&name));
    airs_login::interactive(home.as_path(), LoginFlow::Browser, Some(&name))
        .await
        .with_context(|| {
            format!("Environment {name} was created. Resume sign-in with: {command} login")
        })?;
    offer_typesafe(
        home.as_path(),
        &mut std::io::stdin().lock(),
        &mut std::io::stderr().lock(),
        |home| super::airs_typesafe::set_interactive(home, None, None).map(|_| ()),
    )?;
    eprintln!("Check access: {command} doctor --verify-access");
    eprintln!("Start a session: {command}");
    Ok(())
}

/// Optional judge key: Enter or anything but y/yes skips; a refusal never fails setup.
pub(super) fn offer_typesafe(
    home: &Path,
    input: &mut impl BufRead,
    output: &mut impl Write,
    set: impl FnOnce(&Path) -> anyhow::Result<()>,
) -> anyhow::Result<bool> {
    let answer = airs_login::prompt(
        input,
        output,
        "Optional: save a TypeSafe judge API key for red-team ASR scoring? [y/N]: ",
    )?;
    if !matches!(answer.to_ascii_lowercase().as_str(), "y" | "yes") {
        writeln!(output, "Skipped. Add one later with airs env typesafe set.")?;
        return Ok(false);
    }
    set(home)?;
    writeln!(output, "TypeSafe key saved for this environment.")?;
    Ok(true)
}

#[cfg(test)]
#[path = "airs_setup_tests.rs"]
mod tests;
