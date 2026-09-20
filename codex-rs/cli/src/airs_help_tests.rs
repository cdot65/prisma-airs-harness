use super::*;
use clap::CommandFactory;

#[test]
fn standalone_help_tree_uses_the_public_command_in_examples_and_usage() {
    let mut pending = vec![command(crate::MultitoolCli::command())];
    while let Some(mut command) = pending.pop() {
        let help = command.render_long_help().to_string();
        assert!(!help.contains("codex "), "{}: {help}", command.get_name());
        assert!(!help.contains("`codex`"), "{}: {help}", command.get_name());
        assert!(
            !help.contains("airs-harness "),
            "{}: {help}",
            command.get_name()
        );
        pending.extend(command.get_subcommands().cloned());
    }
}

#[test]
fn standalone_login_help_describes_hidden_input() {
    let mut root = command(crate::MultitoolCli::command());
    let login = root.find_subcommand_mut("login").unwrap();
    insta::assert_snapshot!("airs_login_help", login.render_long_help().to_string());
}

#[test]
fn standalone_mcp_login_help_describes_manual_callback_input() {
    let mut root = command(crate::MultitoolCli::command());
    let mcp = root.find_subcommand_mut("mcp").unwrap();
    let login = mcp.find_subcommand_mut("login").unwrap();
    insta::assert_snapshot!("airs_mcp_login_help", login.render_long_help().to_string());
}

#[test]
fn standalone_help_exposes_local_work_and_gateway_configuration() {
    let mut command = command(crate::MultitoolCli::command())
        .bin_name("airs")
        .term_width(90)
        .disable_help_subcommand(true)
        .override_usage("airs [OPTIONS] [PROMPT]\n       airs [OPTIONS] <COMMAND> [ARGS]")
        .mut_arg("environment", |arg| arg.hide(false));
    // The test also runs in the upstream codex binary, where AIRS-only commands
    // are hidden by derive attributes. Make the presentation fixture explicit.
    for name in ["setup-mcp", "env", "cli"] {
        command = command.mut_subcommand(name, |subcommand| subcommand.hide(false));
    }
    insta::assert_snapshot!("airs_harness_help", command.render_help().to_string());
}

#[test]
fn standalone_environment_help_describes_the_complete_lifecycle() {
    let mut root = command(crate::MultitoolCli::command());
    let env = root.find_subcommand_mut("env").unwrap();
    insta::assert_snapshot!("airs_environment_help", env.render_long_help().to_string());
    let create = env.find_subcommand_mut("create").unwrap();
    insta::assert_snapshot!(
        "airs_environment_create_help",
        create.render_long_help().to_string()
    );
    let typesafe = env.find_subcommand_mut("typesafe").unwrap();
    insta::assert_snapshot!(
        "airs_environment_typesafe_help",
        typesafe.render_long_help().to_string()
    );
}

#[test]
fn standalone_doctor_help_discloses_optional_inference_probe() {
    let mut root = command(crate::MultitoolCli::command());
    let doctor = root.find_subcommand_mut("doctor").unwrap();
    insta::assert_snapshot!("airs_doctor_help", doctor.render_long_help().to_string());
}
