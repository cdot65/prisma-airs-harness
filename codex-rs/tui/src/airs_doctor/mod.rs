//! Private AIRS diagnostics. Reports are transient UI state, never model context.
pub(crate) mod process;
pub(crate) mod report;
pub(crate) mod views;
use crate::airs_mcp_manager::Connection;
use codex_protocol::ThreadId;
use serde::Deserialize;

pub(crate) const VIEW_ID: &str = "airs_doctor";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Inspect,
    Verify,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Check {
    pub(crate) name: String,
    pub(crate) passed: bool,
    pub(crate) detail: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Report {
    pub(crate) schema_version: u32,
    pub(crate) product: String,
    pub(crate) authentication: String,
    pub(crate) checks: Vec<Check>,
}

pub(crate) enum Event {
    Open(Mode),
    Report {
        session: std::sync::Arc<report::Session>,
        action: report::Action,
    },
    Cancel(u64),
    SignIn,
    ConfirmVerify,
    Done {
        attempt: u64,
        thread: Option<ThreadId>,
        environment: String,
        report: Result<Report, String>,
        connections: Vec<Connection>,
    },
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AirsDoctorEvent [private diagnostics]")
    }
}

pub(crate) fn display(value: &str) -> String {
    value
        .chars()
        .filter(|c| {
            !c.is_control()
                && !matches!(c,
        '\u{061c}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' |
        '\u{2060}'..='\u{206f}' | '\u{feff}')
        })
        .take(2048)
        .collect()
}

#[cfg(test)]
#[path = "doctor_tests.rs"]
mod tests;
