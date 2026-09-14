//! Bounded helper-to-runtime failure contract. Provider bodies and credentials never
//! become recovery instructions; only these explicit local helper markers are accepted.
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum CredentialRecovery {
    #[error(
        "Your work sign-in is no longer renewable. Use /signin to continue; your conversation is preserved."
    )]
    SignInRequired,
    #[error(
        "The last token renewal could not be confirmed. Use /signin to continue safely; the previous refresh token will not be retried."
    )]
    OutcomeUnknown,
    #[error(
        "The credential store is unavailable. Unlock your native credential store, then retry. Run airs-harness doctor if access remains unavailable."
    )]
    StoreUnavailable,
    #[error(
        "The sign-in service is temporarily unavailable. Retry when the connection recovers; your saved session has been preserved."
    )]
    TemporarilyUnavailable,
}

impl CredentialRecovery {
    pub fn marker(self) -> &'static str {
        match self {
            Self::SignInRequired => "AIRS_CREDENTIAL_STATUS:sign_in_required",
            Self::OutcomeUnknown => "AIRS_CREDENTIAL_STATUS:refresh_outcome_unknown",
            Self::StoreUnavailable => "AIRS_CREDENTIAL_STATUS:credential_store_unavailable",
            Self::TemporarilyUnavailable => "AIRS_CREDENTIAL_STATUS:temporarily_unavailable",
        }
    }

    pub(crate) fn from_stderr(stderr: &[u8]) -> Option<Self> {
        if stderr.len() > 16_384 {
            return None;
        }
        let text = std::str::from_utf8(stderr).ok()?;
        [
            Self::SignInRequired,
            Self::OutcomeUnknown,
            Self::StoreUnavailable,
            Self::TemporarilyUnavailable,
        ]
        .into_iter()
        .find(|reason| text.lines().any(|line| line == reason.marker()))
    }
}
