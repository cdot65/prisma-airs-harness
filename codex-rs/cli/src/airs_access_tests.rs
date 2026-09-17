use super::*;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;

fn config(url: &str, selected: &str) -> (toml::Value, Value) {
    let config = serde_json::from_value(serde_json::json!({
        "model": selected,
        "model_providers": { "airs": {
            "name": "Prisma AIRS AI Gateway", "base_url": url,
            "wire_api": "responses", "requires_openai_auth": false,
            "supports_websockets": false,
            "gateway": {"default_route": "airs-gateway-default"}
        }}
    }))
    .unwrap();
    let catalog = serde_json::json!({"models":[
        {"slug":"airs-gateway-default"}, {"slug":"@fixture/chat"}
    ]});
    (config, catalog)
}

fn prepared(home: &Path, url: &str, selected: &str) -> Prepared {
    let (config, catalog) = config(url, selected);
    let (endpoint, body) = probe_configuration(&config, &catalog).unwrap();
    let mut credential = HeaderValue::from_static("Bearer fixture-secret");
    credential.set_sensitive(true);
    Prepared {
        endpoint,
        body,
        credential,
        header_name: "authorization",
        session: AirsSessionGuard::capture(home).unwrap(),
    }
}

fn success() -> Value {
    serde_json::json!({"object":"response", "status":"completed", "output":[{"type":"message", "content":[{"type":"output_text","text":"OK"}]}]})
}

#[tokio::test]
async fn default_and_catalog_routes_send_exact_bounded_probe() {
    for selected in ["airs-gateway-default", "@fixture/chat"] {
        let home = TempDir::new().unwrap();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(success()))
            .mount(&server)
            .await;
        let request_id = Uuid::new_v4();
        assert_eq!(
            probe(prepared(home.path(), &server.uri(), selected), request_id).await,
            Ok(())
        );
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let request = &requests[0];
        assert_eq!(request.url.path(), "/responses");
        assert_eq!(request.headers["authorization"], "Bearer fixture-secret");
        assert_eq!(
            request.headers["x-client-request-id"],
            request_id.to_string()
        );
        let mut expected = serde_json::json!({
            "input":"Reply only with OK. This is a Prisma AIRS Harness connectivity check.",
            "max_output_tokens":16,"store":false,"stream":false
        });
        if selected.starts_with('@') {
            expected["model"] = Value::String(selected.to_owned());
        }
        assert_eq!(request.body_json::<Value>().unwrap(), expected);
    }
}

#[tokio::test]
async fn reasoning_only_probe_verifies_access_without_requiring_final_text() {
    for status in ["completed", "incomplete"] {
        let home = TempDir::new().unwrap();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "object": "response", "status": status, "error": null,
                "output": [{"type": "reasoning", "id": "rs_fixture", "summary": []}],
                "usage": {"output_tokens": 16}
            })))
            .mount(&server)
            .await;
        assert_eq!(
            probe(
                prepared(home.path(), &server.uri(), "airs-gateway-default"),
                Uuid::new_v4()
            )
            .await,
            Ok(())
        );
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].body_json::<Value>().unwrap()["max_output_tokens"],
            16
        );
    }
}

#[test]
fn unlisted_routes_and_unsafe_destinations_fail_before_credential_resolution() {
    for (url, selected) in [
        ("https://gateway.example/v1", "@unlisted/model"),
        ("https://gateway.example/v1", "gpt-4.1"),
        (
            "https://name:secret@gateway.example/v1",
            "airs-gateway-default",
        ),
        (
            "https://gateway.example/v1?secret=value",
            "airs-gateway-default",
        ),
        ("http://gateway.example/v1", "airs-gateway-default"),
    ] {
        let (config, catalog) = config(url, selected);
        assert_eq!(
            probe_configuration(&config, &catalog),
            Err(Failure::Configuration)
        );
    }
}

#[tokio::test]
async fn denial_offline_and_soft_denial_are_not_verified() {
    let home = TempDir::new().unwrap();
    for (status, body, expected) in [
        (
            401,
            serde_json::json!({"error":"fixture-secret"}),
            Failure::Denied,
        ),
        (
            403,
            serde_json::json!({"error":"fixture-secret"}),
            Failure::Denied,
        ),
        (
            503,
            serde_json::json!({"error":"fixture-secret"}),
            Failure::Rejected,
        ),
        (
            200,
            serde_json::json!({"error":"fixture-secret"}),
            Failure::InvalidResponse,
        ),
        (
            200,
            serde_json::json!({"object":"response","status":"failed","output":[]}),
            Failure::InvalidResponse,
        ),
        (
            200,
            serde_json::json!({"object":"response","status":"completed","output":[]}),
            Failure::InvalidResponse,
        ),
        (
            200,
            serde_json::json!({"object":"response","status":"failed","output":[{"type":"reasoning"}]}),
            Failure::InvalidResponse,
        ),
        (
            200,
            serde_json::json!({"object":"response","status":"completed","error":{"code":"denied"},"output":[{"type":"reasoning"}]}),
            Failure::InvalidResponse,
        ),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
            .mount(&server)
            .await;
        let result = probe(
            prepared(home.path(), &server.uri(), "airs-gateway-default"),
            Uuid::new_v4(),
        )
        .await;
        assert_eq!(result, Err(expected));
        assert!(
            !Verification {
                request_id: Uuid::nil(),
                outcome: result
            }
            .after_login()
            .contains("fixture-secret")
        );
    }
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let offline = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    assert_eq!(
        probe(
            prepared(home.path(), &offline, "airs-gateway-default"),
            Uuid::new_v4()
        )
        .await,
        Err(Failure::Offline)
    );
}

#[tokio::test]
async fn redirects_never_forward_credentials_or_send_second_request() {
    let home = TempDir::new().unwrap();
    let destination = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(success()))
        .mount(&destination)
        .await;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(307).insert_header("location", destination.uri()))
        .mount(&server)
        .await;
    assert_eq!(
        probe(
            prepared(home.path(), &server.uri(), "airs-gateway-default"),
            Uuid::new_v4()
        )
        .await,
        Err(Failure::Redirect)
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    assert!(destination.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn oversized_response_fails_without_echoing_body() {
    let home = TempDir::new().unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("x".repeat(MAX_RESPONSE_BYTES + 1)),
        )
        .mount(&server)
        .await;
    assert_eq!(
        probe(
            prepared(home.path(), &server.uri(), "airs-gateway-default"),
            Uuid::new_v4()
        )
        .await,
        Err(Failure::OversizedResponse)
    );
}

#[tokio::test]
async fn logout_before_send_never_uses_resolved_credential() {
    let home = TempDir::new().unwrap();
    let server = MockServer::start().await;
    let ready = prepared(home.path(), &server.uri(), "airs-gateway-default");
    std::fs::write(home.path().join("logged-out"), "signed out").unwrap();
    assert_eq!(probe(ready, Uuid::new_v4()).await, Err(Failure::SignedOut));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[cfg(unix)]
#[tokio::test]
async fn helper_output_is_bounded_and_failed_helpers_never_disclose_tokens() {
    for script in [
        "printf 'fixture-secret'; exit 1",
        "printf 'not a token'",
        "head -c 17000 /dev/zero",
    ] {
        let mut command = Command::new("sh");
        command.arg("-c").arg(script);
        assert_eq!(helper_token(command).await, Err(Failure::Credential));
    }
    let mut command = Command::new("sh");
    command.arg("-c").arg("printf 'fixture-secret\\n'");
    assert_eq!(helper_token(command).await.unwrap(), "fixture-secret");
    let mut command = Command::new("sleep");
    command.arg("60");
    assert!(
        tokio::time::timeout(Duration::from_millis(50), helper_token(command))
            .await
            .is_err()
    );
}

#[test]
fn honest_verification_messages_have_snapshot_coverage() {
    let verified = Verification {
        request_id: Uuid::nil(),
        outcome: Ok(()),
    };
    let offline = Verification {
        request_id: Uuid::nil(),
        outcome: Err(Failure::Offline),
    };
    insta::assert_snapshot!(format!("{DISCLOSURE}\n\n{}\n\n{}", verified.after_login(), offline.after_login()), @r"
    Checking gateway access with one minimal inference request (up to 16 output tokens). This sends only a fixed connectivity message, with no local files or tools. The request asks the provider not to store the response; gateway logging policy still applies.

    Gateway access verified by one inference response. MCP permissions were not tested.
    Client correlation ID: 00000000-0000-0000-0000-000000000000

    Credential saved; gateway access not yet verified. The gateway connection failed. Check connectivity, DNS and TLS.
    Client correlation ID: 00000000-0000-0000-0000-000000000000
    Retry: airs doctor --verify-access (select the same environment).
    ");
}

#[test]
fn workspace_credential_limit_is_preserved_for_probe_headers() {
    let largest = "a".repeat(16_384);
    let with_line_ending = format!("{largest}\r\n");
    assert_eq!(validate_token(&with_line_ending).unwrap(), largest);
    assert_eq!(
        validate_token(&"a".repeat(16_385)),
        Err(Failure::Credential)
    );
    assert_eq!(
        validate_token("token\r\nheader: injected"),
        Err(Failure::Credential)
    );
}
