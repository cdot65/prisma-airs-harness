//! Request retry advice never grants retry eligibility or restarts a deadline.

use codex_client::RetryOn;
use codex_client::RetryPolicy;
use codex_client::run_with_retry;
use codex_http_client::Request;
use codex_http_client::RetryAfter;
use codex_http_client::TransportError;
use http::Method;
use http::StatusCode;
use pretty_assertions::assert_eq;
use std::sync::Mutex;
use std::time::Duration;
use tokio::time::Instant;

fn policy() -> RetryPolicy {
    RetryPolicy {
        max_attempts: 2,
        base_delay: Duration::from_secs(30),
        retry_on: RetryOn {
            retry_429: true,
            retry_5xx: true,
            retry_transport: true,
        },
    }
}

fn request() -> Request {
    Request::new(
        Method::POST,
        "https://gateway.example.test/v1/responses".into(),
    )
}

fn response(status: StatusCode, advice: RetryAfter) -> TransportError {
    TransportError::Http {
        status,
        url: None,
        headers: None,
        body: None,
        retry_after: Some(advice),
    }
}

#[tokio::test(start_paused = true)]
async fn http_deadline_survives_processing_and_exhausted_attempts() {
    let start = Instant::now();
    let advice = RetryAfter::from_delay(Duration::from_secs(10)).unwrap();
    let attempts = Mutex::new(Vec::new());
    let result = run_with_retry(policy(), request, |_, attempt| {
        attempts
            .lock()
            .unwrap()
            .push((attempt, Instant::now() - start));
        async move {
            // Processing consumes the original advice, including when it has expired.
            tokio::time::sleep(Duration::from_secs(3)).await;
            Err::<(), _>(response(StatusCode::SERVICE_UNAVAILABLE, advice))
        }
    })
    .await
    .unwrap_err();
    assert_eq!(
        attempts.into_inner().unwrap(),
        vec![
            (0, Duration::ZERO),
            (1, Duration::from_secs(10)),
            (2, Duration::from_secs(13)),
        ]
    );
    assert_eq!(result.retry_after(), Some(advice));
    assert_eq!(
        result.retry_after().unwrap().remaining_delay(),
        Duration::ZERO
    );
}

#[tokio::test(start_paused = true)]
async fn retry_advice_does_not_make_auth_or_policy_statuses_retryable() {
    for status in [401, 403, 446, 200] {
        let advice = RetryAfter::from_delay(Duration::from_secs(300)).unwrap();
        let start = Instant::now();
        let attempts = Mutex::new(Vec::new());
        let error = run_with_retry(policy(), request, |_, attempt| {
            attempts.lock().unwrap().push(attempt);
            std::future::ready(Err::<(), _>(response(
                StatusCode::from_u16(status).unwrap(),
                advice,
            )))
        })
        .await
        .unwrap_err();
        assert_eq!(attempts.into_inner().unwrap(), vec![0]);
        assert_eq!(Instant::now(), start);
        assert_eq!(error.retry_after(), Some(advice));
    }
}

#[tokio::test(start_paused = true)]
async fn disabled_status_retries_ignore_even_valid_advice() {
    let mut policy = policy();
    policy.retry_on.retry_429 = false;
    let attempts = Mutex::new(Vec::new());
    let advice = RetryAfter::from_delay(Duration::ZERO).unwrap();
    let error = run_with_retry(policy, request, |_, attempt| {
        attempts.lock().unwrap().push(attempt);
        std::future::ready(Err::<(), _>(response(
            StatusCode::TOO_MANY_REQUESTS,
            advice,
        )))
    })
    .await
    .unwrap_err();
    assert_eq!(attempts.into_inner().unwrap(), vec![0]);
    assert_eq!(error.retry_after(), Some(advice));
}

#[tokio::test(start_paused = true)]
async fn cancelling_a_long_retry_wait_sends_no_followup() {
    let advice = RetryAfter::from_delay(Duration::from_secs(3600)).unwrap();
    let attempts = Mutex::new(Vec::new());
    let retry = run_with_retry(policy(), request, |_, attempt| {
        attempts.lock().unwrap().push(attempt);
        std::future::ready(Err::<(), _>(response(
            StatusCode::SERVICE_UNAVAILABLE,
            advice,
        )))
    });
    assert!(
        tokio::time::timeout(Duration::from_secs(1), retry)
            .await
            .is_err()
    );
    tokio::time::advance(Duration::from_secs(3600)).await;
    assert_eq!(attempts.into_inner().unwrap(), vec![0]);
}

#[tokio::test(start_paused = true)]
async fn exhausted_http_error_mapping_does_not_restart_deadline() {
    let advice = RetryAfter::from_delay(Duration::from_secs(10)).unwrap();
    let error = run_with_retry(policy(), request, |_, _| {
        std::future::ready(Err::<(), _>(response(
            StatusCode::INTERNAL_SERVER_ERROR,
            advice,
        )))
    })
    .await
    .unwrap_err();
    tokio::time::advance(Duration::from_secs(2)).await;
    let error = codex_api::map_api_error(error.into());
    assert_eq!(
        (
            error.retry_after(),
            error.retry_delay(),
            error.is_retryable()
        ),
        (Some(advice), Some(Duration::ZERO), true)
    );
}

#[tokio::test]
async fn interrupt_cancels_a_turn_waiting_on_stream_retry_advice() -> anyhow::Result<()> {
    use codex_protocol::protocol::EventMsg;
    use codex_protocol::protocol::Op;
    use codex_protocol::turn_input::TurnInputRequest;
    use codex_protocol::user_input::UserInput;
    use core_test_support::responses;
    use core_test_support::test_codex::test_codex;
    use core_test_support::wait_for_event;
    use serde_json::json;
    let server = responses::start_mock_server().await;
    let requests = responses::mount_sse_once(
        &server,
        responses::sse(vec![json!({
            "type": "response.failed", "response": { "error": {
                "code": "rate_limit_exceeded", "message": "Please try again in 3600s."
            }}
        })]),
    )
    .await;
    let test = test_codex()
        .with_config(|config| {
            config.model_provider.request_max_retries = Some(0);
            config.model_provider.stream_max_retries = Some(1);
        })
        .build_with_auto_env(&server)
        .await?;
    test.codex
        .start_or_steer_turn(TurnInputRequest::user_input(vec![UserInput::Text {
            text: "test interrupt during backoff".into(),
            text_elements: vec![],
        }]))
        .await?;
    wait_for_event(&test.codex, |event| {
        matches!(event, EventMsg::StreamError(_))
    })
    .await;
    test.codex.submit(Op::Interrupt).await?;
    tokio::time::timeout(
        Duration::from_secs(10),
        wait_for_event(&test.codex, |event| {
            matches!(event, EventMsg::TurnAborted(_))
        }),
    )
    .await
    .expect("interrupt must not wait for an hour-long server deadline");
    assert_eq!(requests.requests().len(), 1);
    Ok(())
}

#[tokio::test]
async fn websocket_fallback_waits_for_the_same_gateway_deadline() -> anyhow::Result<()> {
    use core_test_support::responses;
    use core_test_support::test_codex::test_codex;
    use std::sync::Arc;
    use wiremock::Mock;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::method;
    let server = responses::start_mock_server().await;
    let received = Arc::new(Mutex::new(Vec::new()));
    let websocket_received = received.clone();
    Mock::given(method("GET"))
        .respond_with(move |_: &wiremock::Request| {
            websocket_received
                .lock()
                .unwrap()
                .push(("GET", std::time::Instant::now()));
            ResponseTemplate::new(503).insert_header("retry-after", "1")
        })
        .mount(&server)
        .await;
    let http_received = received.clone();
    Mock::given(method("POST"))
        .respond_with(move |_: &wiremock::Request| {
            http_received
                .lock()
                .unwrap()
                .push(("POST", std::time::Instant::now()));
            responses::sse_response(responses::sse(vec![
                responses::ev_response_created("fallback"),
                responses::ev_completed("fallback"),
            ]))
        })
        .mount(&server)
        .await;
    let test = test_codex()
        .with_config(|config| {
            config.model_provider.supports_websockets = true;
            config.model_provider.request_max_retries = Some(0);
            config.model_provider.stream_max_retries = Some(0);
        })
        .build(&server)
        .await?;
    test.submit_turn("respect advice when falling back to HTTP")
        .await?;
    let received = received.lock().unwrap();
    let http = received
        .iter()
        .filter(|(method, _)| *method == "POST")
        .collect::<Vec<_>>();
    assert_eq!(http.len(), 1);
    let last_websocket = received
        .iter()
        .rev()
        .find(|(method, _)| *method == "GET")
        .unwrap();
    assert!(http[0].1.duration_since(last_websocket.1) >= Duration::from_secs(1));
    Ok(())
}
