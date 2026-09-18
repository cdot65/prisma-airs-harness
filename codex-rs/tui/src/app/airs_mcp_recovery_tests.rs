use super::super::airs_recovery::AirsRecoveryState;
use super::*;
use pretty_assertions::assert_eq;

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
