use super::*;
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[tokio::test]
async fn desktop_memory_auth_pins_destination_and_checks_revocation_before_headers() {
    let provider: ModelProviderInfo = serde_json::from_value(serde_json::json!({
        "name":"Prisma AIRS AI Gateway","base_url":"https://gateway.example/v1",
        "gateway":{"default_route":"airs-gateway-default"},"requires_openai_auth":false
    }))
    .unwrap();
    let active = Arc::new(AtomicBool::new(true));
    let check = Arc::clone(&active);
    let mut key = HeaderValue::from_static("fixture-protected-key");
    key.set_sensitive(true);
    // Keep the process-global production binding untouched: other provider tests
    // resolve unrelated providers in the same test executable.
    let binding = DesktopGatewayAuth {
        provider: provider.clone(),
        auth: Arc::new(MemoryCredential {
            key,
            session_check: Arc::new(move || {
                if check.load(Ordering::Acquire) {
                    Ok(())
                } else {
                    Err(std::io::Error::other("Revoked"))
                }
            }),
        }),
    };
    let auth = binding.resolve(&provider).unwrap();
    let headers = auth.resolve_auth_headers().await.unwrap();
    assert_eq!(headers["x-portkey-api-key"], "fixture-protected-key");
    assert!(headers["x-portkey-api-key"].is_sensitive());
    let mut other = provider.clone();
    other.base_url = Some("https://other.example/v1".into());
    assert!(binding.resolve(&other).is_err());
    active.store(false, Ordering::Release);
    assert!(auth.resolve_auth_headers().await.is_err());
    assert_eq!(auth.to_auth_headers(), HeaderMap::new());
}
