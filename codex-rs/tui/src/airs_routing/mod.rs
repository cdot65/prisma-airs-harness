//! Conversation routing is explicit state, never model-visible instructions.
pub(crate) mod views;
use codex_protocol::ThreadId;

pub(crate) const VIEW_ID: &str = "airs_routing";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Config,
    Model,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Selection {
    pub(crate) saved_config: Option<String>,
    pub(crate) model: Option<String>,
}

impl Selection {
    pub(crate) fn changed(&self, kind: Kind, value: Option<String>) -> Self {
        match kind {
            Kind::Config if self.saved_config != value => Self {
                saved_config: value,
                model: None,
            },
            Kind::Config => self.clone(),
            Kind::Model => Self {
                saved_config: self.saved_config.clone(),
                model: value,
            },
        }
    }
    pub(crate) fn describe(&self) -> String {
        format!(
            "Config: {} · Model: {}",
            self.saved_config.as_deref().unwrap_or("gateway default"),
            self.model.as_deref().unwrap_or("follow config routing")
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Proposal {
    pub(crate) thread: ThreadId,
    pub(crate) baseline: Selection,
    pub(crate) target: Selection,
    pub(crate) kind: Kind,
}

#[derive(Default)]
pub(crate) struct State {
    pub(crate) saved_config: Option<String>,
    pub(crate) pending: Option<(u64, Proposal)>,
    pub(crate) uncertain: bool,
}

pub(crate) enum Event {
    Open(Kind),
    Input(Kind),
    Propose(Kind, Option<String>),
    Verify(Proposal),
    Cancel(u64),
    Checked {
        attempt: u64,
        proposal: Proposal,
        result: Result<(), String>,
    },
    CommitFailed(u64),
    Applied(u64),
    Deadline(u64),
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AirsRoutingEvent [private selection]")
    }
}
