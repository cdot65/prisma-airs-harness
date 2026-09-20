//! Optional judge settings stay outside conversation context and event logs.
pub(crate) mod input;
pub(crate) mod process;
pub(crate) mod views;

pub(crate) const VIEW_ID: &str = "airs_typesafe";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operation {
    Status,
    Set,
    Clear,
}

pub(crate) enum Event {
    Open,
    Set,
    ConfirmClear,
    Clear,
    Done {
        attempt: u64,
        thread: Option<codex_protocol::ThreadId>,
        home: std::path::PathBuf,
        result: Result<String, String>,
    },
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AirsTypeSafeEvent [private settings]")
    }
}
