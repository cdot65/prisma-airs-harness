//! Selects the memory-only Desktop host credential for a configured MCP server.
//!
//! A Desktop credential applies only to a plain local Streamable HTTP
//! registration whose name and URL match the bootstrap binding. Such a server
//! never reads stored OAuth credentials or configured authorization headers.
use codex_api::SharedAuthProvider;
use codex_config::McpServerAuth;
use codex_config::McpServerConfig;
use codex_config::McpServerTransportConfig;

pub(crate) fn desktop_auth_provider(
    server_name: &str,
    config: &McpServerConfig,
) -> Option<SharedAuthProvider> {
    match &config.transport {
        McpServerTransportConfig::StreamableHttp {
            url,
            bearer_token_env_var: None,
            http_headers,
            env_http_headers,
            http_headers_helper: None,
        } if matches!(config.auth, McpServerAuth::OAuth)
            && config.is_local_environment()
            && http_headers
                .as_ref()
                .is_none_or(std::collections::HashMap::is_empty)
            && env_http_headers
                .as_ref()
                .is_none_or(std::collections::HashMap::is_empty) =>
        {
            codex_model_provider::desktop_mcp_auth_provider(server_name, url)
        }
        McpServerTransportConfig::StreamableHttp { .. }
        | McpServerTransportConfig::Stdio { .. } => None,
    }
}
