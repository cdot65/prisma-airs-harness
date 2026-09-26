use super::*;
use codex_http_client::DestinationPolicy;
use codex_http_client::NetworkPolicy;
use codex_http_client::NetworkPolicyController;
use codex_http_client::NetworkPolicyDenied;
use serde_json::json;

fn fixture(endpoint: &str, policy: NetworkPolicy) -> (Provider, crate::Tokens) {
    let keys: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/test-only-jwks.json")).unwrap();
    let config = IdentityConfig {
        issuer: "https://issuer.example".into(),
        client_id: "terminal".into(),
        audience: "gateway".into(),
    };
    let previous = crate::Tokens {
        identity: crate::Identity {
            config: config.clone(),
            subject: "fixture-user".into(),
            display_name: None,
        },
        access_token: "fixture-access".into(),
        refresh_token: "fixture-refresh".into(),
        expires_at: 1,
        nonce: None,
    };
    let provider = Provider {
        config,
        discovery: serde_json::from_value(json!({"issuer":"https://issuer.example",
            "authorization_endpoint":"https://issuer.example/auth", "token_endpoint":endpoint,
            "jwks_uri":"https://issuer.example/keys", "code_challenge_methods_supported":["S256"]}))
        .unwrap(),
        http: reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
        network_policy: policy,
        oidc_keys: serde_json::from_value(keys.clone()).unwrap(),
        access_keys: serde_json::from_value(keys).unwrap(),
    };
    (provider, previous)
}

#[tokio::test]
async fn denied_discovery_and_refresh_do_not_open_a_connection() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!("https://{}", listener.local_addr().unwrap());
    let controller = NetworkPolicyController::default();
    controller.publish(
        controller.policy().revision(),
        DestinationPolicy::Restricted {
            allowed_hosts: Default::default(),
        },
    );
    let error = Provider::discover(
        IdentityConfig {
            issuer: issuer.clone(),
            client_id: "terminal".into(),
            audience: "gateway".into(),
        },
        controller.policy(),
    )
    .await
    .err()
    .unwrap();
    assert_eq!(
        error.downcast_ref::<NetworkPolicyDenied>(),
        Some(&NetworkPolicyDenied::Destination)
    );
    let (provider, previous) = fixture(&format!("{issuer}/token"), controller.policy());
    let error = provider.prepare_refresh(&previous).err().unwrap();
    assert_eq!(
        error.downcast_ref::<NetworkPolicyDenied>(),
        Some(&NetworkPolicyDenied::Destination)
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(50), listener.accept())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn in_flight_refresh_revocation_cancels_without_replaying() {
    use std::sync::Arc;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    let server = MockServer::start().await;
    let seen = Arc::new(tokio::sync::Notify::new());
    let signal = Arc::clone(&seen);
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(move |_: &wiremock::Request| {
            signal.notify_one();
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(30))
                .set_body_json(json!({}))
        })
        .expect(1)
        .mount(&server)
        .await;
    let controller = NetworkPolicyController::default();
    controller.publish(
        controller.policy().revision(),
        DestinationPolicy::Unrestricted,
    );
    let (provider, previous) = fixture(&format!("{}/token", server.uri()), controller.policy());
    let refresh = provider.prepare_refresh(&previous).unwrap();
    let (result, ()) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(refresh.complete(), async {
            seen.notified().await;
            controller.publish(
                controller.policy().revision(),
                DestinationPolicy::Restricted {
                    allowed_hosts: Default::default(),
                },
            );
        })
    })
    .await
    .unwrap();
    assert_eq!(
        result.err().unwrap().downcast_ref::<NetworkPolicyDenied>(),
        Some(&NetworkPolicyDenied::Revoked)
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    server.verify().await;
}

#[tokio::test]
async fn admitted_refresh_cannot_follow_a_redirect() {
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    let origin = MockServer::start().await;
    let destination = MockServer::start().await;
    Mock::given(wiremock::matchers::method("POST"))
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("location", format!("{}/stolen", destination.uri())),
        )
        .expect(1)
        .mount(&origin)
        .await;
    let controller = NetworkPolicyController::default();
    controller.publish(
        controller.policy().revision(),
        DestinationPolicy::Unrestricted,
    );
    let (provider, previous) = fixture(&format!("{}/token", origin.uri()), controller.policy());
    assert!(
        provider
            .prepare_refresh(&previous)
            .unwrap()
            .complete()
            .await
            .is_err()
    );
    assert!(destination.received_requests().await.unwrap().is_empty());
    origin.verify().await;
}

#[tokio::test]
async fn device_polling_stops_when_policy_is_revoked() {
    use std::sync::Arc;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::path;
    let server = MockServer::start().await;
    let seen = Arc::new(tokio::sync::Notify::new());
    let signal = Arc::clone(&seen);
    Mock::given(path("/device"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "device_code":"fixture-device", "user_code":"FIXTURE-CODE", "expires_in":60,
            "interval":1, "verification_uri":format!("{}/verify", server.uri()),
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/token"))
        .respond_with(move |_: &wiremock::Request| {
            signal.notify_one();
            ResponseTemplate::new(400)
                .set_delay(Duration::from_secs(30))
                .set_body_json(json!({"error":"authorization_pending"}))
        })
        .expect(1)
        .mount(&server)
        .await;
    let controller = NetworkPolicyController::default();
    controller.publish(
        controller.policy().revision(),
        DestinationPolicy::Unrestricted,
    );
    let (mut provider, _) = fixture(&format!("{}/token", server.uri()), controller.policy());
    provider.discovery.device_authorization_endpoint =
        Some(format!("{}/device", server.uri()).parse().unwrap());
    let login = provider.device_login().await.unwrap();
    let (result, ()) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(login.complete(&provider), async {
            seen.notified().await;
            controller.publish(
                controller.policy().revision(),
                DestinationPolicy::Restricted {
                    allowed_hosts: Default::default(),
                },
            );
        })
    })
    .await
    .unwrap();
    assert_eq!(
        result.err().unwrap().downcast_ref::<NetworkPolicyDenied>(),
        Some(&NetworkPolicyDenied::Revoked)
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
    server.verify().await;
}
