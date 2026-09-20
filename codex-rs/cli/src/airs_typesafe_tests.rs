use super::*;
use pretty_assertions::assert_eq;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Default)]
struct MockStore {
    entries: RefCell<HashMap<Uuid, String>>,
    fail_save: bool,
    fail_load: bool,
    fail_delete: bool,
}

impl Store for MockStore {
    fn save(&self, account: Uuid, token: &str) -> anyhow::Result<()> {
        anyhow::ensure!(!self.fail_save, "native store unavailable");
        self.entries.borrow_mut().insert(account, token.to_owned());
        Ok(())
    }
    fn load(&self, account: Uuid) -> anyhow::Result<Option<String>> {
        anyhow::ensure!(!self.fail_load, "readback unavailable");
        Ok(self.entries.borrow().get(&account).cloned())
    }
    fn delete(&self, account: Uuid) -> anyhow::Result<()> {
        anyhow::ensure!(!self.fail_delete, "delete unavailable");
        self.entries.borrow_mut().remove(&account);
        Ok(())
    }
}

#[test]
fn save_round_trips_settings_and_exports_saved_values() {
    let home = tempfile::tempdir().unwrap();
    let store = MockStore::default();
    let saved = save(
        home.path(),
        &store,
        "ts-key-1",
        Some("jev-1.13.0"),
        Some("https://api.typesafe.ai/"),
    )
    .unwrap();
    assert_eq!(
        read(home.path()).unwrap(),
        Some(Settings {
            schema_version: 1,
            id: saved.id,
            model: Some("jev-1.13.0".into()),
            base_url: Some("https://api.typesafe.ai".into()),
            key_fingerprint: crate::airs_credentials::fingerprint("ts-key-1"),
        })
    );
    let (origin, variables) = exports(home.path(), &store, |_| false).unwrap();
    assert_eq!(origin, Origin::Environment);
    assert_eq!(
        variables,
        vec![
            (KEY_VARIABLE, "ts-key-1".to_owned()),
            (BASE_URL_VARIABLE, "https://api.typesafe.ai".to_owned()),
            (MODEL_VARIABLE, "jev-1.13.0".to_owned()),
        ]
    );
}

#[test]
fn process_variable_wins_over_the_saved_key() {
    let home = tempfile::tempdir().unwrap();
    let store = MockStore::default();
    save(home.path(), &store, "ts-key-1", None, None).unwrap();
    let (origin, variables) = exports(home.path(), &store, |name| name == KEY_VARIABLE).unwrap();
    assert_eq!((origin, variables), (Origin::ProcessVariable, Vec::new()));
    assert!(
        status_detail(home.path(), |name| name == KEY_VARIABLE)
            .starts_with("Configured from process variable")
    );
}

#[test]
fn unset_environment_exports_nothing_and_reports_optional() {
    let home = tempfile::tempdir().unwrap();
    let store = MockStore::default();
    let (origin, variables) = exports(home.path(), &store, |_| false).unwrap();
    assert_eq!((origin, variables), (Origin::NotConfigured, Vec::new()));
    assert_eq!(
        status_detail(home.path(), |_| false),
        "Not configured (optional). Set with airs env typesafe set"
    );
    assert!(!clear(home.path(), &store).unwrap());
}

#[test]
fn replacing_a_key_removes_the_previous_native_entry() {
    let home = tempfile::tempdir().unwrap();
    let store = MockStore::default();
    let first = save(home.path(), &store, "ts-key-1", None, None).unwrap();
    let second = save(home.path(), &store, "ts-key-2", None, None).unwrap();
    assert_ne!(first.id, second.id);
    assert_eq!(
        store.entries.borrow().keys().copied().collect::<Vec<_>>(),
        vec![second.id]
    );
    assert!(clear(home.path(), &store).unwrap());
    assert!(store.entries.borrow().is_empty());
    assert_eq!(read(home.path()).unwrap(), None);
}

#[test]
fn a_failed_native_save_leaves_settings_untouched() {
    let home = tempfile::tempdir().unwrap();
    let working = MockStore::default();
    save(home.path(), &working, "ts-key-1", None, None).unwrap();
    let before = read(home.path()).unwrap();
    let failing = MockStore {
        fail_save: true,
        ..MockStore::default()
    };
    assert!(save(home.path(), &failing, "ts-key-2", None, None).is_err());
    assert_eq!(read(home.path()).unwrap(), before);
}

#[test]
fn fingerprint_mismatch_refuses_to_export_a_tampered_key() {
    let home = tempfile::tempdir().unwrap();
    let store = MockStore::default();
    let saved = save(home.path(), &store, "ts-key-1", None, None).unwrap();
    store
        .entries
        .borrow_mut()
        .insert(saved.id, "different".into());
    let error = exports(home.path(), &store, |_| false).unwrap_err();
    assert!(error.to_string().contains("fingerprint"), "{error}");
}

#[test]
fn invalid_inputs_are_rejected_before_any_store_write() {
    let home = tempfile::tempdir().unwrap();
    let store = MockStore::default();
    for (token, model, base_url) in [
        ("", None, None),
        ("has space", None, None),
        ("ok", Some("bad model!"), None),
        ("ok", None, Some("http://api.typesafe.ai")),
        ("ok", None, Some("https://user:pw@api.typesafe.ai")),
        ("ok", None, Some("https://api.typesafe.ai/?x=1")),
    ] {
        assert!(save(home.path(), &store, token, model, base_url).is_err());
    }
    assert!(store.entries.borrow().is_empty());
    assert_eq!(read(home.path()).unwrap(), None);
    assert_eq!(
        validate_base_url("http://127.0.0.1:8080/").unwrap(),
        "http://127.0.0.1:8080"
    );
}

#[test]
fn unreadable_settings_are_reported_not_ignored() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(home.path().join(SETTINGS_FILE), b"{\"schema_version\":1").unwrap();
    assert!(read(home.path()).is_err());
    assert!(status_detail(home.path(), |_| false).contains("Invalid saved TypeSafe settings"));
}

#[test]
fn child_command_keeps_credentials_out_of_arguments_and_parent_environment() {
    let home = tempfile::tempdir().unwrap();
    let store = MockStore::default();
    save(home.path(), &store, "child-only-test-key", None, None).unwrap();
    let before = std::env::var_os(KEY_VARIABLE);
    let args = vec!["python3".into(), "judge.py".into(), "--dry-run".into()];
    let command = child_command(home.path(), &store, &args).unwrap();
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        vec!["judge.py", "--dry-run"]
    );
    assert_eq!(std::env::var_os(KEY_VARIABLE), before);
    if before.is_none() {
        assert!(command.get_envs().any(|(key, value)| key == KEY_VARIABLE
            && value == Some(std::ffi::OsStr::new("child-only-test-key"))));
    }
}

#[test]
fn failed_readback_cleans_uncommitted_key_and_preserves_previous_binding() {
    let home = tempfile::tempdir().unwrap();
    let mut store = MockStore::default();
    let first = save(home.path(), &store, "original", None, None).unwrap();
    store.fail_load = true;
    assert!(save(home.path(), &store, "replacement", None, None).is_err());
    assert_eq!(read(home.path()).unwrap(), Some(first));
    assert_eq!(store.entries.borrow().len(), 1);
}

#[test]
fn failed_cleanup_is_journaled_and_clear_retries_it() {
    let home = tempfile::tempdir().unwrap();
    let mut store = MockStore::default();
    save(home.path(), &store, "original", None, None).unwrap();
    store.fail_delete = true;
    let error = save(home.path(), &store, "replacement", None, None).unwrap_err();
    assert!(error.to_string().contains("cleanup is pending"));
    assert_eq!(store.entries.borrow().len(), 2);
    assert!(home.path().join("typesafe-cleanup.json").exists());
    store.fail_delete = false;
    assert!(clear(home.path(), &store).unwrap());
    assert!(store.entries.borrow().is_empty());
    assert!(!home.path().join("typesafe-cleanup.json").exists());
}

#[tokio::test]
async fn model_probe_validates_the_response_and_refuses_redirects() {
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::header;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    for (status, body, expected) in [
        (200, r#"{"models":[]}"#, true),
        (200, r#"{"private":"not a models response"}"#, false),
        (401, "private", false),
        (302, "private", false),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .and(header("authorization", "Bearer fixture-key"))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_string(body)
                    .insert_header("Location", format!("{}/should-not-follow", server.uri())),
            )
            .expect(1)
            .mount(&server)
            .await;
        let result = probe_key("fixture-key", &server.uri()).await;
        assert_eq!(result.is_ok(), expected);
        if let Err(error) = result {
            assert!(!error.to_string().contains("private"));
        }
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}

#[test]
fn skill_binding_uses_owning_environment_not_default_and_rejects_mismatch() {
    let root = tempfile::tempdir().unwrap();
    let id = Uuid::new_v4();
    let other = Uuid::new_v4();
    let home = root.path().join("environments").join(id.to_string());
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(
        root.path().join("environments.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version":1,"active":"other","environments":{
                "judge":{"id":id,"gateway_url":"https://fixture.invalid/v1"},
                "other":{"id":other,"gateway_url":"https://fixture.invalid/v1"}
            }
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        skill_environment(root.path(), &home, None).unwrap(),
        Some("judge".into())
    );
    assert!(skill_environment(root.path(), &home, Some("other")).is_err());
    assert!(skill_environment(root.path(), root.path(), None).is_err());
    std::fs::write(
        root.path().join("environments.json"),
        br#"{"schema_version":1,"active":null,"environments":{}}"#,
    )
    .unwrap();
    assert!(skill_environment(root.path(), &home, None).is_err());
}

#[test]
fn skill_binding_supports_unregistered_legacy_root_only_without_named_selection() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(
        skill_environment(root.path(), root.path(), None).unwrap(),
        None
    );
    assert!(skill_environment(root.path(), root.path(), Some("missing")).is_err());
}
