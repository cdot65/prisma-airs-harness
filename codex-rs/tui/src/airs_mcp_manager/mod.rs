//! AIRS-only MCP presentation and a private adapter to the existing MCP commands.
//! OAuth, credential storage and configuration edits remain owned by those commands.
mod authorization;
pub(crate) mod process;
pub(crate) mod views;

pub(crate) use authorization::AuthorizationView;
pub(crate) use authorization::BrowserMode;
use codex_app_server_protocol::McpServerStatus;
use codex_protocol::ThreadId;
use tokio::sync::oneshot;

pub(crate) const VIEW_ID: &str = "airs_mcp_manager";

#[derive(Clone, Debug)]
pub(crate) struct Connection {
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) status: String,
    pub(crate) can_login: bool,
}

#[derive(Clone, Debug)]
pub(crate) enum Operation {
    Add { name: String, url: String },
    Login(String),
    Verify(String),
    Logout(String),
    Remove(String),
}

impl Operation {
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Add { name, .. }
            | Self::Login(name)
            | Self::Verify(name)
            | Self::Logout(name)
            | Self::Remove(name) => name,
        }
    }
}

pub(crate) enum Event {
    Open,
    Select(Connection),
    AddName,
    AddUrl(String),
    Confirm(Operation),
    Run(Operation),
    Cancel,
    Overview {
        attempt: u64,
        thread: ThreadId,
        connections: Vec<Connection>,
    },
    Authorize {
        attempt: u64,
        thread: ThreadId,
        url: String,
        callback: oneshot::Sender<String>,
    },
    Progress {
        attempt: u64,
        thread: ThreadId,
        progress: Progress,
    },
    Done {
        attempt: u64,
        thread: ThreadId,
        operation: Operation,
        result: Result<Vec<McpServerStatus>, String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Progress {
    ExchangingCode,
    SavingCredential,
    DiscoveringTools,
}

// Callback capability and provider authorization URLs must never enter debug logs.
impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AirsMcpManagerEvent [private interaction]")
    }
}

pub(crate) fn connections(
    config: &crate::legacy_core::config::Config,
    statuses: &[McpServerStatus],
) -> Vec<Connection> {
    use codex_app_server_protocol::McpServerConnectionStatus as State;
    use codex_config::types::McpServerTransportConfig;
    let mut connections: Vec<_> = config
        .mcp_servers
        .get()
        .iter()
        .map(|(name, server)| {
            let status = statuses.iter().find(|status| status.name == *name);
            let state = if !server.enabled {
                "Disabled".into()
            } else if let Some(status) = status {
                match status.runtime_status {
                    Some(State::Disabled) => "Disabled".into(),
                    Some(State::NotStarted) => "Not started · select to reconnect".into(),
                    Some(State::Starting) => "Connecting".into(),
                    Some(State::AuthenticationRequired) => "Sign-in required".into(),
                    Some(State::Failed | State::Cancelled) => {
                        "Not connected · select to retry".into()
                    }
                    Some(State::Connected) if status.tools_error.is_none() => {
                        format!("Connected · {} tools", status.tools.len())
                    }
                    Some(State::Connected) => "Connected · tool discovery failed".into(),
                    None if status.server_info.is_some() && status.tools_error.is_none() => {
                        format!(
                            "{} tools discovered · runtime unchecked",
                            status.tools.len()
                        )
                    }
                    None => "Not verified · select to reconnect".into(),
                }
            } else {
                "Not verified · select to reconnect".into()
            };
            let auth = status
                .map(|s| match s.auth_status {
                    codex_app_server_protocol::McpAuthStatus::OAuth => "OAuth saved",
                    codex_app_server_protocol::McpAuthStatus::NotLoggedIn => "Sign-in required",
                    codex_app_server_protocol::McpAuthStatus::BearerToken => "Bearer credential",
                    codex_app_server_protocol::McpAuthStatus::CredentialHelper => {
                        "Credential helper"
                    }
                    codex_app_server_protocol::McpAuthStatus::Unsupported => "OAuth unavailable",
                    codex_app_server_protocol::McpAuthStatus::Unknown => "Authentication unknown",
                })
                .unwrap_or("Authentication unchecked");
            Connection {
                name: name.clone(),
                status: format!("{state} · {auth}"),
                can_login: crate::chatwidget::airs_mcp_recovery::supports_oauth(server),
                url: match &server.transport {
                    McpServerTransportConfig::StreamableHttp { url, .. } => url.clone(),
                    McpServerTransportConfig::Stdio { .. } => "Local MCP transport".into(),
                },
            }
        })
        .collect();
    connections.sort_by(|a, b| a.name.cmp(&b.name));

    connections
}
