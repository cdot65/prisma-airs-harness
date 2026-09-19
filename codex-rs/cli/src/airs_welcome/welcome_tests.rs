use super::*;
use pretty_assertions::assert_eq;

#[test]
fn resolving_multiple_environments_preserves_the_saved_default_and_history() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let args = airs_harness::SetupArgs {
        gateway_url: "https://gateway.example/v1".into(),
        ..Default::default()
    };
    let first = airs_environment::create(root, "work", &args).unwrap();
    let second = airs_environment::create(root, "staging", &args).unwrap();
    std::fs::write(first.join("history.jsonl"), "first history").unwrap();
    let registry = std::fs::read(root.join("environments.json")).unwrap();
    assert_eq!(
        airs_environment::resolve(root, Some("work")).unwrap(),
        Some(airs_environment::Selection {
            name: Some("work".into()),
            home: first.clone(),
            gateway: args.gateway_url.clone()
        })
    );
    assert_eq!(
        airs_environment::resolve(root, None).unwrap(),
        Some(airs_environment::Selection {
            name: Some("staging".into()),
            home: second,
            gateway: args.gateway_url
        })
    );
    assert_eq!(
        std::fs::read(root.join("environments.json")).unwrap(),
        registry
    );
    assert_eq!(
        std::fs::read_to_string(first.join("history.jsonl")).unwrap(),
        "first history"
    );
}

#[test]
fn resolving_a_changed_gateway_refuses_to_cross_the_binding_boundary() {
    let directory = tempfile::tempdir().unwrap();
    let args = airs_harness::SetupArgs {
        gateway_url: "https://gateway.example/v1".into(),
        ..Default::default()
    };
    let home = airs_environment::create(directory.path(), "work", &args).unwrap();
    let path = home.join("config.toml");
    let changed = std::fs::read_to_string(&path)
        .unwrap()
        .replace("gateway.example", "other.example");
    std::fs::write(&path, &changed).unwrap();
    assert!(airs_environment::resolve(directory.path(), Some("work")).is_err());
    assert_eq!(std::fs::read_to_string(path).unwrap(), changed);
}

#[test]
fn animation_preference_uses_the_environment_and_last_explicit_override() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("config.toml"),
        "[tui]\nanimations = false\n",
    )
    .unwrap();
    assert!(!options(directory.path(), &[]).animations);
    assert!(
        options(
            directory.path(),
            &[
                "tui.animations = false".into(),
                "tui.animations=true".into()
            ]
        )
        .animations
    );
    assert!(!options(directory.path(), &["model=\"tui.animations=true\"".into()]).animations);
}

#[test]
fn selected_environment_cancellation_has_snapshot_coverage() {
    insta::assert_snapshot!(cancelled(Some("staging")).to_string(), @r"
    Sign-in cancelled. Existing environments are preserved; resume with airs --environment staging login.
    ");
    insta::assert_snapshot!(cancelled(None).to_string(), @r"
    Sign-in cancelled. Existing environments are preserved; resume with airs login.
    ");
}
