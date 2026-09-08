use super::*;
use crate::auth::AuthProvider;
use crate::provider::RetryConfig;
use codex_client::ReqwestTransport;
use codex_http_client::HttpClientBuilder;
use codex_utils_home_dir::airs_session::AUTH_GENERATION_FILE;
use codex_utils_home_dir::airs_session::AirsSessionGuard;
use codex_utils_home_dir::airs_session::AuthGeneration;
use codex_utils_home_dir::airs_session::AuthGenerationState;
use futures::StreamExt;
use pretty_assertions::assert_eq;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;

struct RetainedAuth;
impl AuthProvider for RetainedAuth {
    fn add_auth_headers(&self, headers: &mut HeaderMap) {
        headers.insert(
            "authorization",
            "Bearer fixture-credential".parse().unwrap(),
        );
    }
}

fn publish(home: &Path, state: AuthGenerationState, nonce: &str) {
    std::fs::write(
        home.join(AUTH_GENERATION_FILE),
        AuthGeneration::new(state, nonce.to_owned())
            .unwrap()
            .encode(),
    )
    .unwrap();
}

fn client(url: String, home: &Path) -> EndpointSession<ReqwestTransport> {
    let transport =
        ReqwestTransport::from_http_client(HttpClientBuilder::new().build_direct().unwrap());
    let provider = Provider {
        name: "logout-regression".to_owned(),
        base_url: url,
        query_params: None,
        headers: HeaderMap::new(),
        retry: RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            retry_429: true,
            retry_5xx: true,
            retry_transport: true,
        },
        stream_idle_timeout: Duration::from_secs(10),
    };
    let mut client = EndpointSession::new(transport, provider, Arc::new(RetainedAuth));
    client.guard.session = Ok(Some(AirsSessionGuard::capture(home).unwrap()));
    client
}

async fn send(client: &EndpointSession<ReqwestTransport>) -> Result<Response, ApiError> {
    client
        .execute(Method::POST, "responses", HeaderMap::new(), None)
        .await
}

#[tokio::test]
async fn retained_inference_client_stops_sending_after_logout_and_relogin() {
    let home = TempDir::new().unwrap();
    publish(
        home.path(),
        AuthGenerationState::Active,
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;
    let retained = client(server.uri(), home.path());
    assert!(send(&retained).await.is_ok());
    // The remote endpoint continues accepting the cached credential throughout this test.
    publish(
        home.path(),
        AuthGenerationState::Revoked,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    assert!(send(&retained).await.is_err());
    assert!(
        retained
            .stream_encoded_json_with(Method::POST, "responses", HeaderMap::new(), None, |_| {})
            .await
            .is_err()
    );
    publish(
        home.path(),
        AuthGenerationState::Active,
        "cccccccccccccccccccccccccccccccc",
    );
    assert!(send(&retained).await.is_err());
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    assert!(send(&client(server.uri(), home.path())).await.is_ok());
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
}

#[tokio::test]
async fn pending_inference_headers_cancel_within_logout_deadline() {
    let home = TempDir::new().unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(30)))
        .mount(&server)
        .await;
    let retained = client(server.uri(), home.path());
    let pending = tokio::spawn(async move { send(&retained).await });
    tokio::time::timeout(Duration::from_secs(5), async {
        while server.received_requests().await.unwrap().is_empty() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    publish(
        home.path(),
        AuthGenerationState::Revoked,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    let outcome = tokio::time::timeout(Duration::from_secs(2), pending)
        .await
        .unwrap()
        .unwrap();
    assert!(outcome.is_err());
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn previously_opened_inference_stream_stops_delivering_after_logout() {
    let home = TempDir::new().unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("data: payload\n\n"))
        .mount(&server)
        .await;
    let retained = client(server.uri(), home.path());
    let mut response = retained
        .stream_encoded_json_with(Method::POST, "responses", HeaderMap::new(), None, |_| {})
        .await
        .unwrap();
    publish(
        home.path(),
        AuthGenerationState::Revoked,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    assert!(response.bytes.next().await.unwrap().is_err());
    assert!(response.bytes.next().await.is_none());
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn retryable_remote_failure_cannot_retry_after_logout() {
    let home = TempDir::new().unwrap();
    let path = home.path().to_path_buf();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(move |_: &wiremock::Request| {
            publish(
                &path,
                AuthGenerationState::Revoked,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            );
            ResponseTemplate::new(503)
        })
        .mount(&server)
        .await;
    assert!(send(&client(server.uri(), home.path())).await.is_err());
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}
