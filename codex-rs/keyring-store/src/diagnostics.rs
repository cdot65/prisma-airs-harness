//! Bounded diagnostics that never render platform error payloads or credentials.
use super::CredentialStoreError;
use keyring::Error;

/// Stable categories for reporting native credential operations to users.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialStoreErrorKind {
    Missing,
    Unavailable,
    TooLong,
    Invalid,
    Corrupt,
    Ambiguous,
    PlatformFailure,
    Denied,
    Cancelled,
    InteractionRequired,
    SessionUnavailable,
    Unknown,
}

impl CredentialStoreErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Unavailable => "unavailable",
            Self::TooLong => "too-long",
            Self::Invalid => "invalid",
            Self::Corrupt => "corrupt",
            Self::Ambiguous => "ambiguous",
            Self::PlatformFailure => "platform-failure",
            Self::Denied => "denied",
            Self::Cancelled => "cancelled",
            Self::InteractionRequired => "interaction-required",
            Self::SessionUnavailable => "session-unavailable",
            Self::Unknown => "unknown",
        }
    }
}

/// Safe to display or log: contains no secret, account, attribute or backend text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CredentialStoreDiagnostic {
    pub kind: CredentialStoreErrorKind,
    pub native_code: Option<i32>,
}

impl CredentialStoreError {
    /// Classify a failure without formatting arbitrary backend error values.
    ///
    /// Access failures do not by themselves establish that a store is locked.
    /// Callers may refine recovery advice using a recognized native status code.
    pub fn diagnostic(&self) -> CredentialStoreDiagnostic {
        let Self::Other(error) = self;
        let kind = match error {
            Error::NoEntry => CredentialStoreErrorKind::Missing,
            Error::NoStorageAccess(_) => CredentialStoreErrorKind::Unavailable,
            Error::TooLong(_, _) => CredentialStoreErrorKind::TooLong,
            Error::Invalid(_, _) => CredentialStoreErrorKind::Invalid,
            Error::BadEncoding(_) => CredentialStoreErrorKind::Corrupt,
            Error::Ambiguous(_) => CredentialStoreErrorKind::Ambiguous,
            Error::PlatformFailure(_) => CredentialStoreErrorKind::PlatformFailure,
            _ => CredentialStoreErrorKind::Unknown,
        };
        match error {
            Error::NoStorageAccess(source) | Error::PlatformFailure(source) => {
                native_diagnostic(source.as_ref(), kind)
            }
            _ => CredentialStoreDiagnostic {
                kind,
                native_code: None,
            },
        }
    }
}

fn native_diagnostic(
    source: &(dyn std::error::Error + Send + Sync + 'static),
    kind: CredentialStoreErrorKind,
) -> CredentialStoreDiagnostic {
    #[cfg(target_os = "macos")]
    if let Some(error) = source.downcast_ref::<security_framework::base::Error>() {
        let code = error.code();
        // Apple's Security/base/SecBase.h. Interaction errors can have several
        // causes; neither code establishes that the keychain itself is locked.
        let kind = match code {
            -128 => CredentialStoreErrorKind::Cancelled, // errSecUserCanceled
            -61 | -25292 | -25293 => CredentialStoreErrorKind::Denied,
            -25308 | -25315 => CredentialStoreErrorKind::InteractionRequired,
            -25291 | -25294 | -25307 => CredentialStoreErrorKind::Unavailable,
            -25295 => CredentialStoreErrorKind::Corrupt,
            _ => kind,
        };
        return CredentialStoreDiagnostic {
            kind,
            native_code: Some(code),
        };
    }
    #[cfg(target_os = "windows")]
    if let Some(error) = source.downcast_ref::<keyring::windows::Error>() {
        // WinError.h: ERROR_ACCESS_DENIED, ERROR_CANCELLED,
        // ERROR_NO_SUCH_LOGON_SESSION. Unknown codes retain the library category.
        let kind = match error.0 {
            5 => CredentialStoreErrorKind::Denied,
            1223 => CredentialStoreErrorKind::Cancelled,
            1312 => CredentialStoreErrorKind::SessionUnavailable,
            _ => kind,
        };
        return CredentialStoreDiagnostic {
            kind,
            native_code: i32::try_from(error.0).ok(),
        };
    }
    // Unknown error types are deliberately not parsed or rendered.
    let _ = source;
    CredentialStoreDiagnostic {
        kind,
        native_code: None,
    }
}
