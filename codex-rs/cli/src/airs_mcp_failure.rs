//! Classification at the private AIRS CLI/TUI boundary; raw error strings never cross it.
use codex_protocol::airs_mcp_failure::ConnectionState;
use codex_protocol::airs_mcp_failure::FailureCode;
use codex_protocol::airs_mcp_failure::McpFailure;

pub(crate) fn classify(error: &anyhow::Error) -> McpFailure {
    let context = error.downcast_ref::<McpFailure>().copied();
    let connection = context.map_or(ConnectionState::Unknown, |failure| failure.connection);
    if error
        .downcast_ref::<codex_rmcp_client::McpOAuthLoginTimeout>()
        .is_some()
    {
        return McpFailure::new(FailureCode::TimedOut, connection);
    }
    context.unwrap_or(McpFailure::new(FailureCode::Unknown, connection))
}

/// Attach a fallback only when a more specific typed cause has not been supplied.
pub(crate) fn with_fallback(error: anyhow::Error, fallback: McpFailure) -> anyhow::Error {
    if error.downcast_ref::<McpFailure>().is_some() {
        error
    } else {
        error.context(fallback)
    }
}

#[cfg(test)]
#[path = "airs_mcp_failure_tests.rs"]
mod tests;
