use super::*;

#[test]
fn unavailable_gateway_command_credential_never_becomes_unauthenticated_request() {
    let provider: ModelProviderInfo = serde_json::from_value(serde_json::json!({
        "name": "AIRS", "base_url": "https://gateway.example/v1",
        "gateway": {"default_route": "airs-gateway-default"},
        "auth": {"command": "/missing/helper", "cwd": "/"}
    }))
    .unwrap();
    let Err(error) = resolve_provider_auth(None, &provider) else {
        panic!("unavailable bound credential must stop the request");
    };
    assert!(error.to_string().contains("credential helper"));
}
