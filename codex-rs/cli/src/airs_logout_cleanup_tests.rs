use super::*;
use pretty_assertions::assert_eq;
use std::cell::Cell;
use std::cell::RefCell;

#[derive(Default)]
struct FakeStore {
    fail: Cell<bool>,
    calls: RefCell<Vec<(StoreKind, Uuid)>>,
}

impl Store for FakeStore {
    fn delete(&self, kind: StoreKind, account: Uuid) -> anyhow::Result<()> {
        self.calls.borrow_mut().push((kind, account));
        anyhow::ensure!(!self.fail.get(), "Injected native deletion failure");
        Ok(())
    }
}

fn home(source: Source) -> (tempfile::TempDir, Binding) {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(home.path().join("logged-out"), "explicit logout").unwrap();
    std::fs::write(
        home.path().join("config.toml"),
        "[model_providers.airs]\nbase_url='https://gateway.example/v1'\n",
    )
    .unwrap();
    let binding = Binding {
        schema_version: 1,
        id: Uuid::new_v4(),
        gateway_url: "https://gateway.example/v1".into(),
        credential_fingerprint: "preserve-identity-boundary".into(),
        source: Some(source),
    };
    write_binding(home.path(), &binding);
    (home, binding)
}

fn write_binding(home: &Path, binding: &Binding) {
    std::fs::write(
        home.join("credential-binding.json"),
        serde_json::to_vec(binding).unwrap(),
    )
    .unwrap();
}

#[test]
fn deletion_failure_preserves_typed_journal_for_workspace_and_oidc_retries() {
    let identity: codex_airs_identity::Identity = serde_json::from_value(serde_json::json!({
        "config":{"issuer":"https://idp.example/realms/test","client_id":"client","audience":"gateway"},
        "subject":"user", "display_name":null
    })).unwrap();
    for (source, kind) in [
        (Source::Keyring, StoreKind::WorkspaceKeyringV1),
        (Source::Oidc { identity }, StoreKind::OidcIdentityV1),
    ] {
        let (home, binding) = home(source);
        prepare(home.path(), &binding).unwrap();
        let store = FakeStore::default();
        store.fail.set(true);
        assert!(recover(home.path(), &store).is_err());
        assert!(home.path().join(JOURNAL).exists());
        let disabled = super::super::read_binding(home.path()).unwrap();
        assert!(disabled.source.is_none());
        assert_eq!(
            disabled.credential_fingerprint,
            binding.credential_fingerprint
        );
        let journal: serde_json::Value =
            serde_json::from_slice(&std::fs::read(home.path().join(JOURNAL)).unwrap()).unwrap();
        assert_eq!(
            journal,
            serde_json::json!({"schema_version":1,"account":binding.id,"store":kind})
        );
        store.fail.set(false);
        recover(home.path(), &store).unwrap();
        assert!(!home.path().join(JOURNAL).exists());
        assert_eq!(
            *store.calls.borrow(),
            vec![(kind, binding.id), (kind, binding.id)]
        );
    }
}

#[test]
fn stale_logout_cannot_delete_another_working_binding() {
    let (home, binding) = home(Source::Keyring);
    prepare(home.path(), &binding).unwrap();
    let different = Binding {
        id: Uuid::new_v4(),
        ..binding
    };
    write_binding(home.path(), &different);
    let before = std::fs::read(home.path().join("credential-binding.json")).unwrap();
    let store = FakeStore::default();
    assert!(recover(home.path(), &store).is_err());
    assert!(store.calls.borrow().is_empty());
    assert_eq!(
        std::fs::read(home.path().join("credential-binding.json")).unwrap(),
        before
    );
}

#[test]
fn unmarked_state_cannot_resume_destructive_cleanup() {
    let (home, binding) = home(Source::Keyring);
    prepare(home.path(), &binding).unwrap();
    std::fs::remove_file(home.path().join("logged-out")).unwrap();
    let store = FakeStore::default();
    assert!(recover(home.path(), &store).is_err());
    assert!(store.calls.borrow().is_empty());
    assert!(
        super::super::read_binding(home.path())
            .unwrap()
            .source
            .is_some()
    );
}

#[test]
fn malformed_cleanup_and_binding_diagnostics_never_include_file_values() {
    let (home, binding) = home(Source::Keyring);
    let store = FakeStore::default();
    std::fs::write(home.path().join(JOURNAL), br#"{"store":"PRIVATE-CANARY"}"#).unwrap();
    let error = recover(home.path(), &store).unwrap_err();
    assert!(!format!("{error:#}").contains("PRIVATE-CANARY"));
    std::fs::remove_file(home.path().join(JOURNAL)).unwrap();
    prepare(home.path(), &binding).unwrap();
    let mut malformed = serde_json::to_value(&binding).unwrap();
    malformed["source"] = serde_json::json!({"kind":"PRIVATE-CANARY"});
    std::fs::write(
        home.path().join("credential-binding.json"),
        serde_json::to_vec(&malformed).unwrap(),
    )
    .unwrap();
    let error = recover(home.path(), &store).unwrap_err();
    assert!(!format!("{error:#}").contains("PRIVATE-CANARY"));
    assert!(store.calls.borrow().is_empty());
}
