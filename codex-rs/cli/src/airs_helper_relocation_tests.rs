use super::*;
use pretty_assertions::assert_eq;
use uuid::Uuid;

struct Fixture {
    _temporary: tempfile::TempDir,
    home: std::path::PathBuf,
    old: std::path::PathBuf,
    new: std::path::PathBuf,
    mcp: Uuid,
}

impl Fixture {
    fn new() -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let home = temporary.path().canonicalize().unwrap();
        let old = home.join("old user's installation").join("airs-harness");
        let new = home.join("new installation").join("airs-harness");
        let id = Uuid::new_v4();
        let mcp = Uuid::new_v4();
        let key = home.join("synthetic-key");
        airs_environment::atomic_write(&key, b"synthetic-relocation-key").unwrap();
        airs_environment::atomic_write(&home.join("models.json"), b"synthetic catalog").unwrap();
        let binding = serde_json::json!({"schema_version":1,"id":id,
            "gateway_url":"https://gateway.example/v1",
            "credential_fingerprint":airs_credentials::fingerprint("synthetic-relocation-key"),
            "source":{"kind":"file","path":key}});
        airs_environment::atomic_write(
            &home.join("credential-binding.json"),
            &serde_json::to_vec(&binding).unwrap(),
        )
        .unwrap();
        let config: toml::Value = serde_json::from_value(serde_json::json!({
            "model_catalog_json":home.join("models.json"),"model_context_window":1000000,
            "model_providers":{"airs":{"base_url":"https://gateway.example/v1","auth":{
                "command":old,"args":["credential","--home",home,"--binding",id],
                "timeout_ms":60000,"refresh_interval_ms":1000,"cwd":home}}}
        }))
        .unwrap();
        std::fs::write(home.join("config.toml"), toml::to_string(&config).unwrap()).unwrap();
        Self {
            _temporary: temporary,
            home,
            old,
            new,
            mcp,
        }
    }

    fn config(&self) -> toml::Value {
        toml::from_str(&std::fs::read_to_string(self.home.join("config.toml")).unwrap()).unwrap()
    }

    fn write(&self, value: &toml::Value) {
        airs_environment::atomic_write(
            &self.home.join("config.toml"),
            toml::to_string_pretty(value).unwrap().as_bytes(),
        )
        .unwrap();
    }

    fn add_mcp(&self) {
        let mut config = self.config();
        config.as_table_mut().unwrap().insert("mcp_servers".into(), serde_json::from_value(serde_json::json!({
            "scanner":{"url":"https://scanner.example/mcp","enabled_tools":["scan"],
                "http_headers_helper":airs_mcp::helper_command(&self.old,&self.home,self.mcp).unwrap()}
        })).unwrap());
        self.write(&config);
        std::fs::create_dir(self.home.join("mcp-bindings")).unwrap();
        let binding = serde_json::json!({"schema_version":1,"id":self.mcp,"server":"scanner",
            "url":"https://scanner.example/mcp","credential_file":self.home.join("synthetic-key"),
            "credential_fingerprint":airs_credentials::fingerprint("synthetic-relocation-key")});
        airs_environment::atomic_write(
            &self
                .home
                .join("mcp-bindings")
                .join(format!("{}.json", self.mcp)),
            &serde_json::to_vec(&binding).unwrap(),
        )
        .unwrap();
    }

    fn seed_legacy(&self) {
        crate::airs_session_binding::validate_locked(&self.home).unwrap();
        let path = self.home.join("session-binding.json");
        let mut revision: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        if let Some(servers) = self.config().get("mcp_servers") {
            revision["mcp_config_revision"] = serde_json::json!(airs_credentials::fingerprint(
                &toml::to_string(servers).unwrap()
            ));
        }
        airs_environment::atomic_write(&path, &serde_json::to_vec_pretty(&revision).unwrap())
            .unwrap();
    }
}

#[test]
fn inference_path_relocation_preserves_identity_and_session_bytes() {
    let fixture = Fixture::new();
    fixture.seed_legacy();
    std::fs::create_dir_all(fixture.old.parent().unwrap()).unwrap();
    std::fs::write(&fixture.old, b"existing stale executable is never executed").unwrap();
    let binding = std::fs::read(fixture.home.join("credential-binding.json")).unwrap();
    let revision = std::fs::read(fixture.home.join("session-binding.json")).unwrap();
    let mut expected = fixture.config();
    expected["model_providers"]["airs"]["auth"]["command"] =
        toml::Value::String(fixture.new.to_str().unwrap().into());
    rewrite_locked(&fixture.home, &fixture.new).unwrap();
    crate::airs_session_binding::validate_locked(&fixture.home).unwrap();
    assert_eq!(fixture.config(), expected);
    assert_eq!(
        std::fs::read(fixture.home.join("credential-binding.json")).unwrap(),
        binding
    );
    assert_eq!(
        std::fs::read(fixture.home.join("session-binding.json")).unwrap(),
        revision
    );
}

#[test]
fn normalized_revision_first_recovers_before_and_after_configuration_write() {
    let fixture = Fixture::new();
    fixture.add_mcp();
    fixture.seed_legacy();
    let config = std::fs::read(fixture.home.join("config.toml")).unwrap();
    let legacy = std::fs::read(fixture.home.join("session-binding.json")).unwrap();
    crate::airs_session_binding::validate_locked(&fixture.home).unwrap();
    let normalized = std::fs::read(fixture.home.join("session-binding.json")).unwrap();
    assert_ne!(legacy, normalized);
    assert_eq!(
        std::fs::read(fixture.home.join("config.toml")).unwrap(),
        config
    );
    // Simulate restart after revision commit, while old executable paths remain.
    crate::airs_session_binding::validate_locked(&fixture.home).unwrap();
    rewrite_locked(&fixture.home, &fixture.new).unwrap();
    assert_eq!(
        fixture.config()["mcp_servers"]["scanner"]["http_headers_helper"].as_str(),
        Some(
            airs_mcp::helper_command(&fixture.new, &fixture.home, fixture.mcp)
                .unwrap()
                .as_str()
        )
    );
    // A subsequent installation move needs no further identity revision change.
    crate::airs_session_binding::validate_locked(&fixture.home).unwrap();
    rewrite_locked(&fixture.home, &fixture.old).unwrap();
    crate::airs_session_binding::validate_locked(&fixture.home).unwrap();
    assert_eq!(
        std::fs::read(fixture.home.join("session-binding.json")).unwrap(),
        normalized
    );
}

#[test]
fn configuration_first_cannot_bypass_legacy_history_validation() {
    let fixture = Fixture::new();
    fixture.add_mcp();
    fixture.seed_legacy();
    let revision = std::fs::read(fixture.home.join("session-binding.json")).unwrap();
    rewrite_locked(&fixture.home, &fixture.new).unwrap();
    assert!(crate::airs_session_binding::validate_locked(&fixture.home).is_err());
    assert_eq!(
        std::fs::read(fixture.home.join("session-binding.json")).unwrap(),
        revision
    );
}

#[test]
fn legacy_and_normalized_revisions_reject_unrelated_changes_before_writing() {
    for normalized in [false, true] {
        for field in ["url", "tools", "args", "identity", "home", "binding"] {
            let fixture = Fixture::new();
            fixture.add_mcp();
            fixture.seed_legacy();
            if normalized {
                crate::airs_session_binding::validate_locked(&fixture.home).unwrap();
            }
            let previous = std::fs::read(fixture.home.join("session-binding.json")).unwrap();
            let mut config = fixture.config();
            let server = &mut config["mcp_servers"]["scanner"];
            match field {
                "url" => server["url"] = toml::Value::String("https://other.example/mcp".into()),
                "tools" => {
                    server["enabled_tools"] =
                        toml::Value::Array(vec![toml::Value::String("other".into())])
                }
                "args" => {
                    server["http_headers_helper"] = toml::Value::String(format!(
                        "{} --extra",
                        server["http_headers_helper"].as_str().unwrap()
                    ))
                }
                "home" => {
                    server["http_headers_helper"] = toml::Value::String(
                        airs_mcp::helper_command(
                            &fixture.old,
                            &fixture.home.join("other"),
                            fixture.mcp,
                        )
                        .unwrap(),
                    )
                }
                "binding" => {
                    server["http_headers_helper"] = toml::Value::String(
                        airs_mcp::helper_command(&fixture.old, &fixture.home, Uuid::new_v4())
                            .unwrap(),
                    )
                }
                "identity" => {
                    let path = fixture.home.join("credential-binding.json");
                    let mut binding: serde_json::Value =
                        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                    binding["credential_fingerprint"] = serde_json::json!("changed");
                    std::fs::write(path, serde_json::to_vec(&binding).unwrap()).unwrap();
                }
                _ => unreachable!(),
            }
            fixture.write(&config);
            assert!(
                crate::airs_session_binding::validate_locked(&fixture.home).is_err(),
                "accepted {field}"
            );
            assert_eq!(
                std::fs::read(fixture.home.join("session-binding.json")).unwrap(),
                previous
            );
            assert_eq!(fixture.config(), config);
        }
    }
}

#[test]
fn arbitrary_custom_helpers_stay_literal_and_cannot_impersonate_normalized_metadata() {
    let fixture = Fixture::new();
    fixture.add_mcp();
    let mut config = fixture.config();
    config["model_providers"]["airs"]["auth"]["command"] =
        toml::Value::String("my-custom-auth".into());
    config["mcp_servers"]["scanner"]["http_headers_helper"] =
        toml::Value::String("custom-helper --headers".into());
    fixture.write(&config);
    fixture.seed_legacy();
    let revision = std::fs::read(fixture.home.join("session-binding.json")).unwrap();
    rewrite_locked(&fixture.home, &fixture.new).unwrap();
    crate::airs_session_binding::validate_locked(&fixture.home).unwrap();
    assert_eq!(fixture.config(), config);
    assert_eq!(
        std::fs::read(fixture.home.join("session-binding.json")).unwrap(),
        revision
    );

    let managed = Fixture::new();
    managed.add_mcp();
    managed.seed_legacy();
    crate::airs_session_binding::validate_locked(&managed.home).unwrap();
    let mut forged = managed.config();
    forged["mcp_servers"]["scanner"]["http_headers_helper"] = toml::Value::String(format!(
        "airs-managed-mcp-revision-v1 --home {} --binding {}",
        managed.home.display(),
        managed.mcp
    ));
    managed.write(&forged);
    assert!(crate::airs_session_binding::validate_locked(&managed.home).is_err());
}

#[test]
fn malformed_binding_and_unbounded_metadata_are_rejected() {
    let fixture = Fixture::new();
    fixture.add_mcp();
    let path = fixture
        .home
        .join("mcp-bindings")
        .join(format!("{}.json", fixture.mcp));
    std::fs::write(&path, b"malformed synthetic metadata").unwrap();
    assert!(mcp_revision(&fixture.home, &fixture.config()).is_err());
    std::fs::write(&path, vec![b'x'; 16_385]).unwrap();
    assert!(mcp_revision(&fixture.home, &fixture.config()).is_err());
}

#[test]
fn native_oauth_catalog_changes_preserve_inference_and_legacy_helper_history() {
    for legacy_helper in [false, true] {
        let fixture = Fixture::new();
        if legacy_helper {
            fixture.add_mcp();
        }
        crate::airs_session_binding::validate_locked(&fixture.home).unwrap();
        let revision = std::fs::read(fixture.home.join("session-binding.json")).unwrap();
        let mut config = fixture.config();
        config
            .as_table_mut()
            .unwrap()
            .entry("mcp_servers")
            .or_insert_with(|| toml::Value::Table(Default::default()))
            .as_table_mut()
            .unwrap()
            .insert(
                "native".into(),
                toml::toml! {
                    url = "https://native.example/mcp"
                    scopes = ["read"]
                    [oauth]
                    client_id = "native-client"
                    callback_url = "http://127.0.0.1/callback"
                }
                .into(),
            );
        fixture.write(&config);
        crate::airs_session_binding::validate_locked(&fixture.home).unwrap();
        assert_eq!(
            std::fs::read(fixture.home.join("session-binding.json")).unwrap(),
            revision
        );
        config["mcp_servers"]
            .as_table_mut()
            .unwrap()
            .remove("native");
        fixture.write(&config);
        crate::airs_session_binding::validate_locked(&fixture.home).unwrap();
        assert_eq!(
            std::fs::read(fixture.home.join("session-binding.json")).unwrap(),
            revision
        );
    }
}

#[test]
fn typed_raw_helper_cannot_impersonate_owned_revision_metadata() {
    let fixture = Fixture::new();
    fixture.add_mcp();
    let mut config = fixture.config();
    config["mcp_servers"]["scanner"]["http_headers_helper"] =
        serde_json::from_value(serde_json::json!({
            "airs_managed_helper_revision_v1":{"home":fixture.home,"binding":fixture.mcp}
        }))
        .unwrap();
    assert!(mcp_revision(&fixture.home, &config).is_err());
}

#[cfg(unix)]
#[test]
fn metadata_links_and_nonregular_files_are_rejected_without_following() {
    let fixture = Fixture::new();
    let link = fixture.home.join("metadata-link");
    std::os::unix::fs::symlink(fixture.home.join("synthetic-key"), &link).unwrap();
    assert!(read(&link, 16_384).is_err());
    assert!(read(&fixture.home, 16_384).is_err());
    let fifo = fixture.home.join("metadata-fifo");
    let name = std::ffi::CString::new(fifo.as_os_str().as_encoded_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    assert!(read(&fifo, 16_384).is_err());
}
