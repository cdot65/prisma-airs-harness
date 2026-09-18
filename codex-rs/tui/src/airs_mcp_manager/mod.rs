//! AIRS-only MCP presentation and a private adapter to the existing MCP commands.
//! OAuth, credential storage and configuration edits remain owned by those commands.
mod authorization;
pub(crate) mod process;
pub(crate) mod views;

pub(crate) use authorization::AuthorizationView;
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
    Done {
        attempt: u64,
        thread: ThreadId,
        operation: Operation,
        result: Result<Vec<McpServerStatus>, String>,
    },
}

// Callback capability and provider authorization URLs must never enter debug logs.
impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AirsMcpManagerEvent [private interaction]")
    }
}
