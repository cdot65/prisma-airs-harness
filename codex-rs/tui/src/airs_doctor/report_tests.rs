use super::*;
use crate::airs_doctor::Check;
use pretty_assertions::assert_eq;

#[test]
fn report_ignores_private_fields_and_uses_pessimistic_known_checks() {
    let mut report = Report {
        schema_version: 1,
        product: "PRIVATE-PRODUCT".into(),
        authentication: "PRIVATE-AUTH\u{202e}\x1b".into(),
        checks: vec![
            Check {
                name: "credential_service".into(),
                passed: false,
                detail: "PRIVATE-KEYRING-ERROR".into(),
            },
            Check {
                name: "configuration".into(),
                passed: true,
                detail: "https://PRIVATE-GATEWAY/path?code=PRIVATE-CODE".into(),
            },
            Check {
                name: "gateway_health".into(),
                passed: true,
                detail: "PRIVATE-HEADER".into(),
            },
            Check {
                name: "gateway_health".into(),
                passed: false,
                detail: "PRIVATE-ENVIRONMENT".into(),
            },
            Check {
                name: "PRIVATE-CHECK".into(),
                passed: false,
                detail: "PRIVATE-CONVERSATION".into(),
            },
        ],
    };
    let rendered = render(Some(&report));
    assert!(!rendered.contains("PRIVATE"));
    assert!(!rendered.contains("https://"));
    assert!(!rendered.contains('\u{202e}'));
    let normalized = rendered
        .replace(codex_utils_home_dir::AIRS_HARNESS_VERSION, "[VERSION]")
        .replace(
            &format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
            "[PLATFORM]",
        );
    insta::assert_snapshot!("airs_report_redacted_failure", normalized);
    for check in &mut report.checks {
        check.detail = "different secret /home/private/token".into();
    }
    report.authentication = "another unexpected auth value".into();
    report.product = "another private product".into();
    assert_eq!(render(Some(&report)), rendered);
}

#[test]
fn explicit_access_and_authentication_are_distinct_from_health() {
    let report = Report {
        schema_version: 1,
        product: "Prisma AIRS Harness".into(),
        authentication: "Workspace API key".into(),
        checks: vec![Check {
            name: "gateway_access".into(),
            passed: true,
            detail: "private correlation ID".into(),
        }],
    };
    let text = render(Some(&report));
    assert!(text.contains("Authentication: Workspace API key\n"));
    assert!(text.contains("gateway_access: PASS\n"));
    assert!(!text.contains("Not verified"));
    let unavailable = render(None);
    assert!(unavailable.contains("Authentication: Unknown\n"));
    assert!(unavailable.contains("gateway_access: Not verified\n"));
    assert!(unavailable.contains("Diagnostics unavailable. Retry /doctor."));
}

#[test]
fn local_save_preserves_existing_state_and_rejects_invalid_destination() {
    let directory = tempfile::tempdir().unwrap();
    let sentinel = directory.path().join("config.toml");
    std::fs::write(&sentinel, "private existing configuration").unwrap();
    let text = render(None);
    let first = save(directory.path(), &text).unwrap();
    let second = save(directory.path(), &text).unwrap();
    assert_ne!(first, second);
    assert_eq!(std::fs::read_to_string(&first).unwrap(), text.as_ref());
    assert_eq!(std::fs::read_to_string(&second).unwrap(), text.as_ref());
    assert_eq!(
        std::fs::read_to_string(&sentinel).unwrap(),
        "private existing configuration"
    );
    assert!(
        first
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("diagnostic-report-")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&first).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    assert!(save(&sentinel, &text).is_err());
    assert!(save(directory.path(), &"x".repeat(MAX_REPORT + 1)).is_err());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 3);
}
