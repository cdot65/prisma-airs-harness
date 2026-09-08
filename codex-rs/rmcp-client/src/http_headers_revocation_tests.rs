use super::*;
use codex_exec_server::RouteAwareHttpClient;
use codex_http_client::HttpClientFactory;
use codex_http_client::OutboundProxyPolicy;
use codex_utils_home_dir::airs_session::AUTH_GENERATION_FILE;
use codex_utils_home_dir::airs_session::AuthGeneration;
use codex_utils_home_dir::airs_session::AuthGenerationState;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::header;
use wiremock::matchers::method;

fn publish(home: &Path, state: AuthGenerationState, nonce: &str) {
    std::fs::write(
        home.join(AUTH_GENERATION_FILE),
        AuthGeneration::new(state, nonce.to_owned())
            .unwrap()
            .encode(),
    )
    .unwrap();
}

fn client(url: &str, home: &Path) -> HttpHeadersClient {
    let provider =
        HttpHeadersProvider::new(url, "unused fixture helper", home.to_path_buf()).unwrap();
    let mut values = HeaderMap::new();
    values.insert(
        "proxy-authorization",
        "Bearer fixture-credential".parse().unwrap(),
    );
    provider.cache.lock().unwrap().current = futures::future::ready(Ok(Arc::new(values)))
        .boxed()
        .shared();
    HttpHeadersClient {
        inner: Arc::new(RouteAwareHttpClient::new(HttpClientFactory::new(
            OutboundProxyPolicy::ReqwestDefault,
        ))),
        provider,
        session: Some(AirsSessionGuard::capture(home).unwrap()),
    }
}

fn request(url: &str) -> HttpRequestParams {
    HttpRequestParams {
        method: "POST".to_owned(),
        url: url.to_owned(),
        headers: Vec::new(),
        body: None,
        timeout_ms: Some(30_000),
        redirect_policy: HttpRedirectPolicy::Stop,
        request_id: "mcp-logout-regression".to_owned(),
        stream_response: false,
    }
}

#[tokio::test]
async fn cached_mcp_headers_never_send_after_logout_or_relogin() {
    let home = TempDir::new().unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(header("proxy-authorization", "Bearer fixture-credential"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;
    let retained = client(&server.uri(), home.path());
    assert_eq!(
        retained
            .http_request(request(&server.uri()))
            .await
            .unwrap()
            .status,
        200
    );
    publish(
        home.path(),
        AuthGenerationState::Revoked,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    assert!(retained.http_request(request(&server.uri())).await.is_err());
    assert!(
        retained
            .http_request_stream(request(&server.uri()))
            .await
            .is_err()
    );
    publish(
        home.path(),
        AuthGenerationState::Active,
        "cccccccccccccccccccccccccccccccc",
    );
    assert!(retained.http_request(request(&server.uri())).await.is_err());
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    assert_eq!(
        client(&server.uri(), home.path())
            .http_request(request(&server.uri()))
            .await
            .unwrap()
            .status,
        200
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
}

#[tokio::test]
async fn pending_mcp_headers_cancel_within_logout_deadline() {
    let home = TempDir::new().unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(30)))
        .mount(&server)
        .await;
    let retained = client(&server.uri(), home.path());
    let params = request(&server.uri());
    let pending = tokio::spawn(async move { retained.http_request_stream(params).await });
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
    assert!(
        tokio::time::timeout(Duration::from_secs(2), pending)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn mcp_authentication_retry_is_blocked_when_logout_races_response() {
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
            ResponseTemplate::new(401)
        })
        .mount(&server)
        .await;
    let retained = client(&server.uri(), home.path());
    assert!(retained.http_request(request(&server.uri())).await.is_err());
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}
