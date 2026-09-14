use super::*;
use pretty_assertions::assert_eq;

struct RecoverableCredential {
    first: AtomicBool,
    reason: codex_login::auth::CredentialRecovery,
}

impl codex_login::ExternalAuth for RecoverableCredential {
    fn credential_recovery(&self) -> Option<codex_login::auth::CredentialRecovery> {
        Some(self.reason)
    }
    fn resolve(&self) -> codex_login::ExternalAuthFuture<'_, CodexAuth> {
        Box::pin(async move {
            if self.first.swap(false, Ordering::SeqCst) {
                Ok(CodexAuth::from_api_key("test-provider-token"))
            } else {
                Err(std::io::Error::other(self.reason))
            }
        })
    }
    fn refresh(
        &self,
        _: codex_login::ExternalAuthRefreshContext,
    ) -> codex_login::ExternalAuthFuture<'_, CodexAuth> {
        self.resolve()
    }
}

#[tokio::test]
async fn gateway_preserves_auth_recovery_classification_before_dispatch() {
    use codex_login::auth::CredentialRecovery;
    for reason in [
        CredentialRecovery::SignInRequired,
        CredentialRecovery::OutcomeUnknown,
        CredentialRecovery::StoreUnavailable,
        CredentialRecovery::TemporarilyUnavailable,
    ] {
        let manager = AuthManager::from_auth_for_testing(CodexAuth::from_api_key(
            "test-provider-token",
        ));
        manager
            .set_external_auth(Arc::new(RecoverableCredential {
                first: AtomicBool::new(true),
                reason,
            }))
            .await
            .unwrap();
        let auth = manager.auth().await;
        assert_eq!(auth, None);
        let provider: ModelProviderInfo = serde_json::from_value(serde_json::json!({
            "name":"AIRS", "base_url":"https://gateway.example/v1",
            "gateway":{"default_route":"airs-gateway-default"},
            "auth":{"command":"/unused/helper", "cwd":"/"}
        }))
        .unwrap();
        let result = resolve_provider_auth_for_scope(
            Some(manager),
            auth.as_ref(),
            &provider,
            ProviderAuthScope {
                agent_identity_policy: AgentIdentityAuthPolicy::JwtOnly,
                session_source: SessionSource::Cli,
                agent_identity_session_fallback: Default::default(),
            },
        )
        .await;
        let error = result.err().expect("credential failure must stop dispatch");
        assert_eq!(
            error.to_codex_protocol_error()
                == codex_protocol::protocol::CodexErrorInfo::Unauthorized,
            matches!(
                reason,
                CredentialRecovery::SignInRequired | CredentialRecovery::OutcomeUnknown
            )
        );
        assert!(error.to_string().starts_with(&reason.to_string()));
        if matches!(
            reason,
            CredentialRecovery::SignInRequired | CredentialRecovery::OutcomeUnknown
        ) {
            assert!(error.to_string().contains(
                "AIRS_HARNESS_HOME='/' airs-harness login --restore-session --no-browser"
            ));
        }
    }
}

#[test]
fn unavailable_gateway_command_credential_never_becomes_unauthenticated_request() {
    let provider: ModelProviderInfo = serde_json::from_value(serde_json::json!({
        "name": "AIRS", "base_url": "https://gateway.example/v1",
        "gateway": {"default_route": "airs-gateway-default"},
        "auth": {"command": "/missing/helper", "cwd": "/"}
    }))
    .unwrap();
    let Err(error) = resolve_provider_auth(None, &provider) else {
        panic!("unavailable bound credential must stop the request");
    };
    assert!(error.to_string().contains("credential helper"));
}
