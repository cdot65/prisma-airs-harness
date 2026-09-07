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
