use codex_keyring_store::CredentialStoreDiagnostic;
use codex_keyring_store::CredentialStoreError;
use codex_keyring_store::CredentialStoreErrorKind;
use keyring::Error;
use pretty_assertions::assert_eq;

#[test]
fn diagnostic_classification_does_not_render_sensitive_error_payloads() {
    let secret = "must-never-appear-in-diagnostics";
    let cases = [
        (Error::NoEntry, CredentialStoreErrorKind::Missing),
        (
            Error::NoStorageAccess(Box::new(std::io::Error::other(secret))),
            CredentialStoreErrorKind::Unavailable,
        ),
        (
            Error::PlatformFailure(Box::new(std::io::Error::other(secret))),
            CredentialStoreErrorKind::PlatformFailure,
        ),
        (
            Error::TooLong(secret.into(), 128),
            CredentialStoreErrorKind::TooLong,
        ),
        (
            Error::Invalid(secret.into(), secret.into()),
            CredentialStoreErrorKind::Invalid,
        ),
        (
            Error::BadEncoding(secret.as_bytes().to_vec()),
            CredentialStoreErrorKind::Corrupt,
        ),
        (
            Error::Ambiguous(Vec::new()),
            CredentialStoreErrorKind::Ambiguous,
        ),
    ];
    for (error, kind) in cases {
        let diagnostic = CredentialStoreError::new(error).diagnostic();
        assert_eq!(
            diagnostic,
            CredentialStoreDiagnostic {
                kind,
                native_code: None
            }
        );
        assert!(!format!("{diagnostic:?}").contains(secret));
        assert!(!diagnostic.kind.as_str().contains(secret));
    }
}

#[test]
fn status_looking_backend_text_is_not_treated_as_native_evidence() {
    let error = Error::PlatformFailure(Box::new(std::io::Error::other("OSStatus -25308")));
    assert_eq!(
        CredentialStoreError::new(error).diagnostic(),
        CredentialStoreDiagnostic {
            kind: CredentialStoreErrorKind::PlatformFailure,
            native_code: None,
        }
    );
}

#[cfg(target_os = "macos")]
#[test]
fn captures_typed_security_framework_status() {
    for (code, kind) in [
        (-25308, CredentialStoreErrorKind::InteractionRequired),
        (-25315, CredentialStoreErrorKind::InteractionRequired),
        (-128, CredentialStoreErrorKind::Cancelled),
        (-25293, CredentialStoreErrorKind::Denied),
        (-25307, CredentialStoreErrorKind::Unavailable),
        (-25295, CredentialStoreErrorKind::Corrupt),
        (-123456, CredentialStoreErrorKind::PlatformFailure),
    ] {
        let error = Error::PlatformFailure(Box::new(security_framework::base::Error::from(code)));
        assert_eq!(
            CredentialStoreError::new(error).diagnostic(),
            CredentialStoreDiagnostic {
                kind,
                native_code: Some(code)
            }
        );
    }
}

#[cfg(target_os = "windows")]
#[test]
fn captures_typed_windows_status() {
    for (code, kind) in [
        (1312, CredentialStoreErrorKind::SessionUnavailable),
        (5, CredentialStoreErrorKind::Denied),
        (1223, CredentialStoreErrorKind::Cancelled),
        (123456, CredentialStoreErrorKind::PlatformFailure),
    ] {
        let error = Error::PlatformFailure(Box::new(keyring::windows::Error(code)));
        assert_eq!(
            CredentialStoreError::new(error).diagnostic(),
            CredentialStoreDiagnostic {
                kind,
                native_code: Some(code as i32)
            }
        );
    }
}
