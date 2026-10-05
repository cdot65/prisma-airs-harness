use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

fn bootstrap() -> serde_json::Value {
    json!({"id":0,"method":"desktop/bootstrap","params":{
        "v":1,"tenantId":"11111111-1111-4111-8111-111111111111",
        "workspaceId":"22222222-2222-4222-8222-222222222222",
        "credentialId":"33333333-3333-4333-8333-333333333333","credentialEpoch":1,
        "gatewayEndpoint":"https://gateway.example/v1","apiKey":"fixture-protected-key",
        "projectPath":null,"gatewayConfig":null,"model":null
    }})
}

#[test]
fn bootstrap_rejects_unknown_fields_and_unselected_write_access() {
    let mut value = bootstrap();
    value["params"]["credentialFile"] = json!("/tmp/auth.json");
    assert!(serde_json::from_value::<BootstrapRequest>(value).is_err());
    let mut value = bootstrap();
    value["params"]["localAccess"] = json!("workspaceWrite");
    assert!(
        serde_json::from_value::<BootstrapRequest>(value)
            .unwrap()
            .validate()
            .is_err()
    );
}

#[tokio::test]
async fn bounded_reader_rejects_oversized_and_truncated_frames_without_echo() {
    let mut input = std::io::Cursor::new(b"fixture-secret-without-newline");
    let error = read_frame(&mut input, /*limit*/ 12).await.err().unwrap();
    assert_eq!(error.to_string(), "Desktop frame exceeded its limit");
    let mut input = std::io::Cursor::new(b"fixture-secret-without-newline");
    let error = read_frame(&mut input, FRAME_LIMIT).await.err().unwrap();
    assert_eq!(error.to_string(), "Incomplete desktop frame");
}

#[test]
fn context_serialization_excludes_bootstrap_credential() {
    let request: BootstrapRequest = serde_json::from_value(bootstrap()).unwrap();
    let context = request.validate().unwrap();
    assert!(
        !serde_json::to_string(&context)
            .unwrap()
            .contains("fixture-protected-key")
    );
}

#[tokio::test]
async fn desktop_transport_redacts_json_escaped_and_nested_credentials() -> anyhow::Result<()> {
    let secret = r#"fixture-"\\-protected-key"#;
    let nested = serde_json::to_string(&json!({"diagnostic":secret}))?;
    let twice = serde_json::to_string(&nested)?;
    let encoded = serde_json::to_string(secret)?;
    let key = format!("diagnostic-{}", &encoded[1..encoded.len() - 1]);
    let mut repeated = secret.to_owned();
    for _ in 0..12 {
        let quoted = serde_json::to_string(&repeated)?;
        repeated = quoted[1..quoted.len() - 1].to_owned();
    }
    let deep_key = format!("repeated-{repeated}");
    let value = json!({"direct":secret,"arguments":nested,"nested":twice,secret:"private",key:"encoded-private",deep_key:"repeated-private"});
    let mut bytes = Vec::new();
    transport::write_json(&mut bytes, &value, secret).await?;
    let output: String = String::from_utf8(bytes)?;
    let decoded: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(decoded["direct"], "[protected credential]");
    assert_eq!(
        decoded["arguments"],
        r#"{"diagnostic":"[protected credential]"}"#
    );
    assert!(
        decoded["nested"]
            .as_str()
            .unwrap()
            .contains("[protected credential]")
    );
    assert_eq!(decoded["[protected credential]"], "private");
    assert_eq!(
        decoded["diagnostic-[protected credential]"],
        "encoded-private"
    );
    assert_eq!(
        decoded["repeated-[protected credential]"],
        "repeated-private"
    );
    Ok(())
}
