use super::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering;

#[derive(Debug)]
struct LimitedStore {
    values: Mutex<HashMap<String, String>>,
    writes_left: AtomicIsize,
}

impl LimitedStore {
    fn new() -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
            writes_left: AtomicIsize::new(-1),
        }
    }
}

impl KeyringStore for LimitedStore {
    fn load(&self, _: &str, account: &str) -> Result<Option<String>, CredentialStoreError> {
        Ok(self.values.lock().unwrap().get(account).cloned())
    }
    fn save(&self, _: &str, account: &str, value: &str) -> Result<(), CredentialStoreError> {
        assert!(
            value.encode_utf16().count() * 2 <= 2560,
            "entry exceeds Windows credential capacity"
        );
        let left = self.writes_left.load(Ordering::SeqCst);
        if left == 0 {
            return Err(invalid());
        }
        if left > 0 {
            self.writes_left.fetch_sub(1, Ordering::SeqCst);
        }
        self.values
            .lock()
            .unwrap()
            .insert(account.into(), value.into());
        Ok(())
    }
    fn delete(&self, _: &str, account: &str) -> Result<bool, CredentialStoreError> {
        Ok(self.values.lock().unwrap().remove(account).is_some())
    }
}

#[test]
fn large_unicode_bundle_fits_windows_entries_and_round_trips() {
    let store = ChunkedStore(LimitedStore::new());
    let value = "token bundle · teammate 🔑".repeat(2000);
    store.save("fixture", "account", &value).unwrap();
    assert_eq!(store.load("fixture", "account").unwrap(), Some(value));
    store.save("fixture", "account", "refresh-pending").unwrap();
    assert_eq!(
        store.load("fixture", "account").unwrap(),
        Some("refresh-pending".into())
    );
    assert_eq!(store.0.values.lock().unwrap().len(), 2);
    assert!(store.delete("fixture", "account").unwrap());
    assert_eq!(store.0.values.lock().unwrap().len(), 0);
}

#[test]
fn interrupted_chunk_or_manifest_write_never_exposes_partial_replacement() {
    for allowed_writes in [1, 4] {
        let store = ChunkedStore(LimitedStore::new());
        store.save("fixture", "account", "previous-state").unwrap();
        store.0.writes_left.store(allowed_writes, Ordering::SeqCst);
        assert!(
            store
                .save("fixture", "account", &"new-state".repeat(200))
                .is_err()
        );
        assert_eq!(
            store.load("fixture", "account").unwrap(),
            Some("previous-state".into())
        );
    }
}

#[test]
fn missing_or_changed_chunk_fails_closed() {
    for corrupt in [false, true] {
        let store = ChunkedStore(LimitedStore::new());
        store.save("fixture", "account", "test-token").unwrap();
        let mut values = store.0.values.lock().unwrap();
        let chunk = values.keys().find(|key| *key != "account").unwrap().clone();
        if corrupt {
            values.insert(chunk, "0000".into());
        } else {
            values.remove(&chunk);
        }
        drop(values);
        assert!(store.load("fixture", "account").is_err());
    }
}
