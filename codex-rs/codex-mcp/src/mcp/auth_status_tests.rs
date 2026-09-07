use super::*;
use codex_exec_server_test_support::environment_manager_without_environments;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn helper_inventory_does_not_run_helper_or_discover_oauth() {
    let directory = tempfile::tempdir().unwrap();
    let context = McpRuntimeContext::new(
        Arc::new(environment_manager_without_environments()),
        directory.path().to_path_buf(),
    );
    let mut config: McpServerConfig = serde_json::from_value(serde_json::json!({
        "url": "http://127.0.0.1:1/mcp",
        "http_headers_helper": "airs-test-helper-must-not-run"
    }))
    .unwrap();
    for (enabled, expected) in [
        (true, McpAuthState::CredentialHelper),
        (false, McpAuthState::Unsupported),
    ] {
        config.enabled = enabled;
        let state = compute_auth_status(
            "security",
            &config,
            OAuthCredentialsStoreMode::Keyring,
            AuthKeyringBackendKind::Direct,
            /*has_runtime_auth*/ false,
            &context,
            StreamableHttpRedirectMode::Legacy,
        )
        .await
        .unwrap();
        assert_eq!(state, expected);
    }
}
