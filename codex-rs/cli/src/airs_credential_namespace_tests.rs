use super::*;
use pretty_assertions::assert_eq;
use std::cell::RefCell;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Namespace {
    Legacy,
    WorkspaceV2,
}

fn namespace(kind: StoreKind) -> Namespace {
    match kind {
        StoreKind::WorkspaceKeyringV1 | StoreKind::OidcIdentityV1 => Namespace::Legacy,
        StoreKind::WorkspaceKeyringV2 => Namespace::WorkspaceV2,
    }
}

// Both legacy formats address the same native root. Deliberately do not model
// them as isolated entries merely because their serialized kinds differ.
#[derive(Default)]
struct SharedNamespaceStore {
    values: RefCell<BTreeMap<(Namespace, Uuid), String>>,
    mutations: RefCell<Vec<&'static str>>,
}

impl Store for SharedNamespaceStore {
    fn save(&self, kind: StoreKind, account: Uuid, value: &str) -> anyhow::Result<()> {
        self.mutations.borrow_mut().push("save");
        self.values
            .borrow_mut()
            .insert((namespace(kind), account), value.into());
        Ok(())
    }

    fn load(&self, kind: StoreKind, account: Uuid) -> anyhow::Result<Option<String>> {
        Ok(self
            .values
            .borrow()
            .get(&(namespace(kind), account))
            .cloned())
    }

    fn delete(&self, kind: StoreKind, account: Uuid) -> anyhow::Result<()> {
        self.mutations.borrow_mut().push("delete");
        self.values.borrow_mut().remove(&(namespace(kind), account));
        Ok(())
    }
}

fn binding(kind: StoreKind, id: Uuid) -> Binding {
    let source = match kind {
        StoreKind::WorkspaceKeyringV1 => Source::Keyring,
        StoreKind::WorkspaceKeyringV2 => Source::KeyringV2,
        StoreKind::OidcIdentityV1 => Source::Oidc {
            identity: serde_json::from_value(serde_json::json!({
                "config":{"issuer":"https://idp.example/realms/test","client_id":"client","audience":"gateway"},
                "subject":"user", "display_name":null
            })).unwrap(),
        },
    };
    Binding {
        schema_version: 1,
        id,
        gateway_url: "https://gateway.example/v1".into(),
        credential_fingerprint: "existing-identity".into(),
        source: Some(source),
    }
}

#[test]
fn conflicting_legacy_cleanup_preserves_shared_root_and_all_metadata() {
    for (live, pending) in [
        (StoreKind::OidcIdentityV1, StoreKind::WorkspaceKeyringV1),
        (StoreKind::WorkspaceKeyringV1, StoreKind::OidcIdentityV1),
    ] {
        let home = tempfile::tempdir().unwrap();
        let account = Uuid::new_v4();
        let binding = serde_json::to_vec(&binding(live, account)).unwrap();
        let journal = serde_json::to_vec(&Pending {
            schema_version: 1,
            store: pending,
            account,
        })
        .unwrap();
        std::fs::write(home.path().join("credential-binding.json"), &binding).unwrap();
        std::fs::write(home.path().join(JOURNAL), &journal).unwrap();
        let store = SharedNamespaceStore::default();
        store
            .values
            .borrow_mut()
            .insert((Namespace::Legacy, account), "live-native-root".into());
        let before = store.values.borrow().clone();
        assert!(recover(home.path(), &store).is_err());
        assert_eq!(*store.values.borrow(), before);
        assert_eq!(*store.mutations.borrow(), Vec::<&str>::new());
        assert_eq!(
            std::fs::read(home.path().join("credential-binding.json")).unwrap(),
            binding
        );
        assert_eq!(std::fs::read(home.path().join(JOURNAL)).unwrap(), journal);
    }
}

#[test]
fn conflicting_legacy_persistence_stops_before_even_unrelated_pending_cleanup() {
    for (live, desired) in [
        (StoreKind::OidcIdentityV1, StoreKind::WorkspaceKeyringV1),
        (StoreKind::WorkspaceKeyringV1, StoreKind::OidcIdentityV1),
    ] {
        let home = tempfile::tempdir().unwrap();
        let account = Uuid::new_v4();
        let current = serde_json::to_vec(&binding(live, account)).unwrap();
        std::fs::write(home.path().join("credential-binding.json"), &current).unwrap();
        let pending_account = Uuid::new_v4();
        let journal = serde_json::to_vec(&Pending {
            schema_version: 1,
            store: StoreKind::WorkspaceKeyringV2,
            account: pending_account,
        })
        .unwrap();
        std::fs::write(home.path().join(JOURNAL), &journal).unwrap();
        let store = SharedNamespaceStore::default();
        store.values.borrow_mut().extend([
            ((Namespace::Legacy, account), "live-native-root".into()),
            (
                (Namespace::WorkspaceV2, pending_account),
                "pending-v2-root".into(),
            ),
        ]);
        let before = store.values.borrow().clone();
        assert!(
            persist(
                home.path(),
                &binding(desired, account),
                "replacement",
                &store,
                || panic!("must not install")
            )
            .is_err()
        );
        assert_eq!(*store.values.borrow(), before);
        assert_eq!(*store.mutations.borrow(), Vec::<&str>::new());
        assert_eq!(
            std::fs::read(home.path().join("credential-binding.json")).unwrap(),
            current
        );
        assert_eq!(std::fs::read(home.path().join(JOURNAL)).unwrap(), journal);
    }
}

#[test]
fn separate_v2_namespace_cleanup_still_preserves_either_legacy_format() {
    for live in [StoreKind::WorkspaceKeyringV1, StoreKind::OidcIdentityV1] {
        let home = tempfile::tempdir().unwrap();
        let account = Uuid::new_v4();
        std::fs::write(
            home.path().join("credential-binding.json"),
            serde_json::to_vec(&binding(live, account)).unwrap(),
        )
        .unwrap();
        std::fs::write(
            home.path().join(JOURNAL),
            serde_json::to_vec(&Pending {
                schema_version: 1,
                store: StoreKind::WorkspaceKeyringV2,
                account,
            })
            .unwrap(),
        )
        .unwrap();
        let store = SharedNamespaceStore::default();
        store.values.borrow_mut().extend([
            ((Namespace::Legacy, account), "live-native-root".into()),
            (
                (Namespace::WorkspaceV2, account),
                "uncommitted-v2-root".into(),
            ),
        ]);
        recover(home.path(), &store).unwrap();
        assert_eq!(
            *store.values.borrow(),
            BTreeMap::from([((Namespace::Legacy, account), "live-native-root".into()),])
        );
        assert_eq!(*store.mutations.borrow(), vec!["delete"]);
        assert!(!home.path().join(JOURNAL).exists());
    }
}
