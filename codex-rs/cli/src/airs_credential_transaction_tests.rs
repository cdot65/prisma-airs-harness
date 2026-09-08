use super::*;
use pretty_assertions::assert_eq;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;

#[test]
fn malformed_binding_does_not_leak_values_in_recovery_or_persistence_errors() {
    let home = tempfile::tempdir().unwrap();
    let desired = binding(Source::Keyring);
    let store = FakeStore::default();
    let mut malformed = serde_json::to_value(&desired).unwrap();
    malformed["source"] = serde_json::json!({"kind":"PRIVATE-PARSE-CANARY"});
    std::fs::write(
        home.path().join("credential-binding.json"),
        serde_json::to_vec(&malformed).unwrap(),
    )
    .unwrap();
    let error = persist(home.path(), &desired, "fixture-key", &store, || Ok(())).unwrap_err();
    assert!(!format!("{error:#}").contains("PRIVATE-PARSE-CANARY"));
    std::fs::write(
        home.path().join(JOURNAL),
        serde_json::to_vec(&Pending {
            schema_version: 1,
            store: StoreKind::WorkspaceKeyringV1,
            account: desired.id,
        })
        .unwrap(),
    )
    .unwrap();
    let error = recover(home.path(), &store).unwrap_err();
    assert!(!format!("{error:#}").contains("PRIVATE-PARSE-CANARY"));
    assert!(store.values.borrow().is_empty());
}

#[derive(Default)]
struct FakeStore {
    values: RefCell<BTreeMap<Uuid, String>>,
    fail_save: Cell<bool>,
    fail_read: Cell<bool>,
    wrong_read: Cell<bool>,
    fail_delete: Cell<bool>,
}

impl Store for FakeStore {
    fn save(&self, account: Uuid, token: &str) -> anyhow::Result<()> {
        self.values.borrow_mut().insert(account, token.into());
        anyhow::ensure!(!self.fail_save.get(), "Injected ambiguous write failure");
        Ok(())
    }
    fn load(&self, account: Uuid) -> anyhow::Result<Option<String>> {
        anyhow::ensure!(!self.fail_read.get(), "Injected read failure");
        if self.wrong_read.get() {
            return Ok(Some("wrong-value".into()));
        }
        Ok(self.values.borrow().get(&account).cloned())
    }
    fn delete(&self, account: Uuid) -> anyhow::Result<()> {
        anyhow::ensure!(!self.fail_delete.get(), "Injected delete failure");
        self.values.borrow_mut().remove(&account);
        Ok(())
    }
}

fn binding(source: Source) -> Binding {
    Binding {
        schema_version: 1,
        id: Uuid::new_v4(),
        gateway_url: "https://gateway.example/v1".into(),
        credential_fingerprint: "test-identity".into(),
        source: Some(source),
    }
}

#[test]
fn failed_writes_reads_and_mismatches_clean_new_accounts_before_install() {
    for failure in ["save", "read", "mismatch"] {
        let home = tempfile::tempdir().unwrap();
        let binding = binding(Source::Keyring);
        let store = FakeStore::default();
        match failure {
            "save" => store.fail_save.set(true),
            "read" => store.fail_read.set(true),
            "mismatch" => store.wrong_read.set(true),
            _ => unreachable!(),
        }
        assert!(
            persist(home.path(), &binding, "fixture-key", &store, || panic!(
                "must not install"
            ))
            .is_err()
        );
        assert_eq!(*store.values.borrow(), BTreeMap::new());
        assert!(!home.path().join(JOURNAL).exists());
        assert!(!home.path().join("credential-binding.json").exists());
    }
}

#[test]
fn failed_cleanup_retains_only_account_metadata_and_can_retry_in_another_call() {
    let home = tempfile::tempdir().unwrap();
    let binding = binding(Source::Keyring);
    let store = FakeStore::default();
    store.fail_read.set(true);
    store.fail_delete.set(true);
    assert!(
        persist(
            home.path(),
            &binding,
            "private-fixture-key",
            &store,
            || panic!("must not install")
        )
        .is_err()
    );
    let journal = std::fs::read_to_string(home.path().join(JOURNAL)).unwrap();
    assert!(!journal.contains("private-fixture-key"));
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&journal).unwrap(),
        serde_json::json!({"schema_version": 1, "store": "workspace-keyring-v1", "account": binding.id})
    );
    store.fail_delete.set(false);
    recover(home.path(), &store).unwrap();
    assert_eq!(*store.values.borrow(), BTreeMap::new());
    assert!(!home.path().join(JOURNAL).exists());
}

#[test]
fn partial_install_restores_prior_binding_and_deletes_only_the_uncommitted_key() {
    let home = tempfile::tempdir().unwrap();
    let mut desired = binding(Source::Environment {
        variable: "FIXTURE_KEY".into(),
    });
    let previous = serde_json::to_vec(&desired).unwrap();
    std::fs::write(home.path().join("credential-binding.json"), &previous).unwrap();
    let previous_config = b"old-working-configuration";
    std::fs::write(home.path().join("config.toml"), previous_config).unwrap();
    desired.source = Some(Source::Keyring);
    let store = FakeStore::default();
    let unrelated = Uuid::new_v4();
    store
        .values
        .borrow_mut()
        .insert(unrelated, "unrelated-key".into());
    assert!(
        persist(home.path(), &desired, "fixture-key", &store, || {
            std::fs::write(
                home.path().join("credential-binding.json"),
                serde_json::to_vec(&desired)?,
            )?;
            std::fs::write(home.path().join("config.toml"), "partial-new-configuration")?;
            anyhow::bail!("Injected config installation failure")
        })
        .is_err()
    );
    assert_eq!(
        std::fs::read(home.path().join("credential-binding.json")).unwrap(),
        previous
    );
    assert_eq!(
        std::fs::read(home.path().join("config.toml")).unwrap(),
        previous_config
    );
    assert_eq!(
        *store.values.borrow(),
        BTreeMap::from([(unrelated, "unrelated-key".into())])
    );
    assert!(!home.path().join(JOURNAL).exists());
}

#[test]
fn failed_verification_never_deletes_an_existing_working_keyring_binding() {
    let home = tempfile::tempdir().unwrap();
    let binding = binding(Source::Keyring);
    let previous = serde_json::to_vec(&binding).unwrap();
    std::fs::write(home.path().join("credential-binding.json"), &previous).unwrap();
    let store = FakeStore::default();
    store
        .values
        .borrow_mut()
        .insert(binding.id, "fixture-key".into());
    store.fail_read.set(true);
    assert!(
        persist(home.path(), &binding, "fixture-key", &store, || panic!(
            "must not install"
        ))
        .is_err()
    );
    assert_eq!(
        *store.values.borrow(),
        BTreeMap::from([(binding.id, "fixture-key".into())])
    );
    assert_eq!(
        std::fs::read(home.path().join("credential-binding.json")).unwrap(),
        previous
    );
    assert!(!home.path().join(JOURNAL).exists());
}

#[test]
fn fresh_partial_install_restores_unbound_configuration_and_removes_the_new_key() {
    let home = tempfile::tempdir().unwrap();
    let binding = binding(Source::Keyring);
    let store = FakeStore::default();
    let previous_config = b"working-environment-credential-reference";
    std::fs::write(home.path().join("config.toml"), previous_config).unwrap();
    assert!(
        persist(home.path(), &binding, "fixture-key", &store, || {
            std::fs::write(
                home.path().join("credential-binding.json"),
                serde_json::to_vec(&binding)?,
            )?;
            std::fs::write(home.path().join("config.toml"), "uncommitted-configuration")?;
            anyhow::bail!("Injected partial install failure")
        })
        .is_err()
    );
    assert_eq!(
        std::fs::read(home.path().join("config.toml")).unwrap(),
        previous_config
    );
    assert!(!home.path().join("credential-binding.json").exists());
    assert!(!home.path().join(JOURNAL).exists());
    assert_eq!(*store.values.borrow(), BTreeMap::new());
}

#[test]
fn interrupted_install_with_bound_account_remains_recoverable() {
    let home = tempfile::tempdir().unwrap();
    let binding = binding(Source::Keyring);
    let store = FakeStore::default();
    store
        .values
        .borrow_mut()
        .insert(binding.id, "fixture-key".into());
    std::fs::write(
        home.path().join(JOURNAL),
        serde_json::to_vec(&Pending {
            schema_version: 1,
            store: StoreKind::WorkspaceKeyringV1,
            account: binding.id,
        })
        .unwrap(),
    )
    .unwrap();
    std::fs::write(
        home.path().join("credential-binding.json"),
        serde_json::to_vec(&binding).unwrap(),
    )
    .unwrap();
    let installed = Cell::new(false);
    persist(home.path(), &binding, "fixture-key", &store, || {
        installed.set(true);
        Ok(())
    })
    .unwrap();
    assert!(installed.get());
    assert_eq!(
        *store.values.borrow(),
        BTreeMap::from([(binding.id, "fixture-key".into())])
    );
    assert!(!home.path().join(JOURNAL).exists());
}

#[test]
fn oversized_cleanup_journal_cannot_trigger_deletion() {
    let home = tempfile::tempdir().unwrap();
    let store = FakeStore::default();
    let account = Uuid::new_v4();
    store
        .values
        .borrow_mut()
        .insert(account, "existing-key".into());
    std::fs::write(home.path().join(JOURNAL), vec![b'x'; 1025]).unwrap();
    assert!(recover(home.path(), &store).is_err());
    assert_eq!(
        *store.values.borrow(),
        BTreeMap::from([(account, "existing-key".into())])
    );
}

#[cfg(unix)]
#[test]
fn cleanup_rejects_symlinks_without_touching_the_referenced_account() {
    let home = tempfile::tempdir().unwrap();
    let store = FakeStore::default();
    let account = Uuid::new_v4();
    store
        .values
        .borrow_mut()
        .insert(account, "existing-key".into());
    let target = home.path().join("another-journal");
    let bytes = serde_json::to_vec(&Pending {
        schema_version: 1,
        store: StoreKind::WorkspaceKeyringV1,
        account,
    })
    .unwrap();
    std::fs::write(&target, &bytes).unwrap();
    std::os::unix::fs::symlink(&target, home.path().join(JOURNAL)).unwrap();
    assert!(recover(home.path(), &store).is_err());
    assert_eq!(std::fs::read(target).unwrap(), bytes);
    assert_eq!(
        *store.values.borrow(),
        BTreeMap::from([(account, "existing-key".into())])
    );
}
