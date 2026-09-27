//! Delivery certainty and ownership for clipboard operations.

use super::ClipboardLease;

/// A native clipboard write or an unacknowledged request to the user's terminal.
pub(crate) enum CopyOutcome {
    Copied(Option<ClipboardLease>),
    Requested,
}

impl CopyOutcome {
    /// Replace native ownership only when a backend supplies a new lease.
    pub(crate) fn store(self, lease: &mut Option<ClipboardLease>) -> CopyStatus {
        match self {
            Self::Copied(next_lease) => {
                if let Some(next_lease) = next_lease {
                    *lease = Some(next_lease);
                }
                CopyStatus::Confirmed
            }
            Self::Requested => CopyStatus::Unconfirmed,
        }
    }
}

/// Whether a clipboard backend confirmed the write or only sent a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CopyStatus {
    Confirmed,
    Unconfirmed,
}

impl CopyStatus {
    pub(crate) fn message(self, label: &str) -> String {
        match self {
            Self::Confirmed => format!("Copied {label} to clipboard"),
            Self::Unconfirmed => "Copy unconfirmed; /export saves chat".to_string(),
        }
    }
}
