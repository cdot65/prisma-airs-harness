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
    values: RefCell<BTreeMap<(StoreKind, Uuid), String>>,
    fail_save: Cell<bool>,
    fail_read: Cell<bool>,
    wrong_read: Cell<bool>,
    fail_delete: Cell<bool>,
    read_recovery: Cell<Option<CredentialRecovery>>,
    delete_recovery: Cell<Option<CredentialRecovery>>,
}

impl Store for FakeStore {
    fn save(&self, kind: StoreKind, account: Uuid, token: &str) -> anyhow::Result<()> {
        self.values
            .borrow_mut()
            .insert((kind, account), token.into());
        anyhow::ensure!(!self.fail_save.get(), "Injected ambiguous write failure");
        Ok(())
    }
    fn load(&self, kind: StoreKind, account: Uuid) -> anyhow::Result<Option<String>> {
        if let Some(recovery) = self.read_recovery.get() {
            return Err(anyhow::anyhow!("Injected read failure").context(recovery));
        }
        anyhow::ensure!(!self.fail_read.get(), "Injected read failure");
        if self.wrong_read.get() {
            return Ok(Some("wrong-value".into()));
        }
        Ok(self.values.borrow().get(&(kind, account)).cloned())
    }
    fn delete(&self, kind: StoreKind, account: Uuid) -> anyhow::Result<()> {
        if let Some(recovery) = self.delete_recovery.get() {
            return Err(anyhow::anyhow!("Injected delete failure").context(recovery));
        }
        anyhow::ensure!(!self.fail_delete.get(), "Injected delete failure");
        self.values.borrow_mut().remove(&(kind, account));
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
fn failed_cleanup_preserves_initial_read_failure_and_previous_binding() {
    let home = tempfile::tempdir().unwrap();
    let mut desired = binding(Source::Environment {
        variable: "FIXTURE_KEY".into(),
    });
    let previous = serde_json::to_vec(&desired).unwrap();
    let previous_config = b"previous-working-configuration";
    std::fs::write(home.path().join("credential-binding.json"), &previous).unwrap();
    std::fs::write(home.path().join("config.toml"), previous_config).unwrap();
    desired.source = Some(Source::KeyringV2);
    let store = FakeStore::default();
    store.fail_read.set(true);
    store.fail_delete.set(true);

    let error = persist(home.path(), &desired, "private-fixture-key", &store, || {
        panic!("failed verification must not install a new binding")
    })
    .unwrap_err();

    assert_eq!(
        std::fs::read(home.path().join("credential-binding.json")).unwrap(),
        previous
    );
    assert_eq!(
        std::fs::read(home.path().join("config.toml")).unwrap(),
        previous_config
    );
    let journal = std::fs::read_to_string(home.path().join(JOURNAL)).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&journal).unwrap(),
        serde_json::json!({
            "schema_version": 1,
            "store": "workspace-keyring-v2",
            "account": desired.id,
        })
    );
    let diagnostic = format!("{error:#}");
    assert!(!diagnostic.contains("private-fixture-key"));
    assert!(diagnostic.contains("Credential cleanup is pending"));
    assert!(diagnostic.contains("Injected delete failure"));
    assert!(
        diagnostic.contains("Injected read failure"),
        "cleanup must retain the initiating verification failure: {diagnostic}"
    );
}

#[test]
fn combined_failures_preserve_typed_recovery_and_retry_only_the_pending_account() {
    for (primary, cleanup, expected) in [
        (
            Some(CredentialRecovery::OutcomeUnknown),
            None,
            CredentialRecovery::OutcomeUnknown,
        ),
        (
            None,
            Some(CredentialRecovery::StoreUnavailable),
            CredentialRecovery::StoreUnavailable,
        ),
        (
            Some(CredentialRecovery::OutcomeUnknown),
            Some(CredentialRecovery::StoreUnavailable),
            CredentialRecovery::OutcomeUnknown,
        ),
    ] {
        let home = tempfile::tempdir().unwrap();
        let desired = binding(Source::KeyringV2);
        let store = FakeStore::default();
        let unrelated = (StoreKind::WorkspaceKeyringV2, Uuid::new_v4());
        store
            .values
            .borrow_mut()
            .insert(unrelated, "unrelated-key".into());
        store.fail_read.set(true);
        store.fail_delete.set(true);
        store.read_recovery.set(primary);
        store.delete_recovery.set(cleanup);

        let error = persist(
            home.path(),
            &desired,
            "PRIVATE-TOKEN-CANARY",
            &store,
            || panic!("failed verification must not install a new binding"),
        )
        .unwrap_err();
        assert_eq!(error.downcast_ref::<CredentialRecovery>(), Some(&expected));
        let diagnostic = format!("{error:#}");
        assert!(diagnostic.contains("Injected read failure"));
        assert!(diagnostic.contains("Injected delete failure"));
        assert!(!diagnostic.contains("PRIVATE-TOKEN-CANARY"));
        let journal = std::fs::read_to_string(home.path().join(JOURNAL)).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&journal).unwrap(),
            serde_json::json!({"schema_version": 1, "store": "workspace-keyring-v2", "account": desired.id})
        );

        store.fail_delete.set(false);
        store.delete_recovery.set(None);
        recover(home.path(), &store).unwrap();
        assert_eq!(
            *store.values.borrow(),
            BTreeMap::from([(unrelated, "unrelated-key".into())])
        );
        assert!(!home.path().join(JOURNAL).exists());
    }
}

#[test]
fn isolated_primary_and_cleanup_failures_keep_their_typed_recovery() {
    let home = tempfile::tempdir().unwrap();
    let desired = binding(Source::KeyringV2);
    let store = FakeStore::default();
    store
        .read_recovery
        .set(Some(CredentialRecovery::OutcomeUnknown));
    let primary = persist(home.path(), &desired, "fixture-key", &store, || {
        panic!("failed verification must not install a new binding")
    })
    .unwrap_err();
    assert_eq!(
        primary.downcast_ref::<CredentialRecovery>(),
        Some(&CredentialRecovery::OutcomeUnknown)
    );
    assert!(!home.path().join(JOURNAL).exists());
    assert!(store.values.borrow().is_empty());

    store.read_recovery.set(None);
    store
        .delete_recovery
        .set(Some(CredentialRecovery::StoreUnavailable));
    let cleanup = persist(home.path(), &desired, "fixture-key", &store, || Ok(())).unwrap_err();
    assert_eq!(
        cleanup.downcast_ref::<CredentialRecovery>(),
        Some(&CredentialRecovery::StoreUnavailable)
    );
    assert!(home.path().join(JOURNAL).is_file());
}

#[test]
fn failed_rollbacks_preserve_the_initial_installation_failure_and_each_restore_error() {
    let home = tempfile::tempdir().unwrap();
    let desired = binding(Source::KeyringV2);
    let store = FakeStore::default();
    let error = persist(
        home.path(),
        &desired,
        "PRIVATE-TOKEN-CANARY",
        &store,
        || {
            // Directories cannot be removed as files, so both rollback attempts fail.
            std::fs::create_dir(home.path().join("config.toml"))?;
            std::fs::create_dir(home.path().join("credential-binding.json"))?;
            Err(anyhow::anyhow!("Injected installation failure")
                .context(CredentialRecovery::OutcomeUnknown))
        },
    )
    .unwrap_err();

    assert!(home.path().join(JOURNAL).is_file());
    assert!(home.path().join("config.toml").is_dir());
    assert!(home.path().join("credential-binding.json").is_dir());
    let diagnostic = format!("{error:#}");
    assert!(!diagnostic.contains("PRIVATE-TOKEN-CANARY"));
    assert!(
        diagnostic.contains("Injected installation failure"),
        "rollback must retain the initiating installation failure: {diagnostic}"
    );
    assert!(diagnostic.contains("Credential configuration rollback failed"));
    assert!(diagnostic.contains("Credential binding rollback failed"));
    assert_eq!(
        error.downcast_ref::<CredentialRecovery>(),
        Some(&CredentialRecovery::OutcomeUnknown)
    );

    // Once the filesystem obstruction is resolved, retry only the pending entry.
    std::fs::remove_dir(home.path().join("config.toml")).unwrap();
    std::fs::remove_dir(home.path().join("credential-binding.json")).unwrap();
    recover(home.path(), &store).unwrap();
    assert!(!home.path().join(JOURNAL).exists());
    assert!(store.values.borrow().is_empty());
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
    store.values.borrow_mut().insert(
        (StoreKind::WorkspaceKeyringV1, unrelated),
        "unrelated-key".into(),
    );
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
        BTreeMap::from([(
            (StoreKind::WorkspaceKeyringV1, unrelated),
            "unrelated-key".into()
        )])
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
    store.values.borrow_mut().insert(
        (StoreKind::WorkspaceKeyringV1, binding.id),
        "fixture-key".into(),
    );
    store.fail_read.set(true);
    assert!(
        persist(home.path(), &binding, "fixture-key", &store, || panic!(
            "must not install"
        ))
        .is_err()
    );
    assert_eq!(
        *store.values.borrow(),
        BTreeMap::from([(
            (StoreKind::WorkspaceKeyringV1, binding.id),
            "fixture-key".into()
        )])
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
    store.values.borrow_mut().insert(
        (StoreKind::WorkspaceKeyringV1, binding.id),
        "fixture-key".into(),
    );
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
        BTreeMap::from([(
            (StoreKind::WorkspaceKeyringV1, binding.id),
            "fixture-key".into()
        )])
    );
    assert!(!home.path().join(JOURNAL).exists());
}

#[test]
fn oversized_cleanup_journal_cannot_trigger_deletion() {
    let home = tempfile::tempdir().unwrap();
    let store = FakeStore::default();
    let account = Uuid::new_v4();
    store.values.borrow_mut().insert(
        (StoreKind::WorkspaceKeyringV1, account),
        "existing-key".into(),
    );
    std::fs::write(home.path().join(JOURNAL), vec![b'x'; 1025]).unwrap();
    assert!(recover(home.path(), &store).is_err());
    assert_eq!(
        *store.values.borrow(),
        BTreeMap::from([(
            (StoreKind::WorkspaceKeyringV1, account),
            "existing-key".into()
        )])
    );
}

#[test]
fn same_uuid_in_legacy_binding_does_not_own_pending_v2_entry() {
    let home = tempfile::tempdir().unwrap();
    let binding = binding(Source::Keyring);
    std::fs::write(
        home.path().join("credential-binding.json"),
        serde_json::to_vec(&binding).unwrap(),
    )
    .unwrap();
    let store = FakeStore::default();
    store.values.borrow_mut().extend([
        (
            (StoreKind::WorkspaceKeyringV1, binding.id),
            "legacy-key".into(),
        ),
        (
            (StoreKind::WorkspaceKeyringV2, binding.id),
            "uncommitted-v2-key".into(),
        ),
    ]);
    std::fs::write(
        home.path().join(JOURNAL),
        serde_json::to_vec(&Pending {
            schema_version: 1,
            store: StoreKind::WorkspaceKeyringV2,
            account: binding.id,
        })
        .unwrap(),
    )
    .unwrap();
    recover(home.path(), &store).unwrap();
    assert_eq!(
        *store.values.borrow(),
        BTreeMap::from([(
            (StoreKind::WorkspaceKeyringV1, binding.id),
            "legacy-key".into()
        ),])
    );
}

#[test]
fn failed_v2_install_preserves_legacy_namespace_and_binding() {
    let home = tempfile::tempdir().unwrap();
    let mut desired = binding(Source::Keyring);
    let previous = serde_json::to_vec(&desired).unwrap();
    std::fs::write(home.path().join("credential-binding.json"), &previous).unwrap();
    let store = FakeStore::default();
    store.values.borrow_mut().insert(
        (StoreKind::WorkspaceKeyringV1, desired.id),
        "legacy-key".into(),
    );
    desired.source = Some(Source::KeyringV2);
    assert!(
        persist(home.path(), &desired, "v2-key", &store, || anyhow::bail!(
            "Injected install failure"
        ))
        .is_err()
    );
    assert_eq!(
        *store.values.borrow(),
        BTreeMap::from([(
            (StoreKind::WorkspaceKeyringV1, desired.id),
            "legacy-key".into()
        ),])
    );
    assert_eq!(
        std::fs::read(home.path().join("credential-binding.json")).unwrap(),
        previous
    );
    assert!(!home.path().join(JOURNAL).exists());
}

#[test]
fn interrupted_v2_commit_recovers_without_deleting_owned_credential() {
    let home = tempfile::tempdir().unwrap();
    let desired = binding(Source::KeyringV2);
    let store = FakeStore::default();
    store.fail_read.set(true);
    store.fail_delete.set(true);
    assert!(
        persist(home.path(), &desired, "v2-key", &store, || panic!(
            "must not install"
        ))
        .is_err()
    );
    let pending: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.path().join(JOURNAL)).unwrap()).unwrap();
    assert_eq!(
        pending,
        serde_json::json!({"schema_version":1,"store":"workspace-keyring-v2","account":desired.id})
    );
    std::fs::write(
        home.path().join("credential-binding.json"),
        serde_json::to_vec(&desired).unwrap(),
    )
    .unwrap();
    recover(home.path(), &store).unwrap();
    assert_eq!(
        *store.values.borrow(),
        BTreeMap::from([((StoreKind::WorkspaceKeyringV2, desired.id), "v2-key".into()),])
    );
    assert!(!home.path().join(JOURNAL).exists());
}

fn oidc_binding() -> Binding {
    let identity = serde_json::from_value(serde_json::json!({
        "config":{"issuer":"https://idp.example/realms/test","client_id":"client","audience":"gateway"},
        "subject":"user", "display_name":null
    })).unwrap();
    binding(Source::Oidc { identity })
}

#[test]
fn fresh_oidc_save_read_and_cancelled_commit_failures_leave_no_unowned_token() {
    for failure in ["save", "read", "commit"] {
        let home = tempfile::tempdir().unwrap();
        let desired = oidc_binding();
        let store = FakeStore::default();
        store.fail_save.set(failure == "save");
        store.fail_read.set(failure == "read");
        let error = persist(
            home.path(),
            &desired,
            "private-encoded-oidc-bundle",
            &store,
            || {
                assert_eq!(failure, "commit");
                anyhow::bail!("Login was cancelled by a newer logout epoch")
            },
        )
        .unwrap_err();
        assert!(!format!("{error:#}").contains("private-encoded-oidc-bundle"));
        assert_eq!(*store.values.borrow(), BTreeMap::new());
        assert!(!home.path().join(JOURNAL).exists());
        assert!(!home.path().join("credential-binding.json").exists());
    }
}

#[test]
fn failed_oidc_cleanup_retries_only_the_recorded_native_format() {
    let home = tempfile::tempdir().unwrap();
    let desired = oidc_binding();
    let store = FakeStore::default();
    store.values.borrow_mut().insert(
        (StoreKind::WorkspaceKeyringV2, desired.id),
        "separate-workspace-key".into(),
    );
    store.fail_read.set(true);
    store.fail_delete.set(true);
    assert!(
        persist(
            home.path(),
            &desired,
            "private-encoded-oidc-bundle",
            &store,
            || panic!("must not install")
        )
        .is_err()
    );
    let pending: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.path().join(JOURNAL)).unwrap()).unwrap();
    assert_eq!(
        pending,
        serde_json::json!({"schema_version":1,"store":"oidc-identity-v1","account":desired.id})
    );
    store.fail_delete.set(false);
    recover(home.path(), &store).unwrap();
    assert_eq!(
        *store.values.borrow(),
        BTreeMap::from([(
            (StoreKind::WorkspaceKeyringV2, desired.id),
            "separate-workspace-key".into()
        ),])
    );
    assert!(!home.path().join(JOURNAL).exists());
}

#[cfg(unix)]
#[test]
fn cleanup_rejects_symlinks_without_touching_the_referenced_account() {
    let home = tempfile::tempdir().unwrap();
    let store = FakeStore::default();
    let account = Uuid::new_v4();
    store.values.borrow_mut().insert(
        (StoreKind::WorkspaceKeyringV1, account),
        "existing-key".into(),
    );
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
        BTreeMap::from([(
            (StoreKind::WorkspaceKeyringV1, account),
            "existing-key".into()
        )])
    );
}

#[test]
fn replacement_retires_only_the_superseded_account_and_retries_failed_deletion() {
    let home = tempfile::tempdir().unwrap();
    let store = FakeStore::default();
    let previous = binding(Source::KeyringV2);
    let replacement = binding(Source::Oidc {
        identity: codex_airs_identity::Identity {
            config: codex_airs_identity::IdentityConfig {
                issuer: "https://identity.example".into(),
                client_id: "harness".into(),
                audience: "gateway".into(),
            },
            subject: "fixture-subject".into(),
            display_name: None,
        },
    });
    store
        .save(StoreKind::WorkspaceKeyringV2, previous.id, "old-key")
        .unwrap();
    store
        .save(StoreKind::OidcIdentityV1, replacement.id, "new-identity")
        .unwrap();
    std::fs::write(
        home.path().join("credential-binding.json"),
        serde_json::to_vec(&replacement).unwrap(),
    )
    .unwrap();

    store.fail_delete.set(true);
    assert!(retire(home.path(), &previous, &store).is_err());
    assert!(home.path().join(JOURNAL).exists());
    assert_eq!(store.values.borrow().len(), 2);

    store.fail_delete.set(false);
    recover(home.path(), &store).unwrap();
    assert!(!home.path().join(JOURNAL).exists());
    assert_eq!(
        store.values.borrow().keys().copied().collect::<Vec<_>>(),
        vec![(StoreKind::OidcIdentityV1, replacement.id)]
    );

    // Referenced files and variables are never deleted by the harness.
    retire(
        home.path(),
        &binding(Source::File {
            path: home.path().join("key"),
        }),
        &store,
    )
    .unwrap();
    assert!(!home.path().join(JOURNAL).exists());
}
