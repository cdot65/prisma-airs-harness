use super::*;
use pretty_assertions::assert_eq;

#[test]
fn only_bounded_explicit_markers_classify_a_helper_failure() {
    for reason in [
        CredentialRecovery::SignInRequired,
        CredentialRecovery::OutcomeUnknown,
        CredentialRecovery::StoreUnavailable,
        CredentialRecovery::TemporarilyUnavailable,
    ] {
        assert_eq!(
            CredentialRecovery::from_stderr(reason.marker().as_bytes()),
            Some(reason)
        );
    }
    assert_eq!(
        CredentialRecovery::from_stderr(b"HTTP 500: sign in required"),
        None
    );
    assert_eq!(
        CredentialRecovery::from_stderr(b"upstream says AIRS_CREDENTIAL_STATUS:sign_in_required"),
        None
    );
    let oversized = format!(
        "{}\n{}",
        CredentialRecovery::SignInRequired.marker(),
        "x".repeat(16_384)
    );
    assert_eq!(CredentialRecovery::from_stderr(oversized.as_bytes()), None);
}

#[cfg(unix)]
#[tokio::test]
async fn manager_retains_classification_until_a_successful_helper_read() {
    let home = tempfile::tempdir().unwrap();
    let script = home.path().join("helper.sh");
    std::fs::write(
        &script,
        "echo AIRS_CREDENTIAL_STATUS:sign_in_required >&2\nexit 1\n",
    )
    .unwrap();
    let config: ModelProviderAuthInfo = serde_json::from_value(serde_json::json!({
        "command": "/bin/sh", "args": [script], "cwd": home.path(), "refresh_interval_ms": 1
    }))
    .unwrap();
    let manager = super::super::manager::AuthManager::external_bearer_only(config);
    assert_eq!(manager.auth().await, None);
    assert_eq!(
        manager.credential_recovery(),
        Some(CredentialRecovery::SignInRequired)
    );
    std::fs::write(&script, "echo refreshed-test-token\n").unwrap();
    assert_eq!(
        manager.auth().await,
        Some(CodexAuth::from_api_key("refreshed-test-token"))
    );
    assert_eq!(manager.credential_recovery(), None);
}

#[cfg(unix)]
#[tokio::test]
async fn cancelling_the_caller_does_not_kill_a_helper_before_persistence() {
    let home = tempfile::tempdir().unwrap();
    let script = home.path().join("helper.sh");
    std::fs::write(
        &script,
        "touch dispatched\nsleep 0.2\ntouch persisted\necho refreshed-test-token\n",
    )
    .unwrap();
    let config: ModelProviderAuthInfo = serde_json::from_value(serde_json::json!({
        "command": "/bin/sh", "args": [script], "cwd": home.path(), "timeout_ms": 5000
    }))
    .unwrap();
    let caller = tokio::spawn(async move { run_provider_auth_command(&config).await });
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while !home.path().join("dispatched").exists() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    caller.abort();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while !home.path().join("persisted").exists() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}
