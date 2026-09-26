use super::*;
use codex_api::ApiError;
use codex_http_client::NetworkPolicyDenied;
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[derive(Clone)]
struct RevokedStream(Arc<AtomicUsize>);

impl HttpTransport for RevokedStream {
    async fn execute(&self, _request: Request) -> Result<Response, TransportError> {
        panic!("a streaming response must not use buffered execution");
    }

    async fn stream(&self, _request: Request) -> Result<StreamResponse, TransportError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(StreamResponse {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            bytes: Box::pin(futures::stream::iter([
                Ok(Bytes::from(build_responses_body(vec![serde_json::json!({
                    "type": "response.output_text.delta", "delta": "received text"
                })]))),
                Err(TransportError::Policy(NetworkPolicyDenied::Revoked)),
            ])),
        })
    }
}

#[tokio::test]
async fn partial_sse_preserves_policy_denial_and_does_not_replay() -> Result<()> {
    let attempts = Arc::new(AtomicUsize::new(0));
    let mut provider = provider("gateway");
    provider.retry.max_attempts = 3;
    let client = ResponsesClient::new(
        RevokedStream(Arc::clone(&attempts)),
        provider,
        Arc::new(NoAuth),
    );
    let mut stream = client
        .stream(
            serde_json::json!({"input": "fixture"}),
            HeaderMap::new(),
            Compression::None,
            /*turn_state*/ None,
        )
        .await?;
    let mut text = String::new();
    let mut denial = None;
    while let Some(event) = stream.next().await {
        match event {
            Ok(ResponseEvent::OutputTextDelta(delta)) => text.push_str(&delta),
            Ok(ResponseEvent::RateLimits(_)) => {}
            Err(ApiError::Transport(TransportError::Policy(error))) => {
                assert!(denial.replace(error).is_none());
            }
            other => panic!("unexpected stream event: {other:?}"),
        }
    }
    assert_eq!(
        (text.as_str(), denial, attempts.load(Ordering::SeqCst)),
        ("received text", Some(NetworkPolicyDenied::Revoked), 1)
    );
    Ok(())
}
