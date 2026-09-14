//! Stop model-driven fallback at the authentication boundary, after the actual tool
//! result is recorded. Recovery cannot infer identity from an opaque gateway token.
use crate::session::step_context::StepContext;
use codex_protocol::mcp::CallToolResult;

pub(crate) fn observe(step: &StepContext, server: &str, result: &anyhow::Result<CallToolResult>) {
    if step.turn.config.model_provider.gateway.is_none() {
        return;
    }
    let requires_auth = match result {
        Err(error) => codex_rmcp_client::is_authentication_required_error(error),
        Ok(result) => {
            result.is_error == Some(true)
                && result
                    .meta
                    .as_ref()
                    .and_then(|meta| meta.get("mcp/www_authenticate"))
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|challenges| !challenges.is_empty())
        }
    };
    if requires_auth && let Ok(mut failure) = step.mcp_authentication_failure.lock() {
        let server: String = server
            .chars()
            .filter(|character| !character.is_control())
            .take(128)
            .collect();
        let home = step.turn.config.codex_home.to_string_lossy();
        let home = shlex::try_quote(&home).unwrap_or_default();
        let name = shlex::try_quote(&server).unwrap_or_default();
        failure.get_or_insert_with(|| format!(
            "MCP sign-in required: {server}. This turn stopped before an alternate credential path was attempted. In another terminal, run AIRS_HARNESS_HOME={home} airs-harness mcp login {name}, then start a fresh conversation. The gateway does not provide verified account continuity for restoring this MCP conversation; completed tools will not be replayed."
        ));
    }
}
