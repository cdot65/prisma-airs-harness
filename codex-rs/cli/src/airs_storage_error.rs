//! User-facing native storage errors containing only bounded, nonsecret metadata.
use codex_keyring_store::CredentialStoreError;
use codex_keyring_store::CredentialStoreErrorKind;

pub(super) fn report(error: CredentialStoreError, operation: &'static str) -> anyhow::Error {
    let diagnostic = error.diagnostic();
    let backend = if cfg!(target_os = "macos") {
        "macOS Keychain"
    } else if cfg!(windows) {
        "Windows Credential Manager"
    } else if cfg!(target_os = "linux") {
        "Linux credential service"
    } else {
        "native credential store"
    };
    let recovery = match diagnostic.kind {
        CredentialStoreErrorKind::Cancelled => {
            "Authorization was cancelled. Retry login when ready."
        }
        CredentialStoreErrorKind::Denied => {
            "The operating system denied credential access. Check the application's access policy before retrying."
        }
        CredentialStoreErrorKind::InteractionRequired => {
            "The operating system requires user interaction that this session could not complete. Retry login from your signed-in desktop session."
        }
        CredentialStoreErrorKind::SessionUnavailable => {
            "This login session has no accessible credential store. Use your normal signed-in user session."
        }
        CredentialStoreErrorKind::Unavailable => {
            "The credential service could not be reached in this user session. Run airs-harness doctor for diagnostics."
        }
        CredentialStoreErrorKind::Missing => {
            "The saved credential is missing. Run airs-harness login to sign in again."
        }
        CredentialStoreErrorKind::TooLong => {
            "The credential exceeds this storage backend's supported size."
        }
        CredentialStoreErrorKind::Corrupt => {
            "The stored credential could not be decoded safely. Sign in again to replace it."
        }
        CredentialStoreErrorKind::Ambiguous => {
            "More than one credential matched. Run airs-harness doctor before changing saved credentials."
        }
        CredentialStoreErrorKind::Invalid => {
            "The credential store rejected the record format. Run airs-harness doctor for diagnostics."
        }
        CredentialStoreErrorKind::PlatformFailure | CredentialStoreErrorKind::Unknown => {
            "Run airs-harness doctor and include this error code in your support report."
        }
    };
    let category = diagnostic.kind.as_str();
    let native_code = diagnostic
        .native_code
        .map_or_else(|| "unavailable".to_owned(), |code| code.to_string());
    anyhow::anyhow!(
        "Credential storage failed during {operation}: {backend} (category: {category}; OS status: {native_code}). {recovery} No plaintext credential was written."
    )
    .context(codex_login::auth::CredentialRecovery::StoreUnavailable)
}
