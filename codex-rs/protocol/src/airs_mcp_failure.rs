//! Private AIRS helper failures. Only bounded codes and state cross the UI boundary.
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCode {
    InvalidEndpoint,
    DuplicateConnection,
    MissingConnection,
    Configuration,
    Discovery,
    Authorization,
    PermissionDenied,
    PolicyDenied,
    Transport,
    RemoteUnavailable,
    RateLimited,
    TimedOut,
    Cancelled,
    CredentialStore,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Unchanged,
    Saved,
    Unknown,
}

/// Safe failure context: never includes endpoint, provider body or credential material.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct McpFailure {
    pub code: FailureCode,
    pub connection: ConnectionState,
}

impl McpFailure {
    pub fn new(code: FailureCode, connection: ConnectionState) -> Self {
        Self { code, connection }
    }

    pub fn recovery(self) -> &'static str {
        match self.code {
            FailureCode::InvalidEndpoint => {
                "The MCP endpoint is invalid. Use the full HTTPS MCP URL supplied by AI Gateway, without credentials or query parameters."
            }
            FailureCode::DuplicateConnection => {
                "That connection name already exists. Select it in /mcp to sign in or reconnect, or choose a different name."
            }
            FailureCode::MissingConnection => {
                "The MCP connection no longer exists. Refresh /mcp before retrying."
            }
            FailureCode::Configuration => {
                "MCP configuration could not be read or saved. Run /doctor and check access to this environment's configuration before retrying."
            }
            FailureCode::Discovery => {
                "MCP authorization discovery failed. Confirm the gateway-facing MCP URL and its OAuth metadata, then retry from /mcp."
            }
            FailureCode::Authorization => {
                "MCP authorization did not complete. Retry Sign in from /mcp; if it fails again, check the gateway OAuth client, scopes and redirect configuration."
            }
            FailureCode::PermissionDenied => {
                "The gateway or authorization server denied this MCP request (HTTP 403). Check this identity's workspace and MCP permissions before retrying."
            }
            FailureCode::PolicyDenied => {
                "A gateway policy blocked this MCP request (HTTP 446). Check the gateway trace and policy configuration before retrying."
            }
            FailureCode::Transport => {
                "The MCP authorization endpoint could not be reached. Check gateway and issuer DNS, TLS and network access, then retry from /mcp."
            }
            FailureCode::RemoteUnavailable => {
                "The gateway or authorization service is unavailable. Retry from /mcp after the service recovers."
            }
            FailureCode::RateLimited => {
                "The gateway or authorization service rate-limited this request. Wait before retrying from /mcp."
            }
            FailureCode::TimedOut => {
                "The MCP operation timed out. Retry from /mcp; if sign-in expired, select Sign in for a fresh authorization link."
            }
            FailureCode::Cancelled => {
                "MCP sign-in was cancelled. Select Sign in from /mcp when ready."
            }
            FailureCode::CredentialStore => {
                "The MCP credential operation failed. Unlock or restore access to the native credential store, run /doctor, then retry Sign in or Sign out from /mcp."
            }
            FailureCode::Unknown => {
                "MCP operation did not complete and no specific cause was reported. Run /doctor and refresh /mcp before retrying."
            }
        }
    }
}

impl std::fmt::Display for McpFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.recovery(),
            match self.connection {
                ConnectionState::Saved =>
                    "The connection was saved; it may still need sign-in. Do not add it again.",
                ConnectionState::Unchanged => "The connection configuration was not changed.",
                ConnectionState::Unknown =>
                    "Refresh /mcp to inspect whether a connection was saved.",
            }
        )
    }
}

impl std::error::Error for McpFailure {}
