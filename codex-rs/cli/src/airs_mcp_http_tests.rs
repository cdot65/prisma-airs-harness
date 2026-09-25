use super::*;
use codex_exec_server::HttpRedirectPolicy;
use pretty_assertions::assert_eq;
use std::collections::VecDeque;

struct RecordingClient {
    requests: Mutex<Vec<HttpRequestParams>>,
    responses: Mutex<VecDeque<Result<HttpRequestResponse, ExecServerError>>>,
}
impl HttpClient for RecordingClient {
    fn http_request(
        &self,
        params: HttpRequestParams,
    ) -> BoxFuture<'_, Result<HttpRequestResponse, ExecServerError>> {
        self.requests.lock().unwrap().push(params);
        Box::pin(futures::future::ready(
            self.responses.lock().unwrap().pop_front().unwrap(),
        ))
    }
    fn http_request_stream(
        &self,
        params: HttpRequestParams,
    ) -> BoxFuture<'_, Result<(HttpRequestResponse, HttpResponseBodyStream), ExecServerError>> {
        self.requests.lock().unwrap().push(params);
        Box::pin(futures::future::ready(Err(ExecServerError::HttpRequest(
            "PRIVATE transport payload".into(),
        ))))
    }
}

#[tokio::test]
async fn outcomes_are_typed_without_rewriting_requests_or_retaining_challenges() {
    let responses = [403, 200, 446, 429, 503, 504].map(|status| {
        Ok(HttpRequestResponse {
            status,
            headers: vec![],
            body: Vec::new().into(),
        })
    });
    let inner = Arc::new(RecordingClient {
        requests: Mutex::new(vec![]),
        responses: Mutex::new(responses.into()),
    });
    let client = McpHttpDiagnostics::new(inner.clone());
    let mut request = HttpRequestParams {
        method: "POST".into(),
        url: "https://gateway.example/mcp".into(),
        headers: vec![],
        body: Some(b"PRIVATE request bytes".to_vec().into()),
        timeout_ms: Some(1500),
        redirect_policy: HttpRedirectPolicy::Stop,
        request_id: "fixture".into(),
        stream_response: false,
    };
    for (status, expected) in [
        (403, Some(FailureCode::PermissionDenied)),
        (200, None),
        (446, Some(FailureCode::PolicyDenied)),
        (429, Some(FailureCode::RateLimited)),
        (503, Some(FailureCode::RemoteUnavailable)),
        (504, Some(FailureCode::TimedOut)),
    ] {
        assert_eq!(
            client.http_request(request.clone()).await.unwrap().status,
            status
        );
        assert_eq!(client.failure(), expected);
        assert_eq!(inner.requests.lock().unwrap().last(), Some(&request));
    }
    request.stream_response = true;
    assert!(client.http_request_stream(request.clone()).await.is_err());
    assert_eq!(client.failure(), Some(FailureCode::Transport));
    assert_eq!(inner.requests.lock().unwrap().last(), Some(&request));
}
