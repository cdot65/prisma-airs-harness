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
