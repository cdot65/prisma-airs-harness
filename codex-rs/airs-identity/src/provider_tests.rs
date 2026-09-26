use super::*;

#[tokio::test]
async fn discovery_transport_failure_preserves_cause_without_request_url() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test endpoint");
    let address = listener.local_addr().expect("test endpoint address");
    // A plain HTTP response cannot complete TLS, so discovery fails locally and deterministically.
    let server = tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        let (mut stream, _) = tokio::time::timeout(Duration::from_secs(5), listener.accept())
            .await
            .expect("discovery connection deadline")
            .expect("accept discovery connection");
        stream
            .write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n")
            .await
            .ok();
    });
    let result = Provider::discover(
        IdentityConfig {
            issuer: format!("https://{address}"),
            client_id: "test-client".into(),
            audience: "test-audience".into(),
        },
        codex_http_client::NetworkPolicy::unmanaged(),
    )
    .await;
    let error = match result {
        Ok(_) => panic!("plain HTTP must not complete issuer TLS"),
        Err(error) => error,
    };
    server.await.expect("test endpoint task");
    assert_eq!(error.to_string(), "issuer discovery unavailable");
    assert!(
        error.chain().count() > 2,
        "retain the underlying transport cause"
    );
    let details = format!("{error:#}");
    assert!(!details.contains("https://"), "strip the request URL");
    assert!(
        !details.contains(".well-known"),
        "strip the discovery request path"
    );
}
