use super::*;
use crate::storage::chunks::ChunkedStore;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Debug, Default)]
struct WindowsSizedStore(Arc<Mutex<HashMap<(String, String), String>>>);

impl KeyringStore for WindowsSizedStore {
    fn load(&self, service: &str, account: &str) -> Result<Option<String>, CredentialStoreError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .get(&(service.into(), account.into()))
            .cloned())
    }
    fn save(&self, service: &str, account: &str, value: &str) -> Result<(), CredentialStoreError> {
        if value.encode_utf16().count() * 2 > 2560 {
            return Err(CredentialStoreError::new(keyring::Error::TooLong(
                "fixture".into(),
                2560,
            )));
        }
        self.0
            .lock()
            .unwrap()
            .insert((service.into(), account.into()), value.into());
        Ok(())
    }
    fn delete(&self, service: &str, account: &str) -> Result<bool, CredentialStoreError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .remove(&(service.into(), account.into()))
            .is_some())
    }
}

#[test]
fn full_workspace_token_fits_windows_entries_and_does_not_replace_legacy_record() {
    let native = WindowsSizedStore::default();
    let stores = WorkspaceStores {
        legacy: native.clone(),
        chunked: ChunkedStore(native.clone()),
    };
    let account = Uuid::new_v4();
    let token = "A".repeat(16_384);
    assert!(
        native
            .save("fixture", &account.to_string(), &token)
            .is_err()
    );
    native
        .save("fixture", &account.to_string(), "legacy-token")
        .unwrap();
    stores.save_new("fixture", account, &token).unwrap();
    assert_eq!(
        stores
            .load("fixture", WorkspaceCredentialFormat::ChunkedV2, account)
            .unwrap(),
        Some(token)
    );
    assert_eq!(
        stores
            .load("fixture", WorkspaceCredentialFormat::LegacyRaw, account)
            .unwrap(),
        Some("legacy-token".into())
    );
    assert!(
        stores
            .delete("fixture", WorkspaceCredentialFormat::LegacyRaw, account)
            .unwrap()
    );
    assert_eq!(
        stores
            .load("fixture", WorkspaceCredentialFormat::ChunkedV2, account)
            .unwrap(),
        Some("A".repeat(16_384))
    );
    assert!(
        stores
            .delete("fixture", WorkspaceCredentialFormat::ChunkedV2, account)
            .unwrap()
    );
    assert_eq!(*native.0.lock().unwrap(), HashMap::new());
}

#[test]
fn missing_or_corrupt_v2_never_falls_back_to_legacy_and_read_never_migrates() {
    let native = WindowsSizedStore::default();
    let stores = WorkspaceStores {
        legacy: native.clone(),
        chunked: ChunkedStore(native.clone()),
    };
    let account = Uuid::new_v4();
    native
        .save("fixture", &account.to_string(), "legacy-token")
        .unwrap();
    let before = native.0.lock().unwrap().clone();
    assert_eq!(
        stores
            .load("fixture", WorkspaceCredentialFormat::LegacyRaw, account)
            .unwrap(),
        Some("legacy-token".into())
    );
    assert_eq!(
        stores
            .load("fixture", WorkspaceCredentialFormat::ChunkedV2, account)
            .unwrap(),
        None
    );
    assert_eq!(*native.0.lock().unwrap(), before);
    stores.save_new("fixture", account, "new-token").unwrap();
    native
        .save(
            "fixture.workspace-v2",
            &account.to_string(),
            "invalid-manifest",
        )
        .unwrap();
    assert!(
        stores
            .load("fixture", WorkspaceCredentialFormat::ChunkedV2, account)
            .is_err()
    );
    assert_eq!(
        stores
            .load("fixture", WorkspaceCredentialFormat::LegacyRaw, account)
            .unwrap(),
        Some("legacy-token".into())
    );
}

#[test]
fn invalid_new_token_never_changes_either_native_namespace() {
    let native = WindowsSizedStore::default();
    let stores = WorkspaceStores {
        legacy: native.clone(),
        chunked: ChunkedStore(native.clone()),
    };
    let account = Uuid::new_v4();
    stores
        .save_new("fixture", account, "previous-token")
        .unwrap();
    let before = native.0.lock().unwrap().clone();
    for invalid in [
        String::new(),
        "contains whitespace".into(),
        "non-ascii-🔑".into(),
        "A".repeat(16_385),
    ] {
        assert!(stores.save_new("fixture", account, &invalid).is_err());
        assert_eq!(*native.0.lock().unwrap(), before);
    }
}
