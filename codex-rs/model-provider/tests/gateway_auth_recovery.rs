#![cfg(unix)]

use codex_login::auth::AgentIdentityAuthPolicy;
use codex_login::auth::CredentialRecovery;
use codex_model_provider::ProviderAuthScope;
use codex_model_provider::create_model_provider;
use codex_protocol::protocol::CodexErrorInfo;
use codex_protocol::protocol::SessionSource;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn gateway_request_preserves_real_helper_recovery_instructions() {
    for reason in [
        CredentialRecovery::SignInRequired,
        CredentialRecovery::OutcomeUnknown,
        CredentialRecovery::StoreUnavailable,
        CredentialRecovery::TemporarilyUnavailable,
    ] {
        let provider = create_model_provider(
            serde_json::from_value(serde_json::json!({
                "name": "AIRS",
                "base_url": "https://gateway.example/v1",
                "requires_openai_auth": false,
                "gateway": {"default_route": "airs-gateway-default"},
                "auth": {
                    "command": "/bin/sh",
                    "args": ["-c", format!("printf '%s\\n' '{}' >&2; exit 1", reason.marker())],
                    "cwd": std::env::temp_dir()
                }
            }))
            .unwrap(),
            /*auth_manager*/ None,
        );
        // Use the same provider dispatch entry point as the inference client,
        // including its selection between custom and first-party auth paths.
        let result = provider
            .api_auth_for_scope(ProviderAuthScope {
                agent_identity_policy: AgentIdentityAuthPolicy::JwtOnly,
                session_source: SessionSource::Cli,
                agent_identity_session_fallback: Default::default(),
            })
            .await;
        let error = result
            .err()
            .expect("unavailable credentials must stop dispatch");
        let sign_in = matches!(
            reason,
            CredentialRecovery::SignInRequired | CredentialRecovery::OutcomeUnknown
        );
        assert_eq!(
            error.to_codex_protocol_error() == CodexErrorInfo::Unauthorized,
            sign_in
        );
        assert!(error.to_string().contains(&reason.to_string()));
        assert_eq!(error.to_string().contains("/signin"), sign_in);
        assert!(
            !error
                .to_string()
                .contains("could not supply the bound credential")
        );
    }
}

#[tokio::test]
async fn gateway_request_with_available_helper_credential_keeps_bearer_auth() {
    let provider = create_model_provider(
        serde_json::from_value(serde_json::json!({
            "name": "AIRS",
            "base_url": "https://gateway.example/v1",
            "requires_openai_auth": false,
            "gateway": {"default_route": "airs-gateway-default"},
            "auth": {
                "command": "/bin/sh",
                "args": ["-c", "printf '%s\\n' fixture-access-token"],
                "cwd": std::env::temp_dir()
            }
        }))
        .unwrap(),
        /*auth_manager*/ None,
    );
    let resolved = provider
        .api_auth_for_scope(ProviderAuthScope {
            agent_identity_policy: AgentIdentityAuthPolicy::JwtOnly,
            session_source: SessionSource::Cli,
            agent_identity_session_fallback: Default::default(),
        })
        .await
        .unwrap();
    assert_eq!(
        resolved.auth.to_auth_headers()[http::header::AUTHORIZATION],
        "Bearer fixture-access-token"
    );
}
