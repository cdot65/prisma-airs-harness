use super::*;
use pretty_assertions::assert_eq;

#[test]
fn registry_rejects_unknown_versions_and_dangling_active_environment() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("environments.json");
    std::fs::write(
        &path,
        r#"{"schema_version":2,"active":null,"environments":{}}"#,
    )
    .unwrap();
    assert!(read(temp.path()).is_err());
    std::fs::write(
        &path,
        r#"{"schema_version":1,"active":"missing","environments":{}}"#,
    )
    .unwrap();
    assert!(read(temp.path()).is_err());
}

#[test]
fn rename_does_not_change_state_path_and_recreation_has_new_namespace() {
    let root = Path::new("/state");
    let first = Environment {
        id: Uuid::new_v4(),
        gateway_url: "https://one.example/v1".into(),
    };
    let original = environment_home(root, &first);
    let mut registry = Registry::default();
    registry.environments.insert("work".into(), first);
    let renamed = registry.environments.remove("work").unwrap();
    registry.environments.insert("team".into(), renamed);
    assert_eq!(
        environment_home(root, &registry.environments["team"]),
        original
    );
    let recreated = Environment {
        id: Uuid::new_v4(),
        gateway_url: "https://one.example/v1".into(),
    };
    assert_ne!(environment_home(root, &recreated), original);
}

#[test]
fn atomic_write_replaces_complete_document_privately() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("record.json");
    atomic_write(&path, b"old").unwrap();
    atomic_write(&path, b"new document").unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"new document");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[test]
fn recovery_commands_parse_valid_names_without_changing_selection() {
    use clap::Parser;
    for name in ["staging", "-staging", "--", "team.env_2"] {
        validate_name(name).unwrap();
        let command = command(Some(name));
        let parsed = crate::MultitoolCli::try_parse_from(
            command
                .split_whitespace()
                .chain(["doctor", "--verify-access"]),
        )
        .unwrap();
        assert_eq!(parsed.environment.as_deref(), Some(name));
    }
    assert_eq!(command(None), "airs");
}

#[test]
fn recovery_name_tracks_bound_home_instead_of_default_and_handles_legacy_home() {
    let root = tempfile::tempdir().unwrap();
    let args = super::super::airs_harness::SetupArgs {
        gateway_url: "https://gateway.example/v1".into(),
        ..Default::default()
    };
    let selected = create(root.path(), "staging", &args).unwrap();
    create(root.path(), "work", &args).unwrap();
    let before = std::fs::read(root.path().join("environments.json")).unwrap();
    assert_eq!(
        name_for_home(root.path(), &selected).unwrap(),
        Some("staging".into())
    );
    assert_eq!(name_for_home(root.path(), root.path()).unwrap(), None);
    assert_eq!(
        std::fs::read(root.path().join("environments.json")).unwrap(),
        before
    );
}

#[test]
fn loaded_registry_rejects_unsafe_names_without_rewriting_or_echoing_them() {
    for name in [
        "team;PRIVATE-CANARY",
        "team\nPRIVATE-CANARY",
        "team\u{1b}[31mPRIVATE-CANARY",
    ] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("environments.json");
        let bytes = serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "active": name,
            "environments": {
                name: {"id": Uuid::new_v4(), "gateway_url": "https://gateway.example/v1"}
            }
        }))
        .unwrap();
        std::fs::write(&path, &bytes).unwrap();
        for result in [
            read(root.path()).map(|_| ()),
            resolve(root.path(), Some(name)).map(|_| ()),
        ] {
            let diagnostic = result.unwrap_err().to_string();
            assert_eq!(
                diagnostic,
                "environment name must contain 1–64 letters, digits, dots, underscores or hyphens"
            );
            assert!(!diagnostic.contains("PRIVATE-CANARY"));
        }
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}
