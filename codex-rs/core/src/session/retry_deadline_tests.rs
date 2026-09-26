//! Event queue backpressure must consume the existing server deadline.

use super::make_session_and_context;
use crate::responses_retry::ResponsesStreamRequest;
use crate::responses_retry::ResponsesStreamRetryState;
use crate::responses_retry::handle_retryable_response_stream_error;
use codex_http_client::RetryAfter;
use codex_protocol::error::CodexErr;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::WarningEvent;
use futures::poll;
use pretty_assertions::assert_eq;
use std::time::Duration;
use tokio::time::Instant;

#[tokio::test]
async fn queued_retry_notification_does_not_restart_server_deadline() {
    let (mut session, context) = make_session_and_context().await;
    let (tx, rx) = async_channel::bounded(1);
    session.tx_event = tx;
    session
        .tx_event
        .send(Event {
            id: "occupied".into(),
            msg: EventMsg::Warning(WarningEvent {
                message: "occupied".into(),
            }),
        })
        .await
        .unwrap();
    let mut client = session.services.model_client.new_session();
    let mut state = ResponsesStreamRetryState::default();
    tokio::time::pause();
    let started = Instant::now();
    let advice = RetryAfter::from_delay(Duration::from_secs(10)).unwrap();
    let pending = handle_retryable_response_stream_error(
        &mut state,
        /*max_retries*/ 1,
        CodexErr::Stream("retry later".into()).with_retry_after(advice),
        &mut client,
        &session,
        &context,
        ResponsesStreamRequest::Sampling,
    );
    tokio::pin!(pending);
    assert!(poll!(pending.as_mut()).is_pending());
    tokio::time::advance(Duration::from_secs(6)).await;
    assert_eq!(rx.recv().await.unwrap().id, "occupied");
    assert!(poll!(pending.as_mut()).is_pending());
    assert!(matches!(
        rx.try_recv().unwrap().msg,
        EventMsg::StreamError(_)
    ));
    tokio::time::advance(Duration::from_secs(4)).await;
    pending.await.unwrap();
    // Permit timer-wheel rounding, while rejecting a restarted delay that would end at 16s.
    let elapsed = Instant::now() - started;
    assert!(
        (Duration::from_secs(10)..=Duration::from_millis(10010)).contains(&elapsed),
        "elapsed {elapsed:?}"
    );
}
