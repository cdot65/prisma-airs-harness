use super::*;
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[tokio::test]
async fn desktop_mcp_auth_binds_name_and_url_and_checks_revocation() {
    let active = Arc::new(AtomicBool::new(true));
    let check = Arc::clone(&active);
    let session_check: SessionCheck = Arc::new(move || {
        if check.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(std::io::Error::other("Revoked"))
        }
    });
    // Keep the process-global binding untouched; other tests share this executable.
    let bound = bind(
        vec![DesktopMcpCredential {
            server_name: "workspace_tools".into(),
            url: "https://gateway.example/workspace/mcp".into(),
            authorization: HeaderValue::from_static("Bearer fixture-mcp-token"),
        }],
        &session_check,
    )
    .unwrap();
    let auth = resolve(
        &bound,
        "workspace_tools",
        "https://gateway.example/workspace/mcp",
    )
    .unwrap();
    let headers = auth.resolve_auth_headers().await.unwrap();
    assert_eq!(headers[AUTHORIZATION], "Bearer fixture-mcp-token");
    assert!(headers[AUTHORIZATION].is_sensitive());
    assert!(resolve(&bound, "workspace_tools", "https://other.example/mcp").is_none());
    assert!(
        resolve(
            &bound,
            "other_tools",
            "https://gateway.example/workspace/mcp"
        )
        .is_none()
    );
    active.store(false, Ordering::Release);
    assert!(auth.resolve_auth_headers().await.is_err());
    assert_eq!(auth.to_auth_headers(), HeaderMap::new());
}

#[test]
fn desktop_mcp_auth_rejects_duplicate_server_names() {
    let session_check: SessionCheck = Arc::new(|| Ok(()));
    let credential = || DesktopMcpCredential {
        server_name: "workspace_tools".into(),
        url: "https://gateway.example/workspace/mcp".into(),
        authorization: HeaderValue::from_static("Bearer fixture-mcp-token"),
    };
    assert!(bind(vec![credential(), credential()], &session_check).is_err());
}
