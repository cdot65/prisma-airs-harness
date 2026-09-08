use super::delete_with;
use crate::CredentialStoreDiagnostic;
use crate::CredentialStoreError;
use crate::CredentialStoreErrorKind;
use crate::DefaultKeyringStore;
use crate::KeyringStore;
use pretty_assertions::assert_eq;

#[test]
fn failed_native_deletion_preserves_typed_status_for_retry() {
    for (code, kind) in [
        (-25293, CredentialStoreErrorKind::Denied),
        (-128, CredentialStoreErrorKind::Cancelled),
        (-25308, CredentialStoreErrorKind::InteractionRequired),
    ] {
        let error = delete_with("service", "account", |service, account| {
            assert_eq!((service, account), ("service", "account"));
            Err(security_framework::base::Error::from(code))
        })
        .expect_err("native deletion failure must not report success");
        assert_eq!(
            CredentialStoreError::new(error).diagnostic(),
            CredentialStoreDiagnostic {
                kind,
                native_code: Some(code),
            }
        );
    }
}

#[test]
fn missing_item_remains_idempotent_and_invalid_queries_never_execute() {
    let error = delete_with("service", "account", |_, _| {
        Err(security_framework::base::Error::from(-25300))
    })
    .expect_err("missing item");
    assert!(matches!(error, keyring::Error::NoEntry));
    for (service, account) in [("", "account"), ("service", "")] {
        let error = delete_with(service, account, |_, _| {
            panic!("invalid query must never reach native deletion")
        })
        .expect_err("empty keyring identity");
        assert!(matches!(error, keyring::Error::Invalid(_, _)));
    }
}

#[test]
fn native_deletion_preserves_other_accounts_and_services() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let service = format!(
        "io.cdot.airs-harness.delete-test.{}.{nonce}",
        std::process::id()
    );
    let other_service = format!("{service}.other");
    let store = DefaultKeyringStore;
    let entries = [
        (service.as_str(), "target"),
        (service.as_str(), "neighbor"),
        (other_service.as_str(), "target"),
    ];
    // Best-effort cleanup also runs when an assertion fails on a native runner.
    struct Cleanup<'a>(&'a [(&'a str, &'a str)]);
    impl Drop for Cleanup<'_> {
        fn drop(&mut self) {
            for (service, account) in self.0 {
                let _ = DefaultKeyringStore.delete(service, account);
            }
        }
    }
    let _cleanup = Cleanup(&entries);
    for (service, account) in entries {
        store
            .save(service, account, "synthetic-deletion-fixture")
            .unwrap();
    }
    assert!(store.delete(&service, "target").unwrap());
    assert_eq!(store.load(&service, "target").unwrap(), None);
    assert!(!store.delete(&service, "target").unwrap());
    for (service, account) in [entries[1], entries[2]] {
        assert_eq!(
            store.load(service, account).unwrap(),
            Some("synthetic-deletion-fixture".to_owned())
        );
    }
}
