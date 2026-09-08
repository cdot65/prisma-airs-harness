use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

fn environment(source: serde_json::Value) -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        home.path().join("config.toml"),
        "[model_providers.airs]\nbase_url = 'https://gateway.example/v1'\n",
    )
    .unwrap();
    std::fs::write(
        home.path().join("credential-binding.json"),
        serde_json::to_vec(&json!({
            "schema_version":1, "id":uuid::Uuid::new_v4(),
            "gateway_url":"https://gateway.example/v1",
            "credential_fingerprint":airs_credentials::fingerprint("fixture-key"), "source":source
        }))
        .unwrap(),
    )
    .unwrap();
    home
}

fn update_binding(home: &Path, update: impl FnOnce(&mut serde_json::Value)) {
    let path = home.join("credential-binding.json");
    let mut value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    update(&mut value);
    std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
}

#[test]
fn native_metadata_does_not_require_any_stored_secret() {
    for kind in ["keyring", "keyring-v2"] {
        let home = environment(json!({"kind":kind}));
        let before = std::fs::read(home.path().join("credential-binding.json")).unwrap();
        let result = inspect(home.path()).unwrap();
        assert!(result.authentication.contains("saved OS-store binding"));
        assert!(
            result
                .detail
                .contains("availability and gateway access not checked")
        );
        assert_eq!(
            std::fs::read(home.path().join("credential-binding.json")).unwrap(),
            before
        );
    }
}

#[test]
fn oidc_saved_identity_is_bounded_and_never_claims_authentication() {
    let identity = codex_airs_identity::Identity {
        config: codex_airs_identity::IdentityConfig {
            issuer: "https://identity.example/realm".into(),
            client_id: "harness".into(),
            audience: "inference".into(),
        },
        subject: "saved-user".into(),
        display_name: None,
    };
    let home = environment(json!({"kind":"oidc", "identity":identity}));
    update_binding(home.path(), |value| {
        value["credential_fingerprint"] = json!(
            super::super::airs_oidc::fingerprint("https://gateway.example/v1", &identity).unwrap()
        )
    });
    let result = inspect(home.path()).unwrap();
    assert!(result.authentication.contains("not freshly authenticated"));
    assert!(result.authentication.contains("saved-user"));
    for subject in [
        "injected\u{1b}[2J".to_owned(),
        "a".repeat(2049),
        "spoof\u{202e}".into(),
    ] {
        update_binding(home.path(), |value| {
            value["source"]["identity"]["subject"] = json!(subject)
        });
        let error = inspect(home.path()).unwrap_err().to_string();
        assert_eq!(error, "Invalid saved authentication metadata");
    }
}

#[test]
fn invalid_or_inactive_bindings_fail_without_repair() {
    for (field, value) in [
        ("schema_version", json!(99)),
        ("source", json!(null)),
        ("gateway_url", json!("https://different.example/v1")),
        ("credential_fingerprint", json!("invalid")),
    ] {
        let home = environment(json!({"kind":"keyring-v2"}));
        update_binding(home.path(), |binding| binding[field] = value);
        assert!(inspect(home.path()).is_err());
    }
    let home = environment(json!({"kind":"keyring-v2"}));
    std::fs::write(
        home.path().join("auth-generation"),
        format!("v1 revoked {}\n", "a".repeat(32)),
    )
    .unwrap();
    assert!(inspect(home.path()).is_err());
    std::fs::remove_file(home.path().join("auth-generation")).unwrap();
    std::fs::write(home.path().join("logged-out"), "").unwrap();
    assert!(inspect(home.path()).is_err());
}

#[cfg(unix)]
#[test]
fn file_reference_still_checks_private_file_and_bound_identity() {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("credential");
    std::fs::write(&path, "fixture-key").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let home = environment(json!({"kind":"file", "path":path}));
    assert!(
        inspect(home.path())
            .unwrap()
            .detail
            .contains("available locally")
    );
    std::fs::write(&path, "changed-key").unwrap();
    assert!(inspect(home.path()).is_err());
    std::fs::write(&path, "fixture-key").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(inspect(home.path()).is_err());
}

#[test]
fn missing_environment_reference_remains_an_error() {
    let variable = format!("AIRS_MISSING_{}", uuid::Uuid::new_v4().simple());
    let home = environment(json!({"kind":"environment", "variable":variable}));
    assert!(inspect(home.path()).is_err());
    std::fs::remove_file(home.path().join("credential-binding.json")).unwrap();
    std::fs::write(home.path().join("config.toml"), format!(
        "[model_providers.airs]\nbase_url = 'https://gateway.example/v1'\n[model_providers.airs.env_http_headers]\nx-portkey-api-key = '{variable}'\n")).unwrap();
    assert!(inspect(home.path()).is_err());
}

#[test]
fn invalid_gateway_and_oversized_public_metadata_fail_closed() {
    let home = environment(json!({"kind":"keyring"}));
    for gateway in [
        "http://gateway.example/v1",
        "https://user:secret@gateway.example/v1",
        "https://gateway.example/v1?secret=canary",
    ] {
        std::fs::write(
            home.path().join("config.toml"),
            format!("[model_providers.airs]\nbase_url = '{gateway}'\n"),
        )
        .unwrap();
        let error = inspect(home.path()).unwrap_err().to_string();
        assert_eq!(error, "Invalid saved gateway URL");
    }
    std::fs::write(
        home.path().join("config.toml"),
        "x".repeat(MAX_PUBLIC_BYTES as usize + 1),
    )
    .unwrap();
    assert!(
        inspect(home.path())
            .unwrap_err()
            .to_string()
            .contains("at most 1 MiB")
    );
}

#[cfg(unix)]
#[test]
fn metadata_rejects_symlinks_and_fifos_without_blocking() {
    let home = environment(json!({"kind":"keyring"}));
    let binding = home.path().join("credential-binding.json");
    let target = home.path().join("saved-binding");
    std::fs::rename(&binding, &target).unwrap();
    std::os::unix::fs::symlink(&target, &binding).unwrap();
    assert!(inspect(home.path()).is_err());
    std::fs::remove_file(&binding).unwrap();
    let path = std::ffi::CString::new(binding.as_os_str().as_encoded_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
    assert!(inspect(home.path()).is_err());
}
