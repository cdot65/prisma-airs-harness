use super::SetupArgs;
use super::configuration;
use codex_config::config_toml::ConfigToml;
use pretty_assertions::assert_eq;

#[test]
fn setup_context_default_and_override_reach_config_and_catalog() {
    use clap::Parser;

    let dir = tempfile::tempdir().unwrap();
    for (extra, expected) in [
        (Vec::new(), 1_000_000),
        (vec!["--context-window", "32768"], 32768),
    ] {
        let mut argv = vec![
            "airs-harness",
            "env",
            "create",
            "work",
            "--gateway-url",
            "https://gateway.example/v1",
        ];
        argv.extend(extra);
        let cli = crate::MultitoolCli::try_parse_from(argv).unwrap();
        let Some(crate::Subcommand::Env {
            command: Some(crate::airs_environment::Command::Create { args, .. }),
        }) = cli.subcommand
        else {
            panic!("expected setup arguments");
        };
        let (config, catalog) = configuration(&args, dir.path()).unwrap();
        let config: ConfigToml = toml::from_str(&config).unwrap();
        let catalog: serde_json::Value = serde_json::from_str(&catalog).unwrap();
        assert_eq!(config.model_context_window, Some(expected));
        assert_eq!(catalog["models"][0]["context_window"], expected);
        assert_eq!(catalog["models"][0]["max_context_window"], expected);
    }
}

fn args() -> SetupArgs {
    SetupArgs {
        gateway_url: "https://gateway.example.com/prefix/v1/".to_string(),
        credential_env: "TERMINAL_TEST_KEY".to_string(),
        context_window: 32768,
        model: vec!["@vllm/org/model:quantized".to_string()],
        allow_http_loopback: false,
    }
}

#[test]
fn setup_produces_valid_config_and_explicit_capabilities_without_credentials() {
    let dir = tempfile::tempdir().unwrap();
    let (text, catalog) = configuration(&args(), dir.path()).unwrap();
    let config: ConfigToml = toml::from_str(&text).unwrap();
    assert_eq!(config.model_provider.as_deref(), Some("airs"));
    let provider = config.model_providers.get("airs").unwrap();
    provider.validate().unwrap();
    assert_eq!(
        provider.base_url.as_deref(),
        Some("https://gateway.example.com/prefix/v1")
    );
    assert_eq!(
        provider.request_model("airs-gateway-default").unwrap(),
        None
    );
    assert_eq!(
        provider.request_model("@vllm/org/model:quantized").unwrap(),
        Some("@vllm/org/model:quantized".to_string())
    );
    let catalog: serde_json::Value = serde_json::from_str(&catalog).unwrap();
    assert_eq!(catalog["models"].as_array().unwrap().len(), 2);
    assert_eq!(catalog["models"][0]["context_window"], 32768);
    assert_eq!(
        provider
            .env_http_headers
            .as_ref()
            .unwrap()
            .get("x-portkey-api-key")
            .map(String::as_str),
        Some("TERMINAL_TEST_KEY")
    );
}

#[test]
fn setup_rejects_credential_urls_and_nonlocal_plain_http() {
    let dir = tempfile::tempdir().unwrap();
    for url in [
        "https://user:secret@example.com/v1",
        "https://example.com/v1?key=x",
        "https://example.com/v1#key",
        "http://example.com/v1",
        "https://example.com/v1/responses",
    ] {
        let mut config = args();
        config.gateway_url = url.to_string();
        config.allow_http_loopback = true;
        assert!(configuration(&config, dir.path()).is_err(), "{url}");
    }
    let mut config = args();
    config.gateway_url = "http://127.0.0.1:9876/v1".to_string();
    assert!(configuration(&config, dir.path()).is_err());
    config.allow_http_loopback = true;
    assert!(configuration(&config, dir.path()).is_ok());
}

#[test]
fn setup_rejects_ambiguous_models_and_missing_capability_limits() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = args();
    config.model = vec!["gpt-4.1".to_string()];
    assert!(configuration(&config, dir.path()).is_err());
    config = args();
    config.context_window = 0;
    assert!(configuration(&config, dir.path()).is_err());
    config = args();
    config.credential_env = "secret-value".to_string();
    assert!(configuration(&config, dir.path()).is_err());
}
#[test]
fn startup_authentication_recovery_preserves_selected_environment_and_restore_mode() {
    use clap::Parser;
    use codex_login::auth::CredentialRecovery;
    for environment in [None, Some("staging"), Some("-staging"), Some("--")] {
        for reason in [
            CredentialRecovery::SignInRequired,
            CredentialRecovery::OutcomeUnknown,
        ] {
            let message = super::startup_recovery(
                anyhow::Error::new(reason).context("private context"),
                environment,
            )
            .to_string();
            let command = message.split('`').nth(1).unwrap();
            let parsed = crate::MultitoolCli::try_parse_from(command.split_whitespace()).unwrap();
            assert_eq!(parsed.environment.as_deref(), environment);
            let Some(crate::Subcommand::Login(login)) = parsed.subcommand else {
                panic!("startup recovery must offer login");
            };
            assert!(login.airs.restore_session);
            assert!(!message.contains("/signin"));
            assert!(!message.contains("private context"));
            assert!(message.contains("retry your original command"));
        }
    }
}

#[test]
fn unrelated_startup_failures_keep_their_original_error_chain() {
    use codex_login::auth::CredentialRecovery;
    let error = anyhow::anyhow!("configuration unchanged").context("outer context");
    let expected = format!("{error:#}");
    assert_eq!(
        format!("{:#}", super::startup_recovery(error, Some("work"))),
        expected
    );
    for reason in [
        CredentialRecovery::StoreUnavailable,
        CredentialRecovery::TemporarilyUnavailable,
    ] {
        let error = super::startup_recovery(reason.into(), Some("work"));
        assert_eq!(error.downcast_ref::<CredentialRecovery>(), Some(&reason));
    }
}

#[test]
fn startup_recovery_messages_offer_shell_commands_before_the_tui_opens() {
    use codex_login::auth::CredentialRecovery;

    insta::assert_snapshot!(
        super::startup_recovery(CredentialRecovery::SignInRequired.into(), Some("work")).to_string(),
        @"Your work session has ended. Run `airs --environment work login --restore-session` to sign in as the same person, then retry your original command. Your saved conversations are preserved."
    );
    insta::assert_snapshot!(
        super::startup_recovery(CredentialRecovery::OutcomeUnknown.into(), Some("-staging")).to_string(),
        @"Your sign-in needs to be restored. Run `airs --environment=-staging login --restore-session` to sign in as the same person, then retry your original command. Your saved conversations are preserved."
    );
}
