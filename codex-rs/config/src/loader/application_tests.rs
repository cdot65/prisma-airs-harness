use super::*;
use crate::CloudRequirementsFragment;
use crate::ConfigRequirementsToml;
use crate::loader::tests::TestFileSystem;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn local_policy_is_validated_before_cloud_and_capture_is_stable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("requirements.toml");
    let contents = "[application.network.domains]\n'gateway.example' = 'deny'";
    std::fs::write(&path, contents).unwrap();
    let mut overrides = LoaderOverrides::without_managed_config_for_tests();
    overrides.system_requirements_path = Some(path.clone());
    let captured = load_local_application_requirements(&TestFileSystem, &overrides)
        .await
        .unwrap();
    // A later filesystem change cannot silently alter this captured generation.
    std::fs::write(&path, "[application.network]\nenabled = false").unwrap();
    assert_eq!(
        captured
            .compose(CloudRequirementsTomlBundle::default())
            .unwrap(),
        toml::from_str::<ConfigRequirementsToml>(contents)
            .unwrap()
            .application,
    );
    std::fs::write(&path, "[application.network]\nenabled = 'invalid'").unwrap();
    assert!(
        load_local_application_requirements(&TestFileSystem, &overrides)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn cloud_priority_matches_normal_composition_without_unrelated_bootstrap_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("requirements.toml");
    std::fs::write(
        &path,
        "allowed_approval_policies = 42\n[application.network.domains]\n'gateway.example' = 'deny'",
    )
    .unwrap();
    let mut overrides = LoaderOverrides::without_managed_config_for_tests();
    overrides.system_requirements_path = Some(path);
    // Bootstrap only interprets application restrictions, not unrelated settings.
    let local = load_local_application_requirements(&TestFileSystem, &overrides)
        .await
        .unwrap();
    let cloud = CloudRequirementsTomlBundle {
        enterprise_managed: vec![
            CloudRequirementsFragment {
                id: "high".into(), name: "high".into(),
                contents: "[application.network.domains]\n'gateway.example' = 'deny'".into(),
            },
            CloudRequirementsFragment {
                id: "low".into(), name: "low".into(),
                contents: "[application.network.domains]\n'gateway.example' = 'allow'\n'additional.example' = 'allow'".into(),
            },
        ],
    };
    let expected = toml::from_str::<ConfigRequirementsToml>(
        "[application.network.domains]\n'gateway.example' = 'deny'\n'additional.example' = 'allow'",
    )
    .unwrap()
    .application;
    assert_eq!(local.compose(cloud).unwrap(), expected);
}

#[tokio::test]
async fn malformed_cloud_application_is_not_ignored() {
    let local = LocalApplicationRequirements::default();
    let cloud = CloudRequirementsTomlBundle {
        enterprise_managed: vec![CloudRequirementsFragment {
            id: "invalid".into(),
            name: "invalid".into(),
            contents: "[application.network.domains]\n'*.example' = 'allow'".into(),
        }],
    };
    assert!(local.compose(cloud).is_err());
}
