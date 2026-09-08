use super::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering;

#[derive(Debug)]
struct LimitedStore {
    values: Mutex<HashMap<String, String>>,
    writes_left: AtomicIsize,
    deletes_left: AtomicIsize,
    fail_after_write_account: Mutex<Option<String>>,
    corrupt_chunk_reads: AtomicBool,
}

impl LimitedStore {
    fn new() -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
            writes_left: AtomicIsize::new(-1),
            deletes_left: AtomicIsize::new(-1),
            fail_after_write_account: Mutex::new(None),
            corrupt_chunk_reads: AtomicBool::new(false),
        }
    }
}

impl KeyringStore for LimitedStore {
    fn load(&self, _: &str, account: &str) -> Result<Option<String>, CredentialStoreError> {
        if self.corrupt_chunk_reads.load(Ordering::SeqCst) && account.split('.').count() == 3 {
            return Ok(Some("00".into()));
        }
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
        let mut ambiguous = self.fail_after_write_account.lock().unwrap();
        if ambiguous.as_deref() == Some(account) {
            ambiguous.take();
            return Err(invalid());
        }
        Ok(())
    }
    fn delete(&self, _: &str, account: &str) -> Result<bool, CredentialStoreError> {
        let left = self.deletes_left.load(Ordering::SeqCst);
        if left == 0 {
            return Err(invalid());
        }
        if left > 0 {
            self.deletes_left.fetch_sub(1, Ordering::SeqCst);
        }
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
    for allowed_writes in [1, 5] {
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

#[test]
fn interrupted_deletion_disables_reads_and_retains_enumeration_until_retry_finishes() {
    for allowed_deletes in [0, 1, 3] {
        let store = ChunkedStore(LimitedStore::new());
        store
            .save("fixture", "account", &"t".repeat(CHUNK_BYTES * 3))
            .unwrap();
        let active = store.0.values.lock().unwrap()["account"].clone();
        let mut expected: serde_json::Value = serde_json::from_str(&active).unwrap();
        expected["version"] = 2.into();
        store
            .0
            .deletes_left
            .store(allowed_deletes, Ordering::SeqCst);
        assert!(store.delete("fixture", "account").is_err());
        assert_eq!(store.load("fixture", "account").unwrap(), None);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&store.0.values.lock().unwrap()["account"])
                .unwrap(),
            expected
        );
        store.0.deletes_left.store(-1, Ordering::SeqCst);
        assert!(store.delete("fixture", "account").unwrap());
        assert_eq!(*store.0.values.lock().unwrap(), HashMap::new());
    }
}

#[test]
fn failed_tombstone_write_preserves_the_active_credential_and_all_chunks() {
    let store = ChunkedStore(LimitedStore::new());
    let value = "active-token".repeat(100);
    store.save("fixture", "account", &value).unwrap();
    let before = store.0.values.lock().unwrap().clone();
    store.0.writes_left.store(0, Ordering::SeqCst);
    assert!(store.delete("fixture", "account").is_err());
    assert_eq!(*store.0.values.lock().unwrap(), before);
    assert_eq!(store.load("fixture", "account").unwrap(), Some(value));
}

#[test]
fn replacement_waits_for_pending_deletion_to_finish() {
    let store = ChunkedStore(LimitedStore::new());
    store.save("fixture", "account", "old-token").unwrap();
    store.0.deletes_left.store(0, Ordering::SeqCst);
    assert!(store.delete("fixture", "account").is_err());
    let pending = store.0.values.lock().unwrap().clone();
    assert!(store.save("fixture", "account", "new-token").is_err());
    assert_eq!(*store.0.values.lock().unwrap(), pending);
    assert_eq!(store.load("fixture", "account").unwrap(), None);
    store.0.deletes_left.store(-1, Ordering::SeqCst);
    store.save("fixture", "account", "new-token").unwrap();
    assert_eq!(
        store.load("fixture", "account").unwrap(),
        Some("new-token".into())
    );
    assert_eq!(store.0.values.lock().unwrap().len(), 2);
}

#[test]
fn corrupt_manifest_is_retained_when_deletion_cannot_enumerate_chunks() {
    let store = ChunkedStore(LimitedStore::new());
    store.save("fixture", "account", "fixture-token").unwrap();
    store
        .0
        .values
        .lock()
        .unwrap()
        .insert("account".into(), "corrupt-manifest".into());
    let before = store.0.values.lock().unwrap().clone();
    assert!(store.delete("fixture", "account").is_err());
    assert_eq!(*store.0.values.lock().unwrap(), before);
    assert!(store.load("fixture", "account").is_err());
}

#[test]
fn ambiguous_manifest_commit_preserves_both_generations_until_next_save() {
    let store = ChunkedStore(LimitedStore::new());
    store.save("fixture", "account", "previous-token").unwrap();
    *store.0.fail_after_write_account.lock().unwrap() = Some("account".into());
    assert!(store.save("fixture", "account", "committed-token").is_err());
    assert_eq!(
        store.load("fixture", "account").unwrap(),
        Some("committed-token".into())
    );
    let journal = store.0.values.lock().unwrap()["account.pending-generations"].clone();
    assert!(!journal.contains("previous-token"));
    assert!(!journal.contains("committed-token"));
    assert_eq!(store.0.values.lock().unwrap().len(), 4);
    store.save("fixture", "account", "final-token").unwrap();
    assert_eq!(
        store.load("fixture", "account").unwrap(),
        Some("final-token".into())
    );
    assert_eq!(store.0.values.lock().unwrap().len(), 2);
}

#[test]
fn failed_old_generation_cleanup_is_retried_by_delete() {
    let store = ChunkedStore(LimitedStore::new());
    store.save("fixture", "account", "previous-token").unwrap();
    store.0.deletes_left.store(0, Ordering::SeqCst);
    assert!(store.save("fixture", "account", "current-token").is_err());
    assert_eq!(
        store.load("fixture", "account").unwrap(),
        Some("current-token".into())
    );
    assert_eq!(store.0.values.lock().unwrap().len(), 4);
    store.0.deletes_left.store(-1, Ordering::SeqCst);
    assert!(store.delete("fixture", "account").unwrap());
    assert_eq!(*store.0.values.lock().unwrap(), HashMap::new());
}

#[test]
fn interrupted_chunk_write_blocks_new_generation_until_cleanup_succeeds() {
    let store = ChunkedStore(LimitedStore::new());
    store.save("fixture", "account", "previous-token").unwrap();
    store.0.writes_left.store(2, Ordering::SeqCst);
    assert!(
        store
            .save("fixture", "account", &"n".repeat(CHUNK_BYTES * 2))
            .is_err()
    );
    let pending = store.0.values.lock().unwrap().clone();
    assert_eq!(pending.len(), 4);
    store.0.writes_left.store(-1, Ordering::SeqCst);
    store.0.deletes_left.store(0, Ordering::SeqCst);
    assert!(store.save("fixture", "account", "next-token").is_err());
    assert_eq!(*store.0.values.lock().unwrap(), pending);
    assert_eq!(
        store.load("fixture", "account").unwrap(),
        Some("previous-token".into())
    );
    store.0.deletes_left.store(-1, Ordering::SeqCst);
    store.save("fixture", "account", "next-token").unwrap();
    assert_eq!(
        store.load("fixture", "account").unwrap(),
        Some("next-token".into())
    );
    assert_eq!(store.0.values.lock().unwrap().len(), 2);
}

#[test]
fn intervening_legacy_writer_current_generation_survives_recovery() {
    let store = ChunkedStore(LimitedStore::new());
    store.save("fixture", "account", "previous-token").unwrap();
    store.0.writes_left.store(2, Ordering::SeqCst);
    assert!(
        store
            .save("fixture", "account", &"n".repeat(CHUNK_BYTES * 2))
            .is_err()
    );
    store.0.writes_left.store(-1, Ordering::SeqCst);
    // Simulate a v1 writer that knows nothing about generation journals.
    let value = "legacy-writer-token";
    let current = Manifest {
        version: 1,
        generation: Uuid::new_v4(),
        chunks: 1,
        sha256: format!("{:x}", Sha256::digest(value.as_bytes())),
    };
    let chunk: String = value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    store
        .0
        .save("fixture", &current.account("account", 0), &chunk)
        .unwrap();
    let root = serde_json::to_string(&current).unwrap();
    store.0.save("fixture", "account", &root).unwrap();
    // Recovery happens before the next write. Fail that write to inspect just
    // recovery's result without allowing a new replacement to mask damage.
    store.0.writes_left.store(0, Ordering::SeqCst);
    assert!(store.save("fixture", "account", "next-token").is_err());
    assert_eq!(
        store.load("fixture", "account").unwrap(),
        Some(value.into())
    );
    assert_eq!(
        *store.0.values.lock().unwrap(),
        HashMap::from([
            ("account".into(), root),
            (current.account("account", 0), chunk),
        ])
    );
}

#[test]
fn new_chunk_readback_failure_keeps_previous_commit_and_cleanup_evidence() {
    let store = ChunkedStore(LimitedStore::new());
    store.save("fixture", "account", "previous-token").unwrap();
    let previous = store.0.values.lock().unwrap()["account"].clone();
    store.0.corrupt_chunk_reads.store(true, Ordering::SeqCst);
    assert!(store.save("fixture", "account", "new-token").is_err());
    assert_eq!(store.0.values.lock().unwrap()["account"], previous);
    assert_eq!(store.0.values.lock().unwrap().len(), 4);
    store.0.corrupt_chunk_reads.store(false, Ordering::SeqCst);
    assert_eq!(
        store.load("fixture", "account").unwrap(),
        Some("previous-token".into())
    );
    assert!(store.delete("fixture", "account").unwrap());
    assert_eq!(*store.0.values.lock().unwrap(), HashMap::new());
}

#[test]
fn corrupt_pending_journal_preserves_generation_evidence() {
    let store = ChunkedStore(LimitedStore::new());
    store.save("fixture", "account", "previous-token").unwrap();
    store.0.values.lock().unwrap().insert(
        "account.pending-generations".into(),
        "invalid-journal".into(),
    );
    let before = store.0.values.lock().unwrap().clone();
    assert!(store.save("fixture", "account", "next-token").is_err());
    assert_eq!(*store.0.values.lock().unwrap(), before);
    assert_eq!(
        store.load("fixture", "account").unwrap(),
        Some("previous-token".into())
    );
}

#[test]
fn ambiguous_journal_write_prevents_any_new_chunk_and_can_be_retried() {
    let store = ChunkedStore(LimitedStore::new());
    *store.0.fail_after_write_account.lock().unwrap() = Some("account.pending-generations".into());
    assert!(store.save("fixture", "account", "new-token").is_err());
    assert_eq!(store.load("fixture", "account").unwrap(), None);
    assert_eq!(store.0.values.lock().unwrap().len(), 1);
    store.save("fixture", "account", "retry-token").unwrap();
    assert_eq!(
        store.load("fixture", "account").unwrap(),
        Some("retry-token".into())
    );
    assert_eq!(store.0.values.lock().unwrap().len(), 2);
}

#[test]
fn corrupt_active_generation_blocks_retirement_but_explicit_delete_cleans_all() {
    let store = ChunkedStore(LimitedStore::new());
    store.save("fixture", "account", "previous-token").unwrap();
    *store.0.fail_after_write_account.lock().unwrap() = Some("account".into());
    assert!(store.save("fixture", "account", "current-token").is_err());
    let current = Manifest::parse(&store.0.values.lock().unwrap()["account"]).unwrap();
    store
        .0
        .values
        .lock()
        .unwrap()
        .insert(current.account("account", 0), "00".into());
    let before = store.0.values.lock().unwrap().clone();
    assert!(store.save("fixture", "account", "next-token").is_err());
    assert_eq!(*store.0.values.lock().unwrap(), before);
    assert!(store.load("fixture", "account").is_err());
    assert!(store.delete("fixture", "account").unwrap());
    assert_eq!(*store.0.values.lock().unwrap(), HashMap::new());
}
