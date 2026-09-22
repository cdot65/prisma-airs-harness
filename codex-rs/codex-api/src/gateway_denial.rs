//! Keep explicit gateway denials out of transport retry recovery.
use crate::error::ApiError;
use codex_client::StreamResponse;
use futures::StreamExt;
use serde_json::Value;
use std::time::Duration;

const MAX_JSON_BYTES: usize = 64 * 1024;
pub(crate) const POLICY_DENIED: &str = "A gateway guardrail blocked this request (policy denial). Check the gateway trace before explicitly retrying.";

pub(crate) fn is_denied(body: &Value) -> bool {
    body.get("error").is_some_and(|error| {
        ["type", "code"]
            .iter()
            .any(|key| error.get(*key).and_then(Value::as_str) == Some("hooks_failed"))
    }) || body
        .get("hook_results")
        .and_then(Value::as_object)
        .is_some_and(|phases| {
            phases
                .values()
                .filter_map(Value::as_array)
                .flatten()
                .any(|hook| {
                    hook.get("verdict").and_then(Value::as_bool) == Some(false)
                        && ["deny", "softDeny200", "soft_deny_200"]
                            .iter()
                            .any(|flag| hook.get(*flag).and_then(Value::as_bool) == Some(true))
                })
        })
}

/// A JSON body cannot satisfy an event-stream request. Bound its inspection and
/// fail terminally instead of treating it as a disconnected stream and replaying.
/// Only explicit gateway policy fields identify a policy denial; model text is
/// never searched or copied into diagnostics. Other stream transports are untouched.
pub(crate) async fn reject_json(
    response: &mut StreamResponse,
    idle_timeout: Duration,
) -> Result<(), ApiError> {
    let is_json = response
        .headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| {
            let value = value.trim().to_ascii_lowercase();
            value == "application/json" || value.ends_with("+json")
        });
    if !is_json {
        return Ok(());
    }
    let read = async {
        let mut bytes = Vec::new();
        while let Some(chunk) = response.bytes.next().await {
            let chunk = chunk.ok()?;
            if bytes.len().saturating_add(chunk.len()) > MAX_JSON_BYTES {
                return None;
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice::<Value>(&bytes).ok()
    };
    let denied = tokio::time::timeout(idle_timeout.min(Duration::from_secs(2)), read)
        .await
        .ok()
        .flatten()
        .as_ref()
        .is_some_and(is_denied);
    let reason = if denied {
        POLICY_DENIED
    } else {
        "The server returned JSON instead of the requested event stream. Check its route and policy configuration before explicitly retrying."
    };
    let trace = response
        .headers
        .get("x-portkey-trace-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| uuid::Uuid::parse_str(value).ok())
        .map(|id| format!(" Request / gateway trace ID: {id}."))
        .unwrap_or_default();
    Err(ApiError::InvalidRequest {
        message: format!("{reason} HTTP {}.{trace}", response.status.as_u16()),
    })
}

#[cfg(test)]
#[path = "gateway_denial_tests.rs"]
mod tests;
