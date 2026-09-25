use super::super::airs_recovery::AirsRecoveryState;
use super::super::airs_recovery::RecoveryOperation;
use super::*;
use crate::airs_mcp_manager::Event;
use crate::airs_mcp_manager::Progress;
use crate::airs_mcp_manager::views;
use crate::app_event::AppEvent;
use crate::app_event_sender::AppEventSender;
use codex_app_server_protocol::McpAuthStatus;
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
        let (first, first_token) = state.begin(RecoveryOperation::Mcp).unwrap();
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
        let (second, second_token) = state.begin(RecoveryOperation::Doctor).unwrap();
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
        assert!(state.begin(RecoveryOperation::Mcp).is_some());
    }
}

#[test]
fn discovery_requires_successful_initialization_without_assuming_tool_permissions() {
    let mut status: McpServerStatus = serde_json::from_value(serde_json::json!({
        "name": "mcp-server-1", "runtimeStatus": null, "pluginId": null,
        "serverInfo": {"name": "utilities", "version": "1"},
        "tools": {"current_time": {"name": "current_time", "inputSchema": {"type": "object"}}},
        "toolsError": null, "resources": [], "resourceTemplates": [], "authStatus": "oAuth"
    }))
    .unwrap();
    assert_eq!(discovered_tools("mcp-server-1", &[status.clone()]), Ok(1));
    assert!(discovered_tools("another-server", &[status.clone()]).is_err());
    status.tools_error = Some("Authentication required".into());
    assert!(discovered_tools("mcp-server-1", &[status.clone()]).is_err());
    status.tools_error = None;
    status.server_info = None;
    assert!(discovered_tools("mcp-server-1", &[status.clone()]).is_err());
    status.auth_status = McpAuthStatus::NotLoggedIn;
    assert!(discovered_tools("mcp-server-1", &[status.clone()]).is_err());
    status.server_info = Some(
        serde_json::from_value(serde_json::json!({"name":"utilities","version":"1"})).unwrap(),
    );
    // Discovery can be public or use a bearer/helper credential. Do not invent OAuth success.
    for auth in [
        McpAuthStatus::NotLoggedIn,
        McpAuthStatus::BearerToken,
        McpAuthStatus::CredentialHelper,
        McpAuthStatus::Unsupported,
    ] {
        status.auth_status = auth;
        assert_eq!(discovered_tools("mcp-server-1", &[status.clone()]), Ok(1));
    }
    for runtime in [
        McpServerConnectionStatus::AuthenticationRequired,
        McpServerConnectionStatus::Failed,
        McpServerConnectionStatus::Cancelled,
        McpServerConnectionStatus::Disabled,
        McpServerConnectionStatus::NotStarted,
        McpServerConnectionStatus::Starting,
    ] {
        status.runtime_status = Some(runtime);
        assert!(discovered_tools("mcp-server-1", &[status.clone()]).is_err());
    }
}

#[test]
fn cancelling_consent_invalidates_late_completions_and_allows_retry() {
    let mut state = AirsRecoveryState::default();
    let (first, cancellation) = state.begin(RecoveryOperation::Mcp).unwrap();
    assert!(state.begin(RecoveryOperation::Mcp).is_none());
    assert!(state.is_current(first));
    state.cancel();
    assert!(cancellation.is_cancelled());
    assert!(!state.is_current(first));
    let (second, next) = state.begin(RecoveryOperation::Mcp).unwrap();
    assert!(!state.finish(first));
    assert!(state.is_current(second));
    assert!(!next.is_cancelled());
    assert!(state.finish(second));
    assert!(!state.finish(second));
    assert!(state.begin(RecoveryOperation::Mcp).is_some());
}
