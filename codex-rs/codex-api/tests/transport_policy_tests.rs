//! Terminal denials must survive retry orchestration even when every retry option is enabled.

use codex_client::Request;
use codex_client::RetryOn;
use codex_client::RetryPolicy;
use codex_client::TransportError;
use codex_client::run_with_retry;
use codex_http_client::NetworkPolicyDenied;
use http::Method;
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

#[tokio::test]
async fn application_policy_denials_are_returned_without_replaying_requests() {
    for denied in [
        NetworkPolicyDenied::Unavailable,
        NetworkPolicyDenied::Destination,
        NetworkPolicyDenied::Revoked,
        NetworkPolicyDenied::UnsupportedTransport,
    ] {
        let prepared = AtomicUsize::new(0);
        let sent = AtomicUsize::new(0);
        let result: Result<(), TransportError> = run_with_retry(
            RetryPolicy {
                max_attempts: 3,
                base_delay: Duration::ZERO,
                retry_on: RetryOn {
                    retry_429: true,
                    retry_5xx: true,
                    retry_transport: true,
                },
            },
            || {
                prepared.fetch_add(1, Ordering::SeqCst);
                Request::new(Method::POST, "https://gateway.example/v1/responses".into())
            },
            |_request, _attempt| {
                sent.fetch_add(1, Ordering::SeqCst);
                std::future::ready(Err(TransportError::Policy(denied)))
            },
        )
        .await;
        let Err(TransportError::Policy(actual)) = result else {
            panic!("the original policy denial must remain typed");
        };
        assert_eq!(actual, denied);
        assert_eq!((prepared.into_inner(), sent.into_inner()), (1, 1));
    }
}
