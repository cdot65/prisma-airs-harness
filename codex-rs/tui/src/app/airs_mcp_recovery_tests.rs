use super::super::airs_recovery::AirsRecoveryState;
use super::*;
use crate::airs_mcp_manager::Event;
use crate::airs_mcp_manager::Progress;
use crate::airs_mcp_manager::views;
use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use pretty_assertions::assert_eq;

#[test]
fn queued_mcp_cancellation_cannot_cancel_a_newer_recovery_operation() {
    for progress in [
        None,
        Some(Progress::ExchangingCode),
        Some(Progress::SavingCredential),
        Some(Progress::DiscoveringTools),
    ] {
        let mut state = AirsRecoveryState::default();
        let (first, first_token) = state.begin().unwrap();
        let view = match progress {
            Some(progress) => views::progress(first, progress),
            None => views::waiting(first),
        };
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = AppEventSender::new(tx);
        // Queue both the selected Cancel action and Escape before processing either.
        (view.items[0].actions[0])(&sender);
        (view.on_cancel.unwrap())(&sender);
        assert!(state.finish(first));
        let (second, second_token) = state.begin().unwrap();
        for _ in 0..2 {
            let AppEvent::AirsMcpManager(Event::Cancel(attempt)) = rx.try_recv().unwrap() else {
                panic!("expected an attempt-scoped MCP cancellation");
            };
            assert_eq!(attempt, first);
            assert!(!state.cancel_attempt(attempt));
            assert!(state.is_current(second));
            assert!(!second_token.is_cancelled());
        }
        assert!(!first_token.is_cancelled());
        assert!(state.cancel_attempt(second));
        assert!(second_token.is_cancelled());
        assert!(!state.cancel_attempt(second));
        assert!(!state.finish(second));
        assert!(state.begin().is_some());
    }
}

#[test]
fn reconnect_requires_oauth_and_successful_server_initialization() {
    let mut status: McpServerStatus = serde_json::from_value(serde_json::json!({
        "name": "mcp-server-1", "runtimeStatus": null, "pluginId": null,
        "serverInfo": {"name": "utilities", "version": "1"},
        "tools": {"current_time": {"name": "current_time", "inputSchema": {"type": "object"}}},
        "toolsError": null, "resources": [], "resourceTemplates": [], "authStatus": "oAuth"
    }))
    .unwrap();
    assert_eq!(connected_tools("mcp-server-1", &[status.clone()]), Ok(1));
    assert!(connected_tools("another-server", &[status.clone()]).is_err());
    status.tools_error = Some("Authentication required".into());
    assert!(connected_tools("mcp-server-1", &[status.clone()]).is_err());
    status.tools_error = None;
    status.server_info = None;
    assert!(connected_tools("mcp-server-1", &[status.clone()]).is_err());
    status.auth_status = McpAuthStatus::NotLoggedIn;
    assert!(connected_tools("mcp-server-1", &[status]).is_err());
}

#[test]
fn cancelling_consent_invalidates_late_completions_and_allows_retry() {
    let mut state = AirsRecoveryState::default();
    let (first, cancellation) = state.begin().unwrap();
    assert!(state.begin().is_none());
    assert!(state.is_current(first));
    state.cancel();
    assert!(cancellation.is_cancelled());
    assert!(!state.is_current(first));
    let (second, next) = state.begin().unwrap();
    assert!(!state.finish(first));
    assert!(state.is_current(second));
    assert!(!next.is_cancelled());
    assert!(state.finish(second));
    assert!(!state.finish(second));
    assert!(state.begin().is_some());
}
