use super::*;
use bytes::Bytes;
use codex_client::TransportError;
use futures::stream;
use http::HeaderMap;
use http::HeaderValue;
use http::StatusCode;
use pretty_assertions::assert_eq;
use serde_json::json;

fn response(body: Vec<u8>, content_type: &str) -> StreamResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_str(content_type).unwrap(),
    );
    StreamResponse {
        status: StatusCode::OK,
        headers,
        bytes: stream::iter([Ok(Bytes::from(body))]).boxed(),
    }
}

#[tokio::test]
async fn explicit_denials_are_terminal_and_redacted_with_the_real_status() {
    for body in [
        json!({"error":{"type":"hooks_failed","message":"PRIVATE-BODY"}}),
        json!({"error":{"code":"hooks_failed","message":"PRIVATE-BODY"}}),
        json!({"hook_results":{"before_request_hooks":[{"verdict":false,"softDeny200":true}]}}),
        json!({"hook_results":{"after_request_hooks":[{"verdict":false,"deny":true}]}}),
        json!({"hook_results":{"before_request_hooks":[{"verdict":false,"soft_deny_200":true}]}}),
    ] {
        let mut response = response(
            serde_json::to_vec(&body).unwrap(),
            "Application/JSON; charset=utf-8",
        );
        response.headers.insert(
            "x-portkey-trace-id",
            HeaderValue::from_static("00000000-0000-0000-0000-000000000000"),
        );
        response
            .headers
            .insert("retry-after", HeaderValue::from_static("3600"));
        let error = reject_json(&mut response, Duration::from_secs(10))
            .await
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            format!(
                "invalid request: {POLICY_DENIED} HTTP 200. Request / gateway trace ID: 00000000-0000-0000-0000-000000000000."
            )
        );
        assert!(!crate::api_bridge::map_api_error(error).is_retryable());
    }
}

#[tokio::test]
async fn malformed_unrelated_or_nonblocking_json_is_not_mislabeled_as_policy() {
    for body in [
        b"PRIVATE-MALFORMED".to_vec(),
        serde_json::to_vec(&json!({"output":"hooks_failed PRIVATE-OUTPUT"})).unwrap(),
        serde_json::to_vec(
            &json!({"hook_results":{"before_request_hooks":[{"verdict":false,"deny":false}]}}),
        )
        .unwrap(),
        serde_json::to_vec(
            &json!({"hook_results":{"before_request_hooks":[{"verdict":true,"deny":true}]}}),
        )
        .unwrap(),
        vec![b'x'; MAX_JSON_BYTES + 1],
    ] {
        let mut response = response(body, "application/json");
        response.headers.insert(
            "x-portkey-trace-id",
            HeaderValue::from_static("PRIVATE-TRACE"),
        );
        response
            .headers
            .insert("retry-after", HeaderValue::from_static("3600"));
        let error = reject_json(&mut response, Duration::from_secs(10))
            .await
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid request: The server returned JSON instead of the requested event stream. Check its route and policy configuration before explicitly retrying. HTTP 200."
        );
        assert!(!crate::api_bridge::map_api_error(error).is_retryable());
    }
}

#[tokio::test]
async fn stalled_json_body_is_bounded_and_does_not_become_a_retry() {
    let mut response = response(Vec::new(), "application/problem+json");
    response.bytes = stream::pending::<Result<Bytes, TransportError>>().boxed();
    let error = tokio::time::timeout(
        Duration::from_secs(1),
        reject_json(&mut response, Duration::from_millis(1)),
    )
    .await
    .unwrap()
    .unwrap_err();
    assert!(!crate::api_bridge::map_api_error(error).is_retryable());
}

#[tokio::test]
async fn event_stream_bytes_are_not_consumed_by_json_inspection() {
    let bytes = b"data: {\"type\":\"response.completed\"}\n\n";
    let mut response = response(bytes.to_vec(), "text/event-stream");
    reject_json(&mut response, Duration::from_secs(1))
        .await
        .unwrap();
    assert_eq!(
        response.bytes.next().await.unwrap().unwrap(),
        Bytes::from_static(bytes)
    );
}

#[test]
fn streamed_failed_and_completed_policy_envelopes_are_terminal() {
    for kind in ["response.failed", "response.completed"] {
        let event = serde_json::from_value(json!({"type":kind,"response":{"error":{"type":"hooks_failed","message":"PRIVATE-BODY"}}})).unwrap();
        let error = crate::sse::process_responses_event(event).unwrap_err();
        let error = error.into_api_error();
        assert_eq!(
            error.to_string(),
            format!("invalid request: {POLICY_DENIED}")
        );
        assert!(!crate::api_bridge::map_api_error(error).is_retryable());
    }
}

#[tokio::test]
async fn streamed_denial_cannot_be_overwritten_by_later_success_or_stall() {
    for trailing in [
        "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"later\",\"usage\":null}}\n\n",
        "",
    ] {
        let data = format!(
            "data: {{\"type\":\"response.failed\",\"response\":{{\"error\":{{\"type\":\"hooks_failed\"}}}}}}\n\n{trailing}"
        );
        let mut response = response(data.into_bytes(), "text/event-stream");
        response.bytes = response.bytes.chain(stream::pending()).boxed();
        let mut events =
            crate::sse::spawn_response_stream(response, Duration::from_secs(10), None, None);
        let error = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if let Some(Err(error)) = events.next().await {
                    break error;
                }
            }
        })
        .await
        .expect("denial must terminate without waiting for stream closure");
        assert_eq!(
            error.to_string(),
            format!("invalid request: {POLICY_DENIED}")
        );
        assert!(events.next().await.is_none());
    }
}
