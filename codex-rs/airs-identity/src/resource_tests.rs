use super::*;
use serde_json::json;

#[test]
fn discovery_rejects_resource_issuer_and_scope_substitution() {
    let config = IdentityConfig {
        issuer: "https://issuer.example/realm".into(),
        client_id: "mcp-native".into(),
        audience: "https://mcp.example/mcp".into(),
        resource: Some("https://mcp.example/mcp".into()),
        scopes: vec!["airs.gateway.read".into()],
    };
    let metadata = json!({"resource": config.resource, "authorization_servers": [config.issuer], "scopes_supported": config.scopes});
    validate(&metadata, &config).unwrap();
    for (key, value) in [
        ("resource", json!("https://other.example/mcp")),
        (
            "authorization_servers",
            json!(["https://other.example/realm"]),
        ),
        ("scopes_supported", json!([])),
    ] {
        let mut changed = metadata.clone();
        changed[key] = value;
        assert!(validate(&changed, &config).is_err());
    }
}

#[test]
fn legacy_binding_serialization_remains_stable() {
    let old = json!({"issuer":"https://issuer.example/realm", "client_id":"native", "audience":"inference"});
    let config: IdentityConfig = serde_json::from_value(old.clone()).unwrap();
    assert_eq!(serde_json::to_value(config).unwrap(), old);
}

#[tokio::test]
async fn browser_and_token_exchange_send_resource_and_requested_scopes() {
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::body_string_contains;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    let mock = MockServer::start().await;
    let provider = crate::Provider {
        config: IdentityConfig {
            issuer: "https://issuer.example/realm".into(), client_id: "mcp-native".into(),
            audience: "https://mcp.example/mcp".into(), resource: Some("https://mcp.example/mcp".into()),
            scopes: vec!["airs.gateway.read".into(), "airs.profiles.read".into()],
        },
        discovery: serde_json::from_value(json!({"issuer":"https://issuer.example/realm",
            "authorization_endpoint":"https://issuer.example/auth", "token_endpoint":format!("{}/token", mock.uri()),
            "jwks_uri":"https://issuer.example/keys", "code_challenge_methods_supported":["S256"]})).unwrap(),
        http: reqwest::Client::new(),
        oidc_keys: serde_json::from_value(json!({"keys":[]})).unwrap(),
        access_keys: serde_json::from_value(json!({"keys":[]})).unwrap(),
    };
    let login = provider.browser_login().await.unwrap();
    let query: std::collections::HashMap<_, _> = login.authorization_url().query_pairs().collect();
    assert_eq!(query["resource"], "https://mcp.example/mcp");
    assert_eq!(
        query["scope"],
        "openid airs.gateway.read airs.profiles.read"
    );
    assert_eq!(query["code_challenge_method"], "S256");
    for grant in ["authorization_code", "refresh_token"] {
        Mock::given(method("POST"))
            .and(path("/token"))
            .and(body_string_contains(format!("grant_type={grant}")))
            .and(body_string_contains(
                "resource=https%3A%2F%2Fmcp.example%2Fmcp",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                json!({"access_token":"fixture-token", "token_type":"Bearer", "expires_in":300}),
            ))
            .expect(1)
            .mount(&mock)
            .await;
        provider
            .token_request(&[("client_id", "mcp-native"), ("grant_type", grant)])
            .await
            .unwrap();
    }
    assert!(provider.device_login().await.is_err());
}
