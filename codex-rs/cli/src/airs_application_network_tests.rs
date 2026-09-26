use super::*;
use codex_http_client::NetworkPolicyDenied;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn helper_policy_honors_managed_domains_and_rejects_malformed_rules() {
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join("requirements.toml");
    std::fs::write(
        &path,
        "[application.network.domains]\n'gateway.example' = 'allow'\n'issuer.example' = 'deny'",
    )
    .unwrap();
    let mut overrides = LoaderOverrides::without_managed_config_for_tests();
    overrides.system_requirements_path = Some(path.clone());
    let policy = load_with_overrides(&overrides).await.unwrap();
    assert!(
        policy
            .acquire(&"https://gateway.example/v1/responses".parse().unwrap())
            .is_ok()
    );
    assert_eq!(
        policy
            .acquire(&"https://issuer.example/token".parse().unwrap())
            .map(|_| ()),
        Err(NetworkPolicyDenied::Destination)
    );
    assert_eq!(
        policy
            .acquire(&"http://gateway.example/token".parse().unwrap())
            .map(|_| ()),
        Err(NetworkPolicyDenied::Destination)
    );
    std::fs::write(&path, "[application.network]\nenabled = 'invalid'").unwrap();
    assert!(load_with_overrides(&overrides).await.is_err());
}

#[tokio::test]
async fn standalone_mcp_transport_obeys_loaded_application_requirements() {
    use codex_exec_server::HttpClient;
    use codex_exec_server::HttpRedirectPolicy;
    use codex_exec_server::HttpRequestParams;
    use codex_exec_server::RouteAwareHttpClient;
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join("requirements.toml");
    std::fs::write(&path, "[application.network]\nenabled = true").unwrap();
    let mut overrides = LoaderOverrides::without_managed_config_for_tests();
    overrides.system_requirements_path = Some(path);
    let mut config = codex_core::config::ConfigBuilder::default()
        .codex_home(home.path().to_path_buf())
        .loader_overrides(overrides)
        .build()
        .await
        .unwrap();
    let server = wiremock::MockServer::start().await;
    let client = RouteAwareHttpClient::new(for_config(&config)).with_tls_backend_fallback();
    let result = client
        .http_request(HttpRequestParams {
            method: "POST".into(),
            url: format!("{}/mcp", server.uri()),
            headers: vec![],
            body: Some(b"fixture-private-body".to_vec().into()),
            timeout_ms: Some(1000),
            redirect_policy: HttpRedirectPolicy::Stop,
            request_id: "policy-fixture".into(),
            stream_response: false,
        })
        .await;
    assert!(result.is_err());
    assert!(server.received_requests().await.unwrap().is_empty());

    // A live parent policy must not be replaced by this helper's snapshot.
    let controller = NetworkPolicyController::default();
    config.application_network_policy = controller.policy();
    assert_eq!(for_config(&config).network_policy(), &controller.policy());
}
