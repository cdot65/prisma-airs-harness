//! CLI presentation for the existing MCP OAuth flow and scope retry policy.
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use codex_config::types::AuthKeyringBackendKind;
use codex_config::types::OAuthCredentialsStoreMode;
use codex_exec_server::HttpClient;
use codex_mcp::ResolvedMcpOAuthScopes;
use codex_mcp::should_retry_without_scopes;
use codex_rmcp_client::McpOAuthClientRegistration;
use codex_rmcp_client::StreamableHttpRedirectMode;
use codex_rmcp_client::perform_oauth_login;
use codex_rmcp_client::perform_oauth_login_return_url;

#[derive(Clone, Copy)]
pub(super) enum BrowserMode {
    Open,
    Print,
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn perform_oauth_login_retry_without_scopes(
    name: &str,
    url: &str,
    store_mode: OAuthCredentialsStoreMode,
    keyring_backend_kind: AuthKeyringBackendKind,
    http_headers: Option<HashMap<String, String>>,
    env_http_headers: Option<HashMap<String, String>>,
    resolved_scopes: &ResolvedMcpOAuthScopes,
    oauth_client_id: Option<&str>,
    client_registration: McpOAuthClientRegistration,
    oauth_resource: Option<&str>,
    callback_port: Option<u16>,
    callback_url: Option<&str>,
    global_callback_url: Option<&str>,
    http_client: Arc<dyn HttpClient>,
    browser_mode: BrowserMode,
) -> Result<()> {
    let login = async |scopes: Vec<String>| -> Result<()> {
        match browser_mode {
            BrowserMode::Open => {
                perform_oauth_login(
                    name,
                    url,
                    store_mode,
                    keyring_backend_kind,
                    http_headers.clone(),
                    env_http_headers.clone(),
                    &scopes,
                    oauth_client_id,
                    client_registration,
                    oauth_resource,
                    callback_port,
                    callback_url,
                    global_callback_url,
                    Arc::clone(&http_client),
                )
                .await
            }
            BrowserMode::Print => {
                let handle = perform_oauth_login_return_url(
                    name,
                    url,
                    store_mode,
                    keyring_backend_kind,
                    http_headers.clone(),
                    env_http_headers.clone(),
                    &scopes,
                    oauth_client_id,
                    client_registration,
                    oauth_resource,
                    /*timeout_secs*/ None,
                    callback_port,
                    callback_url,
                    global_callback_url,
                    Arc::clone(&http_client),
                    StreamableHttpRedirectMode::Legacy,
                )
                .await?;
                eprintln!(
                    "Authorize `{name}` by opening this URL in your browser:\n{}\n",
                    handle.authorization_url()
                );
                handle.wait().await
            }
        }
    };
    match login(resolved_scopes.scopes.clone()).await {
        Ok(()) => Ok(()),
        Err(err) if should_retry_without_scopes(resolved_scopes, &err) => {
            println!("OAuth provider rejected discovered scopes. Retrying without scopes…");
            login(Vec::new()).await
        }
        Err(err) => Err(err),
    }
}
