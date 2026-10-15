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

#[test]
fn desktop_admission_reads_owned_mcp_status_but_refuses_login_reload_and_direct_calls()
-> anyhow::Result<()> {
    let context: ContextBinding = serde_json::from_value(json!({
        "tenantId":"11111111-1111-4111-8111-111111111111",
        "workspaceId":"22222222-2222-4222-8222-222222222222",
        "credentialId":"33333333-3333-4333-8333-333333333333",
        "credentialEpoch":7,"gatewayEndpoint":"https://gateway.example/v1",
        "projectPath":null,"gatewayConfig":null,"model":null,"localAccess":"readOnly",
        "mcpServers":[{"name":"workspace_tools","serverId":"srv-1","url":"https://mcp.example/mcp"}]
    }))?;
    let threads = HashSet::from(["thread-fixture".to_owned()]);
    let request = |method: &str, params: Value| JSONRPCRequest {
        id: codex_app_server_protocol::RequestId::Integer(40),
        method: method.into(),
        trace: None,
        params: Some(params),
    };
    let mut status = request(
        "mcpServerStatus/list",
        json!({"threadId":"thread-fixture","detail":"toolsAndAuthOnly","limit":8}),
    );
    assert!(matches!(
        validate(&mut status, &context, &threads)?,
        ClientRequest::McpServerStatusList { .. }
    ));
    // Omitted detail would mean Full inventory; Desktop always pins the
    // bounded tools-and-auth view and one page of at most eight servers.
    for (params, limit) in [
        (json!({"threadId":"thread-fixture"}), 8),
        (
            json!({"threadId":"thread-fixture","detail":null,"limit":null}),
            8,
        ),
        (
            json!({"threadId":"thread-fixture","limit":3,"cursor":null}),
            3,
        ),
    ] {
        let mut status = request("mcpServerStatus/list", params);
        let ClientRequest::McpServerStatusList { params, .. } =
            validate(&mut status, &context, &threads)?
        else {
            anyhow::bail!("expected an MCP status request");
        };
        assert_eq!(
            serde_json::to_value(&params)?["detail"],
            json!("toolsAndAuthOnly")
        );
        assert_eq!(params.limit, Some(limit));
    }
    for params in [
        json!({"threadId":"thread-fixture","detail":"full"}),
        json!({"threadId":"thread-fixture","detail":"everything"}),
        json!({"threadId":"thread-fixture","limit":9}),
        json!({"threadId":"thread-fixture","limit":0}),
        json!({"threadId":"thread-fixture","limit":-1}),
        json!({"threadId":"thread-fixture","limit":"8"}),
        json!({"threadId":"thread-fixture","cursor":7}),
    ] {
        let mut status = request("mcpServerStatus/list", params.clone());
        assert!(
            validate(&mut status, &context, &threads).is_err(),
            "{params} must be refused"
        );
    }
    for (method, params) in [
        // Status is bound to an owned conversation.
        ("mcpServerStatus/list", json!({})),
        ("mcpServerStatus/list", json!({"threadId":"unowned-thread"})),
        (
            "mcpServerStatus/list",
            json!({"threadId":"thread-fixture","config":{"mcp_servers":{}}}),
        ),
        (
            "mcpServer/oauth/login",
            json!({"threadId":"thread-fixture","name":"workspace_tools"}),
        ),
        (
            "config/mcpServer/reload",
            json!({"threadId":"thread-fixture"}),
        ),
        (
            "mcpServer/tool/call",
            json!({"threadId":"thread-fixture","server":"workspace_tools","tool":"lookup","arguments":{}}),
        ),
        (
            "mcpServer/resource/read",
            json!({"threadId":"thread-fixture","server":"workspace_tools","uri":"file:///etc/passwd"}),
        ),
        (
            "config/value/write",
            json!({"threadId":"thread-fixture","keyPath":"mcp_servers.x.url","value":"https://x","mergeStrategy":"replace"}),
        ),
    ] {
        let mut forbidden = request(method, params);
        assert!(
            validate(&mut forbidden, &context, &threads).is_err(),
            "{method} must stay unavailable in Desktop"
        );
    }
    Ok(())
}
