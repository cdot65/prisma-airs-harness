use super::*;
use pretty_assertions::assert_eq;
use std::io::Cursor;

fn home() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.toml"),
        "[model_providers.airs]\nbase_url = 'https://gateway.example/v1'\n[model_providers.airs.env_http_headers]\nx-portkey-api-key = 'AIRS_API_KEY'\n",
    )
    .unwrap();
    dir
}

fn config() -> IdentityConfig {
    IdentityConfig {
        resource: None,
        scopes: Vec::new(),
        issuer: "https://identity.example/realms/company".to_owned(),
        client_id: "harness-desktop".to_owned(),
        audience: "gateway-inference".to_owned(),
    }
}

fn args() -> LoginArgs {
    let identity = config();
    LoginArgs {
        issuer_url: Some(identity.issuer),
        oidc_client_id: Some(identity.client_id),
        audience: Some(identity.audience),
        ..Default::default()
    }
}

#[test]
fn settings_roundtrip_has_only_public_gateway_bound_inputs() {
    let home = home();
    remember_settings(home.path(), &args()).unwrap();
    assert_eq!(read_settings(home.path()).unwrap(), Some(config()));
    let actual: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.path().join(SETTINGS_FILE)).unwrap()).unwrap();
    assert_eq!(
        actual,
        serde_json::json!({
            "schema_version": 1,
            "gateway_url": "https://gateway.example/v1",
            "identity": {
                "issuer": "https://identity.example/realms/company",
                "client_id": "harness-desktop",
                "audience": "gateway-inference"
            }
        })
    );
}

#[test]
fn changed_gateway_cannot_reuse_public_settings() {
    let home = home();
    remember_settings(home.path(), &args()).unwrap();
    std::fs::write(
        home.path().join("config.toml"),
        "[model_providers.airs]\nbase_url = 'https://different.example/v1'\n",
    )
    .unwrap();
    assert!(read_settings(home.path()).is_err());
}

#[test]
fn rejects_insecure_issuers_and_terminal_controls_without_replacing_settings() {
    let home = home();
    remember_settings(home.path(), &args()).unwrap();
    let before = std::fs::read(home.path().join(SETTINGS_FILE)).unwrap();
    for issuer in [
        "http://identity.example/realms/company",
        "https://user:password@identity.example/realms/company",
        "https://identity.example/realms/company?token=secret",
        "https://identity.example/realms/company#fragment",
        "https://identity.example/realms/company/",
        "https://identity.example/realms/\x1b[2J",
        "https://identity.example/realms/\ncompany",
    ] {
        let input = LoginArgs {
            issuer_url: Some(issuer.to_owned()),
            ..args()
        };
        assert!(remember_settings(home.path(), &input).is_err());
    }
    assert_eq!(
        std::fs::read(home.path().join(SETTINGS_FILE)).unwrap(),
        before
    );
}

#[test]
fn rejects_empty_oversized_and_nonprintable_identity_fields() {
    for value in ["".to_owned(), "\x1b[2J".to_owned(), "a".repeat(2049)] {
        let mut identity = config();
        identity.client_id = value.clone();
        assert!(validate_identity(&identity).is_err());
        let mut identity = config();
        identity.audience = value;
        assert!(validate_identity(&identity).is_err());
    }
}

#[test]
fn malformed_settings_never_echo_their_contents() {
    let home = home();
    let path = home.path().join(SETTINGS_FILE);
    std::fs::write(&path, "SECRET-CANARY-DO-NOT-ECHO").unwrap();
    let error = read_settings(home.path()).unwrap_err();
    assert_eq!(error.to_string(), "Invalid saved company sign-in settings");
    std::fs::write(path, "x".repeat(MAX_SETTINGS_BYTES as usize + 1)).unwrap();
    assert!(read_settings(home.path()).is_err());
}

#[test]
fn unknown_fields_and_settings_versions_are_rejected() {
    let home = home();
    remember_settings(home.path(), &args()).unwrap();
    let path = home.path().join(SETTINGS_FILE);
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value["token"] = serde_json::json!("fake-token");
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(read_settings(home.path()).is_err());
    value.as_object_mut().unwrap().remove("token");
    value["schema_version"] = serde_json::json!(2);
    std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(read_settings(home.path()).is_err());
}

#[test]
fn bounded_prompt_handles_crlf_empty_and_limit() {
    for (text, expected) in [
        ("hello\r\n".to_owned(), "hello".to_owned()),
        ("\n".to_owned(), String::new()),
        (format!("{}\n", "a".repeat(2048)), "a".repeat(2048)),
    ] {
        let value = prompt(&mut Cursor::new(text), &mut Vec::new(), "Input: ").unwrap();
        assert_eq!(value, expected);
    }
}

#[test]
fn bounded_prompt_rejects_eof_controls_and_overflow() {
    for text in [
        String::new(),
        "unterminated".to_owned(),
        "escape\x1b[2J\n".to_owned(),
        "line\rbreak\n".to_owned(),
        format!("{}\n", "a".repeat(2049)),
    ] {
        assert!(prompt(&mut Cursor::new(text), &mut Vec::new(), "Input: ").is_err());
    }
}

#[test]
fn saved_settings_are_not_printed_until_validated() {
    let home = home();
    remember_settings(home.path(), &args()).unwrap();
    let path = home.path().join(SETTINGS_FILE);
    let mut settings: Settings = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    settings.identity.client_id = "\x1b[2JINJECTED".to_owned();
    std::fs::write(path, serde_json::to_vec(&settings).unwrap()).unwrap();
    let mut output = Vec::new();
    assert!(company_settings(home.path(), &mut Cursor::new("\n"), &mut output).is_err());
    let output = String::from_utf8(output).unwrap();
    assert!(!output.contains("INJECTED"));
    assert!(output.contains("Type 1 to enter replacement settings"));
}

#[test]
fn guided_company_prompts_collect_and_reuse_settings() {
    let home = home();
    let mut input = Cursor::new(
        "https://identity.example/realms/company\nharness-desktop\ngateway-inference\n",
    );
    let mut output = Vec::new();
    assert_eq!(
        company_settings(home.path(), &mut input, &mut output).unwrap(),
        config()
    );
    remember_settings(home.path(), &args()).unwrap();
    assert_eq!(
        company_settings(home.path(), &mut Cursor::new("\n"), &mut output).unwrap(),
        config()
    );
    insta::assert_snapshot!(String::from_utf8(output).unwrap(), @r"

    Use the public sign-in settings supplied by your organization.
    Your company password is entered only in your browser.
    Company issuer URL (HTTPS): Public client ID: Gateway audience: 
    Company issuer: https://identity.example/realms/company
    Client ID: harness-desktop
    Audience: gateway-inference
    Press Enter to continue with these settings, or type 2 to change them: 
    ");
}

#[test]
fn explicit_settings_change_requires_keyboard_choice() {
    let home = home();
    remember_settings(home.path(), &args()).unwrap();
    let mut input = Cursor::new("2\nhttps://other.example/company\nother-client\nother-audience\n");
    assert_eq!(
        company_settings(home.path(), &mut input, &mut Vec::new()).unwrap(),
        IdentityConfig {
            resource: None,
            scopes: Vec::new(),
            issuer: "https://other.example/company".to_owned(),
            client_id: "other-client".to_owned(),
            audience: "other-audience".to_owned(),
        }
    );
    // Merely collecting new settings does not mutate credentials or saved settings.
    assert_eq!(read_settings(home.path()).unwrap(), Some(config()));
}

#[cfg(unix)]
#[test]
fn settings_file_cannot_be_a_symbolic_link() {
    let home = home();
    let target = home.path().join("other.json");
    std::fs::write(&target, "{}").unwrap();
    std::os::unix::fs::symlink(target, home.path().join(SETTINGS_FILE)).unwrap();
    assert!(read_settings(home.path()).is_err());
}

#[test]
fn established_oidc_binding_takes_precedence_over_public_settings() {
    let home = home();
    remember_settings(home.path(), &args()).unwrap();
    let binding_config = IdentityConfig {
        resource: None,
        scopes: Vec::new(),
        issuer: "https://bound.example/company".to_owned(),
        client_id: "bound-client".to_owned(),
        audience: "bound-audience".to_owned(),
    };
    let binding = airs_credentials::Binding {
        schema_version: 1,
        id: uuid::Uuid::new_v4(),
        gateway_url: "https://gateway.example/v1".to_owned(),
        credential_fingerprint: "fixture-fingerprint".to_owned(),
        source: Some(Source::Oidc {
            identity: codex_airs_identity::Identity {
                config: binding_config.clone(),
                subject: "fixture-subject".to_owned(),
                display_name: None,
            },
        }),
    };
    std::fs::write(
        home.path().join("credential-binding.json"),
        serde_json::to_vec(&binding).unwrap(),
    )
    .unwrap();
    assert_eq!(
        company_settings(home.path(), &mut Cursor::new("\n"), &mut Vec::new()).unwrap(),
        binding_config
    );
}

#[cfg(unix)]
#[test]
fn saved_settings_are_owner_only() {
    use std::os::unix::fs::MetadataExt;
    let home = home();
    remember_settings(home.path(), &args()).unwrap();
    let metadata = std::fs::metadata(home.path().join(SETTINGS_FILE)).unwrap();
    assert_eq!(metadata.mode() & 0o077, 0);
}

fn binding(home: &Path, source: Option<Source>) {
    let binding = Binding {
        schema_version: 1,
        id: uuid::Uuid::new_v4(),
        gateway_url: "https://gateway.example/v1".to_owned(),
        credential_fingerprint: "fixture-fingerprint".to_owned(),
        source,
    };
    std::fs::write(
        home.join("credential-binding.json"),
        serde_json::to_vec(&binding).unwrap(),
    )
    .unwrap();
}

#[test]
fn unfinished_onboarding_prompts_unless_legacy_environment_credential_is_present() {
    let home = home();
    assert!(needs_login(home.path(), |_| false).unwrap());
    assert!(!needs_login(home.path(), |name| name == "AIRS_API_KEY").unwrap());
}

#[test]
fn active_bindings_are_left_to_normal_credential_validation() {
    let home = home();
    for source in [
        Source::Keyring,
        Source::Environment {
            variable: "MISSING_EXPLICIT_KEY".to_owned(),
        },
        Source::File {
            path: home.path().join("missing-explicit-key"),
        },
        Source::Oidc {
            identity: codex_airs_identity::Identity {
                config: config(),
                subject: "fixture-subject".to_owned(),
                display_name: None,
            },
        },
    ] {
        binding(home.path(), Some(source));
        assert!(
            !needs_login(home.path(), |_| {
                panic!("existing credentials must not fall back to a default variable")
            })
            .unwrap()
        );
    }
}

#[test]
fn logout_requests_sign_in_without_forgetting_the_identity_boundary() {
    let home = home();
    binding(home.path(), /*source*/ None);
    let before = std::fs::read(home.path().join("credential-binding.json")).unwrap();
    assert!(needs_login(home.path(), |_| true).unwrap());
    std::fs::write(home.path().join("logged-out"), "signed out").unwrap();
    assert!(needs_login(home.path(), |_| true).unwrap());
    assert_eq!(
        std::fs::read(home.path().join("credential-binding.json")).unwrap(),
        before
    );
}

#[test]
fn malformed_binding_is_not_treated_as_a_fresh_or_logged_out_account() {
    let home = home();
    std::fs::write(home.path().join("credential-binding.json"), "{}").unwrap();
    std::fs::write(home.path().join("logged-out"), "signed out").unwrap();
    assert!(needs_login(home.path(), |_| false).is_err());
    let mut output = Vec::new();
    assert!(company_settings(home.path(), &mut Cursor::new("1\n"), &mut output).is_err());
    assert_eq!(output, Vec::<u8>::new());
}

#[test]
fn corrupt_public_settings_replacement_requires_explicit_confirmation() {
    let home = home();
    let path = home.path().join(SETTINGS_FILE);
    std::fs::write(&path, "INVALID-SETTINGS-CANARY").unwrap();
    let mut output = Vec::new();
    assert!(company_settings(home.path(), &mut Cursor::new("\n"), &mut output).is_err());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "INVALID-SETTINGS-CANARY"
    );
    assert!(!String::from_utf8(output).unwrap().contains("CANARY"));

    let mut output = Vec::new();
    let replacement = company_settings(
        home.path(),
        &mut Cursor::new(
            "1\nhttps://identity.example/realms/company\nharness-desktop\ngateway-inference\n",
        ),
        &mut output,
    )
    .unwrap();
    assert_eq!(replacement, config());
    remember_settings(home.path(), &args()).unwrap();
    assert_eq!(read_settings(home.path()).unwrap(), Some(config()));
    insta::assert_snapshot!(String::from_utf8(output).unwrap(), @r"
    Saved company sign-in settings are invalid or unavailable.
    Type 1 to enter replacement settings, or press Enter to cancel: 
    Use the public sign-in settings supplied by your organization.
    Your company password is entered only in your browser.
    Company issuer URL (HTTPS): Public client ID: Gateway audience: 
    ");
}

#[cfg(unix)]
#[test]
fn dangling_binding_link_is_not_treated_as_an_unsigned_environment() {
    let home = home();
    std::os::unix::fs::symlink(
        home.path().join("missing-binding"),
        home.path().join("credential-binding.json"),
    )
    .unwrap();
    assert!(needs_login(home.path(), |_| false).is_err());
}
