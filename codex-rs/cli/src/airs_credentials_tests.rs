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
