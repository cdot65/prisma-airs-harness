use super::*;
use serde_json::json;

fn provider() -> Provider {
    let keys: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/test-only-jwks.json")).unwrap();
    Provider {
        config: super::super::IdentityConfig { issuer: "https://issuer.example/realm".into(), client_id: "terminal".into(), audience: "inference".into() },
        discovery: serde_json::from_value(json!({"issuer":"https://issuer.example/realm",
            "authorization_endpoint":"https://issuer.example/auth", "token_endpoint":"https://issuer.example/token",
            "jwks_uri":"https://issuer.example/keys", "code_challenge_methods_supported":["S256"]})).unwrap(),
        http: reqwest::Client::new(),
        oidc_keys: serde_json::from_value(keys.clone()).unwrap(),
        access_keys: serde_json::from_value(keys).unwrap(),
    }
}

#[tokio::test]
async fn refresh_rejection_is_distinct_from_a_lost_or_invalid_provider_response() {
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    for (status, body, rejected) in [
        (
            400,
            json!({"error":"invalid_grant", "error_description":"private provider detail"}),
            true,
        ),
        (503, json!({"error":"temporarily_unavailable"}), false),
        (403, json!({"error":"access_denied"}), false),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;
        let mut provider = provider();
        provider.discovery.token_endpoint = format!("{}/token", server.uri()).parse().unwrap();
        let error = provider
            .token_request(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", "fixture-only"),
            ])
            .await
            .err()
            .unwrap();
        assert_eq!(
            matches!(
                error.downcast_ref::<crate::TokenExchangeError>(),
                Some(crate::TokenExchangeError::RefreshRejected)
            ),
            rejected
        );
        assert!(!error.to_string().contains("private provider detail"));
        server.verify().await;
    }
}

fn signed(claims: &serde_json::Value) -> String {
    let key =
        jsonwebtoken::EncodingKey::from_rsa_pem(include_bytes!("fixtures/test-only-private.pem"))
            .unwrap();
    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some("test-only".into());
    jsonwebtoken::encode(&header, claims, &key).unwrap()
}

fn claims() -> (serde_json::Value, serde_json::Value) {
    let now = jsonwebtoken::get_current_timestamp();
    let id = json!({"iss":"https://issuer.example/realm", "aud":"terminal", "sub":"user-123",
        "iat":now, "exp":now+120, "nonce":"nonce-123", "preferred_username":"teammate"});
    let access = json!({"iss":"https://issuer.example/realm", "aud":"inference", "azp":"terminal",
        "sub":"user-123", "iat":now, "exp":now+120});
    (id, access)
}

fn response(id: &serde_json::Value, access: &serde_json::Value) -> CoreTokenResponse {
    serde_json::from_value(json!({"access_token":signed(access), "id_token":signed(id),
        "refresh_token":"test-refresh-token", "token_type":"Bearer", "expires_in":120}))
    .unwrap()
}

#[test]
fn validates_both_tokens_and_retains_original_access_jwt() {
    let (id, access) = claims();
    let tokens = provider()
        .verify(response(&id, &access), NoncePolicy::Browser("nonce-123"))
        .unwrap();
    assert_eq!(
        tokens.identity,
        Identity {
            config: provider().config,
            subject: "user-123".into(),
            display_name: Some("teammate".into())
        }
    );
    assert_eq!(tokens.access_token, signed(&access));
}

#[test]
fn rejects_signed_identity_substitution_and_wrong_resource() {
    let (id, access) = claims();
    for (field, value) in [
        ("iss", json!("https://other.example/realm")),
        ("aud", json!("mcp")),
        ("sub", json!("other-user")),
        ("azp", json!("other-client")),
        ("exp", json!(1)),
        ("iat", json!(jsonwebtoken::get_current_timestamp() + 120)),
    ] {
        let mut altered = access.clone();
        altered[field] = value;
        assert!(
            provider()
                .verify(response(&id, &altered), NoncePolicy::Browser("nonce-123"))
                .is_err()
        );
    }
    for (field, value) in [
        ("iss", json!("https://other.example/realm")),
        ("aud", json!("other-client")),
        ("sub", json!("other-user")),
        ("nonce", json!("wrong-nonce")),
        ("exp", json!(1)),
    ] {
        let mut altered = id.clone();
        altered[field] = value;
        assert!(
            provider()
                .verify(
                    response(&altered, &access),
                    NoncePolicy::Browser("nonce-123")
                )
                .is_err()
        );
    }
}

#[test]
fn verifies_access_token_hash_and_device_nonce_absence() {
    let (mut id, access) = claims();
    id["at_hash"] = json!("incorrect-hash");
    assert!(
        provider()
            .verify(response(&id, &access), NoncePolicy::Browser("nonce-123"))
            .is_err()
    );
    id.as_object_mut().unwrap().remove("at_hash");
    assert!(
        provider()
            .verify(response(&id, &access), NoncePolicy::Device)
            .is_err()
    );
    id.as_object_mut().unwrap().remove("nonce");
    assert!(
        provider()
            .verify(response(&id, &access), NoncePolicy::Device)
            .is_ok()
    );
    assert!(
        provider()
            .verify(response(&id, &access), NoncePolicy::Browser("nonce-123"))
            .is_err()
    );
}

#[test]
fn refresh_allows_missing_nonce_but_rejects_a_changed_nonce() {
    let (mut id, access) = claims();
    id.as_object_mut().unwrap().remove("nonce");
    let tokens = provider()
        .verify(
            response(&id, &access),
            NoncePolicy::Refresh(Some("nonce-123")),
        )
        .unwrap();
    assert_eq!(tokens.nonce.as_deref(), Some("nonce-123"));
    id["nonce"] = json!("wrong-nonce");
    assert!(
        provider()
            .verify(
                response(&id, &access),
                NoncePolicy::Refresh(Some("nonce-123"))
            )
            .is_err()
    );
}
