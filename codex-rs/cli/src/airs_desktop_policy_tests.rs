use super::*;
use crate::airs_desktop::LocalAccess;
use pretty_assertions::assert_eq;

#[test]
fn desktop_admission_rejects_valid_steer_with_auxiliary_developer_context() -> anyhow::Result<()> {
    let context: ContextBinding = serde_json::from_value(json!({
        "tenantId":"11111111-1111-4111-8111-111111111111",
        "workspaceId":"22222222-2222-4222-8222-222222222222",
        "credentialId":"33333333-3333-4333-8333-333333333333",
        "credentialEpoch":7,"gatewayEndpoint":"https://gateway.example/v1",
        "projectPath":null,"gatewayConfig":null,"model":null,"localAccess":"readOnly"
    }))?;
    assert_eq!(context.local_access, LocalAccess::ReadOnly);
    let mut request = JSONRPCRequest {
        id: codex_app_server_protocol::RequestId::Integer(99),
        method: "turn/steer".into(),
        trace: None,
        params: Some(
            json!({"threadId":"thread-fixture","expectedTurnId":"turn-fixture",
            "input":[{"type":"text","text":"bounded steering","text_elements":[]}],
            "additionalContext":{"source":{"kind":"application","value":"x".repeat(60*1024)}}}),
        ),
    };
    assert!(
        ClientRequest::try_from(request.clone()).is_ok(),
        "fixture must pass pinned protocol decoding before admission"
    );
    assert!(
        validate(
            &mut request,
            &context,
            &HashSet::from(["thread-fixture".to_owned()])
        )
        .is_err()
    );
    Ok(())
}
