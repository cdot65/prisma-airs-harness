//! Present only the standalone product's supported entry points in CLI help.
use clap::Command;

fn text(value: &str) -> String {
    value
        .replace("OpenAI Codex", "Prisma AIRS Harness")
        .replace("Codex", "AIRS Harness")
        .replace("codex ", "airs ")
        .replace("`codex`", "`airs`")
        .replace("~/.codex", "~/.airs-harness")
        .replace("a AIRS Harness-provided sandbox", "the local sandbox")
}

pub fn command(command: Command) -> Command {
    command_with_name(command, "airs".to_string())
}

fn command_with_name(mut command: Command, name: String) -> Command {
    command = command.bin_name(name.clone());
    command = match name.as_str() {
        "airs" => command
            .override_usage("airs [OPTIONS] [PROMPT]\n       airs [OPTIONS] <COMMAND> [ARGS]"),
        "airs exec" => command.override_usage(
            "airs exec [OPTIONS] [PROMPT]\n       airs exec [OPTIONS] <COMMAND> [ARGS]",
        ),
        "airs mcp add" => {
            command.override_usage("airs mcp add [OPTIONS] <NAME> (--url <URL> | -- <COMMAND>...)")
        }
        _ => command,
    };
    if let Some(about) = command.get_about() {
        command = command.clone().about(text(&about.to_string()));
    }
    if let Some(about) = command.get_long_about() {
        command = command.clone().long_about(text(&about.to_string()));
    }
    if let Some(help) = command.get_before_help() {
        command = command.clone().before_help(text(&help.to_string()));
    }
    if let Some(help) = command.get_before_long_help() {
        command = command.clone().before_long_help(text(&help.to_string()));
    }
    if let Some(help) = command.get_after_help() {
        command = command.clone().after_help(text(&help.to_string()));
    }
    if let Some(help) = command.get_after_long_help() {
        command = command.clone().after_long_help(text(&help.to_string()));
    }
    command = command.mut_args(|mut arg| {
        if let Some(help) = arg.get_help() {
            arg = arg.clone().help(text(&help.to_string()));
        }
        if let Some(help) = arg.get_long_help() {
            arg = arg.clone().long_help(text(&help.to_string()));
        }
        if arg.get_long() == Some("config") {
            arg = arg.help("Override an allowed setting in the selected environment. Gateway, credential, capability and MCP bindings cannot be overridden.")
                .long_help("Override an allowed setting in the selected AIRS environment. Use TOML key=value syntax. Gateway, credential, capability and MCP bindings cannot be overridden. Select models with -m @provider/model or use the gateway default.");
        }
        if arg.get_long() == Some("profile") {
            arg = arg.help("Layer <name>.config.toml from the selected environment over its base configuration")
                .long_help("Layer <name>.config.toml from the selected environment over its base configuration. Destination and credential bindings remain fixed.");
        }
        if arg.get_long() == Some("verify-access") {
            arg = arg.hide(false);
        }
        if arg.get_long() == Some("with-api-key") {
            arg = arg.help("Enter a workspace API key securely and save it in the OS credential store")
                .long_help("Enter a workspace API key at a hidden terminal prompt and save it in the OS credential store. Automation may supply the key through stdin. The key is never saved in a plaintext configuration file.");
        }
        if matches!(arg.get_long(), Some("remote" | "remote-auth-token-env" | "oss" | "local-provider" | "search" | "api-key" | "with-access-token" | "issuer-base-url" | "client-id")) {
            arg = arg.hide(true);
        }
        arg
    });
    for subcommand in command.get_subcommands_mut() {
        *subcommand = command_with_name(
            subcommand.clone(),
            format!("{name} {}", subcommand.get_name()),
        );
        if matches!(
            subcommand.get_name(),
            "agents"
                | "app-server"
                | "remote-control"
                | "cloud"
                | "update"
                | "app"
                | "exec-server"
                | "mcp-server"
                | "plugin"
        ) {
            *subcommand = subcommand.clone().hide(true);
        }
    }
    command
}

#[cfg(test)]
#[path = "airs_help_tests.rs"]
mod tests;
