//! Keep AIRS destinations and credential mechanisms in the selected user home.
use codex_config::config_toml::ConfigToml;
use codex_config::types::McpServerConfig;
use codex_model_provider_info::ModelProviderInfo;
use std::collections::HashMap;
use std::path::Path;

pub(super) fn developer_instructions(
    existing: Option<String>,
    servers: &HashMap<String, McpServerConfig>,
) -> String {
    // Only names and tool allowlists enter context, never URLs or auth settings.
    // Keep this configuration summary deterministic and bounded independently of
    // how many MCP integrations an administrator installs.
    let mut configured = serde_json::Map::new();
    let mut names: Vec<_> = servers.keys().collect();
    names.sort();
    for name in names {
        let server = &servers[name];
        if !server.enabled {
            continue;
        }
        let entry = serde_json::json!({
            "required_at_startup": server.required,
            "tool_allowlist": server.enabled_tools.as_ref().map(|tools| {
                tools.iter().take(4).map(|tool| tool.chars().take(64).collect::<String>()).collect::<Vec<_>>()
            }),
        });
        configured.insert(name.chars().take(64).collect::<String>(), entry);
        if serde_json::Value::Object(configured.clone())
            .to_string()
            .len()
            > 2048
        {
            configured.remove(&name.chars().take(64).collect::<String>());
            break;
        }
        if configured.len() == 4 {
            break;
        }
    }
    let configured = serde_json::Value::Object(configured).to_string();
    let context = "<airs_terminal_runtime>\n\
You are Prisma AIRS Terminal, a standalone Rust terminal agent derived from the open-source Codex client. \
The terminal, filesystem access, shell tools, skills and session state run locally. \
Inference is remote through the configured Prisma AIRS AI Gateway; do not claim the model runs locally or that prompts never leave the machine. \
The gateway default is a routing choice, not a model identity. Do not guess the underlying model or claim it is fine-tuned for this product. \
An explicit @provider/model names the requested route; only verified gateway metadata establishes the actual backend. \
Verified application source facts: the repository is https://github.com/cdot65/prisma-airs-terminal, a Rust workspace under codex-rs/. Its cli/ package launches the application, tui/ implements the terminal UI, and core/ implements the agent runtime. The separate mcp-scanner/ directory implements the optional remote scanner backend. This source checkout is distinct from the user's current workspace and may not be installed locally. Base an architecture overview on these facts; do not invent directories, components or enabled capabilities. Inspect files before claiming details about the current checkout. If the application checkout has not been located, say so; do not claim filesystem access is impossible without checking.\n\
MCP means Model Context Protocol. MCP tools and MCP resources are separate capabilities. \
An empty list_mcp_resources result means no resources were advertised; it does not mean no servers or tools are connected. \
Use the callable tool definitions and actual tool results as evidence of available tools. A server can expose tools without resources. \
List the remote tools actually available in this session when asked; do not infer that all servers are disconnected from an empty resource list. \
Skills are local instruction documents, not executable tools, and their descriptions do not prove that a referenced integration is installed. \
Read a skill before applying it and disclose unavailable dependencies. Use apply_patch for focused source edits when available, and continue through verification and correction rather than stopping at a plan.\n\
</airs_terminal_runtime>";
    let context = format!(
        "{context}\nConfigured MCP servers and tool allowlists (bounded summary, not a live health result): {configured}\nWhen asked about MCP connections, name these configured servers and distinguish configuration, callable tools and advertised resources. Use /mcp for current connection status; never turn an empty resource list into a claim that these servers do not exist."
    );
    match existing {
        Some(existing) if !existing.is_empty() => format!("{existing}\n\n{context}"),
        _ => context,
    }
}

pub(super) fn validate(home: &Path, effective: &ConfigToml) -> std::io::Result<ConfigToml> {
    let input = std::fs::read_to_string(home.join("config.toml")).map_err(|_| {
        invalid("cannot read AIRS environment configuration; run airs-terminal setup")
    })?;
    let user: ConfigToml =
        toml::from_str(&input).map_err(|_| invalid("invalid AIRS user configuration"))?;
    if effective.model_providers != user.model_providers
        || effective.model_catalog_json != user.model_catalog_json
        || effective.model_context_window != user.model_context_window
        || effective.mcp_servers != user.mcp_servers
        || effective.shell_environment_policy != user.shell_environment_policy
        || effective.allow_login_shell != user.allow_login_shell
    {
        return Err(invalid(
            "AIRS gateway, credential, capability, MCP and credential-exclusion settings must come from the selected environment; repository/profile/CLI overrides are not permitted",
        ));
    }
    Ok(user)
}

pub(super) fn validate_provider(
    user: &ConfigToml,
    provider: &ModelProviderInfo,
) -> std::io::Result<()> {
    if user.model_providers.get("airs") != Some(provider) {
        return Err(invalid(
            "selected AIRS provider does not match this environment's binding",
        ));
    }
    Ok(())
}

fn invalid(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}

#[cfg(test)]
#[path = "airs_boundary_tests.rs"]
mod tests;
