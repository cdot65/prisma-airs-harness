use super::*;
use clap::CommandFactory;

#[test]
fn standalone_login_help_describes_hidden_input() {
    let mut root = command(crate::MultitoolCli::command());
    let login = root.find_subcommand_mut("login").unwrap();
    insta::assert_snapshot!("airs_login_help", login.render_long_help().to_string());
}

#[test]
fn standalone_help_exposes_local_work_and_gateway_configuration() {
    let mut command = command(crate::MultitoolCli::command())
        .bin_name("airs-harness")
        .term_width(90)
        .disable_help_subcommand(true)
        .override_usage(
            "airs-harness [OPTIONS] [PROMPT]\n       airs-harness [OPTIONS] <COMMAND> [ARGS]",
        )
        .mut_arg("environment", |arg| arg.hide(false));
    // The test also runs in the upstream codex binary, where AIRS-only commands
    // are hidden by derive attributes. Make the presentation fixture explicit.
    for name in ["setup", "setup-mcp", "env", "status"] {
        command = command.mut_subcommand(name, |subcommand| subcommand.hide(false));
    }
    insta::assert_snapshot!("airs_harness_help", command.render_help().to_string());
}
