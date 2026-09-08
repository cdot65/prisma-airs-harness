use super::*;
use codex_exec_server::HttpHeader;
use codex_exec_server::HttpRedirectPolicy;
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

#[tokio::test]
async fn static_header_mcp_and_oauth_requests_stop_after_logout() {
    let home = TempDir::new().unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(header("authorization", "Bearer fixture-static-token"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;
    let retained: Arc<dyn HttpClient> = Arc::new(AirsHttpClient {
        inner: Arc::new(RouteAwareHttpClient::new(HttpClientFactory::new(
            OutboundProxyPolicy::ReqwestDefault,
        ))),
        session: Ok(Some(AirsSessionGuard::capture(home.path()).unwrap())),
    });
    let request = HttpRequestParams {
        method: "POST".to_owned(),
        url: server.uri(),
        headers: vec![HttpHeader {
            name: "authorization".to_owned(),
            value: "Bearer fixture-static-token".to_owned(),
            value_env_var: None,
        }],
        body: None,
        timeout_ms: Some(5_000),
        redirect_policy: HttpRedirectPolicy::Stop,
        request_id: "mcp-static-token".to_owned(),
        stream_response: false,
    };
    assert_eq!(
        retained.http_request(request.clone()).await.unwrap().status,
        200
    );
    let state = AuthGeneration::new(
        AuthGenerationState::Revoked,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
    )
    .unwrap();
    std::fs::write(home.path().join(AUTH_GENERATION_FILE), state.encode()).unwrap();
    assert!(retained.http_request(request.clone()).await.is_err());
    assert!(retained.http_request_stream(request.clone()).await.is_err());
    let state = AuthGeneration::new(
        AuthGenerationState::Active,
        "cccccccccccccccccccccccccccccccc".to_owned(),
    )
    .unwrap();
    std::fs::write(home.path().join(AUTH_GENERATION_FILE), state.encode()).unwrap();
    let mut refresh = request;
    refresh.request_id = "oauth-request-refresh".to_owned();
    assert!(retained.http_request(refresh).await.is_err());
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}
