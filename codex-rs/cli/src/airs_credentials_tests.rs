use super::*;
use pretty_assertions::assert_eq;

#[test]
fn credential_validation_rejects_controls_and_empty_values() {
    for token in ["", "  ", "key\nother", "key other", "key\u{7}"] {
        assert!(validate_token(token).is_err());
    }
    assert_eq!(validate_token(" test-key\n").unwrap(), "test-key");
}

#[cfg(unix)]
#[test]
fn file_source_rejects_public_files_symlinks_and_changed_identity() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("key");
    std::fs::write(&path, "test-key").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(file_token(&path).is_err());
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(file_token(&path).unwrap(), "test-key");
    let link = temp.path().join("link");
    std::os::unix::fs::symlink(&path, &link).unwrap();
    assert!(file_token(&link).is_err());
    let binding = Binding {
        schema_version: 1,
        id: Uuid::new_v4(),
        gateway_url: "https://one.example/v1".into(),
        credential_fingerprint: fingerprint("test-key"),
        source: Some(Source::File { path: path.clone() }),
    };
    assert_eq!(resolve(&binding).unwrap(), "test-key");
    std::fs::write(path, "different-key").unwrap();
    assert!(resolve(&binding).is_err());
}

#[test]
fn changed_gateway_and_logout_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let binding = Binding {
        schema_version: 1,
        id: Uuid::new_v4(),
        gateway_url: "https://one.example/v1".into(),
        credential_fingerprint: fingerprint("test-key"),
        source: None,
    };
    std::fs::write(
        temp.path().join("credential-binding.json"),
        serde_json::to_vec(&binding).unwrap(),
    )
    .unwrap();
    std::fs::write(
        temp.path().join("config.toml"),
        "[model_providers.airs]\nbase_url = 'https://one.example/v1'\n",
    )
    .unwrap();
    assert!(resolve(&read_binding(temp.path()).unwrap()).is_err());
    std::fs::write(
        temp.path().join("config.toml"),
        "[model_providers.airs]\nbase_url = 'https://other.example/v1'\n",
    )
    .unwrap();
    assert!(read_binding(temp.path()).is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn replacing_a_credential_requires_consent_and_repins_existing_history() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    std::fs::write(
        home.join("config.toml"),
        "[model_providers.airs]\nbase_url = 'https://gateway.example/v1'\n",
    )
    .unwrap();
    let key = |name: &str, value: &str| {
        let path = home.join(name);
        std::fs::write(&path, value).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        path
    };
    let args = |path: PathBuf, replace: bool| LoginArgs {
        credential_file: Some(path),
        replace,
        ..Default::default()
    };
    let first = key("first-key", "first-workspace-key");
    let second = key("second-key", "second-workspace-key");
    login(home, &args(first, false), /*stdin_key*/ false)
        .await
        .unwrap();
    let original = read_binding(home).unwrap();
    let pinned = serde_json::json!({
        "schema_version": 1,
        "gateway_url": "https://gateway.example/v1",
        "credential_identity": fingerprint("first-workspace-key"),
        "capability_revision": "fixture-capabilities",
        "context_window": 1000000,
        "mcp_config_revision": null,
    });
    std::fs::write(
        home.join("session-binding.json"),
        serde_json::to_vec(&pinned).unwrap(),
    )
    .unwrap();

    let error = login(home, &args(second.clone(), false), false)
        .await
        .unwrap_err();
    assert!(error.is::<CredentialChanged>());
    assert_eq!(
        read_binding(home).unwrap().credential_fingerprint,
        original.credential_fingerprint
    );

    login(home, &args(second, true), false).await.unwrap();
    let replaced = read_binding(home).unwrap();
    assert_eq!(
        replaced.credential_fingerprint,
        fingerprint("second-workspace-key")
    );
    assert_ne!(replaced.id, original.id);
    let config = std::fs::read_to_string(home.join("config.toml")).unwrap();
    assert!(config.contains(&replaced.id.to_string()));
    assert!(!config.contains(&original.id.to_string()));
    let mut expected = pinned;
    expected["credential_identity"] = fingerprint("second-workspace-key").into();
    let actual: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.join("session-binding.json")).unwrap()).unwrap();
    assert_eq!(actual, expected);
    assert_eq!(identity(home).unwrap(), fingerprint("second-workspace-key"));
}
