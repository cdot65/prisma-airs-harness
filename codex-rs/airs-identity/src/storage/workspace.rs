//! Explicit workspace formats: legacy reads never silently migrate credentials.
use super::CredentialStore;
use codex_keyring_store::CredentialStoreError;
use codex_keyring_store::DefaultKeyringStore;
use codex_keyring_store::KeyringStore;
use uuid::Uuid;

/// The binding, rather than probing stored contents, selects the native format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceCredentialFormat {
    LegacyRaw,
    ChunkedV2,
}

/// Versioned workspace keys. Callers own binding installation and its journal.
#[derive(Debug, Default)]
pub struct WorkspaceCredentialStore;

impl WorkspaceCredentialStore {
    /// New credentials always use the chunked namespace and verified readback.
    pub fn save_new(
        &self,
        service: &str,
        account: Uuid,
        token: &str,
    ) -> Result<(), CredentialStoreError> {
        native().save_new(service, account, token)
    }

    pub fn load(
        &self,
        service: &str,
        format: WorkspaceCredentialFormat,
        account: Uuid,
    ) -> Result<Option<String>, CredentialStoreError> {
        native().load(service, format, account)
    }

    pub fn delete(
        &self,
        service: &str,
        format: WorkspaceCredentialFormat,
        account: Uuid,
    ) -> Result<bool, CredentialStoreError> {
        native().delete(service, format, account)
    }
}

struct WorkspaceStores<L, C> {
    legacy: L,
    chunked: C,
}

fn native() -> WorkspaceStores<DefaultKeyringStore, CredentialStore> {
    WorkspaceStores {
        legacy: DefaultKeyringStore,
        chunked: CredentialStore,
    }
}

impl<L: KeyringStore, C: KeyringStore> WorkspaceStores<L, C> {
    fn save_new(
        &self,
        service: &str,
        account: Uuid,
        token: &str,
    ) -> Result<(), CredentialStoreError> {
        if token.is_empty()
            || token.len() > 16_384
            || !token.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(CredentialStoreError::new(keyring::Error::Invalid(
                "workspace credential".into(),
                "expected an ASCII token of at most 16 KiB without whitespace".into(),
            )));
        }
        self.chunked.save(
            &format!("{service}.workspace-v2"),
            &account.to_string(),
            token,
        )
    }

    fn load(
        &self,
        service: &str,
        format: WorkspaceCredentialFormat,
        account: Uuid,
    ) -> Result<Option<String>, CredentialStoreError> {
        match format {
            WorkspaceCredentialFormat::LegacyRaw => self.legacy.load(service, &account.to_string()),
            WorkspaceCredentialFormat::ChunkedV2 => self
                .chunked
                .load(&format!("{service}.workspace-v2"), &account.to_string()),
        }
    }

    fn delete(
        &self,
        service: &str,
        format: WorkspaceCredentialFormat,
        account: Uuid,
    ) -> Result<bool, CredentialStoreError> {
        match format {
            WorkspaceCredentialFormat::LegacyRaw => {
                self.legacy.delete(service, &account.to_string())
            }
            WorkspaceCredentialFormat::ChunkedV2 => self
                .chunked
                .delete(&format!("{service}.workspace-v2"), &account.to_string()),
        }
    }
}

#[cfg(test)]
#[path = "workspace_tests.rs"]
mod tests;
