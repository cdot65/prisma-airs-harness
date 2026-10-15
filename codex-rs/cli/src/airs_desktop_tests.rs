use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

const DENIED: Args = Args {
    allow_http_loopback: false,
};
const LOOPBACK: Args = Args {
    allow_http_loopback: true,
};
const MCP_TOKEN: &str = "fixture-workspace-mcp-token";

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
            .validate(&DENIED)
            .is_err()
    );
}

fn mcp_server(name: &str, url: &str) -> serde_json::Value {
    json!({"name":name,"serverId":"srv-0001","url":url,"accessToken":MCP_TOKEN})
}

fn with_mcp(servers: Vec<serde_json::Value>) -> serde_json::Value {
    let mut value = bootstrap();
    value["params"]["mcpServers"] = json!(servers);
    value
}

fn validate_with(value: serde_json::Value, args: &Args) -> anyhow::Result<ContextBinding> {
    serde_json::from_value::<BootstrapRequest>(value)?.validate(args)
}

#[test]
fn bootstrap_accepts_bounded_workspace_mcp_servers() -> anyhow::Result<()> {
    let context = validate_with(
        with_mcp(vec![
            mcp_server(
                "workspace_tools",
                "https://mcp-airs.example/scm-ngfw-mcp/mcp",
            ),
            mcp_server("z2", "https://mcp-airs.example"),
        ]),
        &DENIED,
    )?;
    assert_eq!(
        serde_json::to_value(&context.mcp_servers)?,
        json!([
            {"name":"workspace_tools","serverId":"srv-0001","url":"https://mcp-airs.example/scm-ngfw-mcp/mcp"},
            {"name":"z2","serverId":"srv-0001","url":"https://mcp-airs.example/"}
        ])
    );
    let serialized = serde_json::to_string(&context)?;
    assert!(!serialized.contains(MCP_TOKEN));
    // Replay contexts written before workspace MCP existed still match.
    let context = validate_with(bootstrap(), &DENIED)?;
    assert_eq!(context.mcp_servers, Vec::new());
    assert!(!serde_json::to_string(&context)?.contains("mcpServers"));
    let mut explicit_null = bootstrap();
    explicit_null["params"]["mcpServers"] = serde_json::Value::Null;
    assert_eq!(
        validate_with(explicit_null, &DENIED)?.mcp_servers,
        Vec::new()
    );
    let loopback = with_mcp(vec![mcp_server("local", "http://127.0.0.1:9/mcp")]);
    assert!(validate_with(loopback.clone(), &DENIED).is_err());
    assert_eq!(
        validate_with(loopback, &LOOPBACK)?.mcp_servers[0].url,
        "http://127.0.0.1:9/mcp"
    );
    // The shortest admitted token is sixteen visible ASCII bytes.
    let mut shortest = mcp_server("workspace_tools", "https://mcp-airs.example/mcp");
    shortest["accessToken"] = json!("t".repeat(16));
    assert_eq!(
        validate_with(with_mcp(vec![shortest]), &DENIED)?.mcp_servers[0].name,
        "workspace_tools"
    );
    Ok(())
}

#[test]
fn bootstrap_rejects_invalid_workspace_mcp_servers() {
    let valid = || mcp_server("workspace_tools", "https://mcp-airs.example/mcp");
    let field = |key: &str, value: serde_json::Value| {
        let mut server = valid();
        server[key] = value;
        server
    };
    let cases: Vec<(&str, Vec<serde_json::Value>)> = vec![
        ("uppercase name", vec![field("name", json!("Workspace"))]),
        ("leading digit", vec![field("name", json!("1tools"))]),
        ("hyphenated name", vec![field("name", json!("work-space"))]),
        ("empty name", vec![field("name", json!(""))]),
        (
            "long name",
            vec![field("name", json!(format!("a{}", "b".repeat(32))))],
        ),
        (
            "reserved host namespace",
            vec![field("name", json!("airs_desktop"))],
        ),
        (
            "reserved apps namespace",
            vec![field("name", json!("codex_apps"))],
        ),
        ("duplicate name", vec![valid(), valid()]),
        ("empty server id", vec![field("serverId", json!(""))]),
        (
            "long server id",
            vec![field("serverId", json!("a".repeat(65)))],
        ),
        (
            "server id punctuation",
            vec![field("serverId", json!("srv_01"))],
        ),
        (
            "plain http",
            vec![field("url", json!("http://mcp-airs.example/mcp"))],
        ),
        (
            "http non-loopback with switch",
            vec![field("url", json!("http://10.0.0.1/mcp"))],
        ),
        (
            "query",
            vec![field("url", json!("https://mcp-airs.example/mcp?x=1"))],
        ),
        (
            "fragment",
            vec![field("url", json!("https://mcp-airs.example/mcp#x"))],
        ),
        (
            "userinfo",
            vec![field("url", json!("https://user:pw@mcp-airs.example/mcp"))],
        ),
        (
            "whitespace url",
            vec![field("url", json!("https://mcp-airs.example/m cp"))],
        ),
        ("relative url", vec![field("url", json!("/mcp"))]),
        (
            "oversized url",
            vec![field(
                "url",
                json!(format!("https://mcp-airs.example/{}", "a".repeat(2048))),
            )],
        ),
        ("empty token", vec![field("accessToken", json!(""))]),
        ("one byte token", vec![field("accessToken", json!("t"))]),
        (
            "short token",
            vec![field("accessToken", json!("t".repeat(15)))],
        ),
        (
            "oversized token",
            vec![field("accessToken", json!("t".repeat(16 * 1024 + 1)))],
        ),
        (
            "token whitespace",
            vec![field("accessToken", json!("fixture token"))],
        ),
        (
            "token control",
            vec![field("accessToken", json!("fixture\u{7}token"))],
        ),
        (
            "token newline",
            vec![field("accessToken", json!("fixture\ntoken"))],
        ),
        ("unknown key", vec![field("headers", json!({"x":"y"}))]),
        (
            "missing token",
            vec![{
                let mut server = valid();
                server.as_object_mut().unwrap().remove("accessToken");
                server
            }],
        ),
        (
            "too many servers",
            (0..9)
                .map(|index| mcp_server(&format!("tools{index}"), "https://mcp-airs.example/mcp"))
                .collect(),
        ),
    ];
    for (case, servers) in cases {
        assert!(
            validate_with(with_mcp(servers), &LOOPBACK).is_err(),
            "{case} must reject the bootstrap"
        );
    }
    let mut not_array = bootstrap();
    not_array["params"]["mcpServers"] = json!({"workspace_tools":valid()});
    assert!(validate_with(not_array, &DENIED).is_err());
}

#[test]
fn workspace_mcp_config_prompts_for_every_tool_and_never_carries_credentials() -> anyhow::Result<()>
{
    let request: BootstrapRequest = serde_json::from_value(with_mcp(vec![mcp_server(
        "workspace_tools",
        "https://mcp-airs.example/mcp",
    )]))?;
    let context = request.validate(&DENIED)?;
    let config = mcp::config(&context.mcp_servers)?;
    assert_eq!(
        config,
        toml::from_str::<toml::Value>(
            r#"
            [workspace_tools]
            url = "https://mcp-airs.example/mcp"
            enabled = true
            required = false
            startup_timeout_sec = 30
            tool_timeout_sec = 120
            default_tools_approval_mode = "prompt"
            "#
        )?
    );
    assert!(!toml::to_string(&config)?.contains(MCP_TOKEN));
    let parsed: std::collections::HashMap<String, codex_config::McpServerConfig> =
        config.try_into()?;
    let server = &parsed["workspace_tools"];
    assert!(!server.required);
    assert_eq!(
        server.default_tools_approval_mode,
        Some(codex_config::AppToolApproval::Prompt)
    );
    let credentials = mcp::credentials(request.mcp_grants(), &context.mcp_servers)?;
    assert_eq!(credentials.len(), 1);
    assert_eq!(credentials[0].server_name, "workspace_tools");
    assert_eq!(credentials[0].url, "https://mcp-airs.example/mcp");
    assert_eq!(
        credentials[0].authorization,
        format!("Bearer {MCP_TOKEN}").as_str()
    );
    Ok(())
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
    let context = request.validate(&DENIED).unwrap();
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
    transport::write_json(&mut bytes, &value, &[secret]).await?;
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

#[tokio::test]
async fn desktop_transport_decodes_deep_unicode_object_keys_before_redaction() -> anyhow::Result<()>
{
    let secret = r#"fixture-"\\-protected-key"#;
    let quoted = serde_json::to_string(secret)?;
    for layers in [9, 12] {
        // Only the first character is escaped: this is small enough to retain
        // the entire credential through ordinary tool-output truncation.
        let mut payload = format!(
            r#"{{"\u0066{}":"fixture-unicode-object-key"}}"#,
            &quoted[2..quoted.len() - 1]
        );
        for _ in 0..layers {
            payload = serde_json::to_string(&payload)?;
        }
        let mut bytes = Vec::new();
        transport::write_json(&mut bytes, &json!({"payload":payload}), &[secret]).await?;
        let outer: serde_json::Value = serde_json::from_slice(&bytes)?;
        let mut decoded = outer["payload"].clone();
        for _ in 0..=layers {
            if let Some(text) = decoded.as_str() {
                decoded = serde_json::from_str(text)?;
            }
        }
        assert_eq!(
            decoded["[protected credential]"],
            "fixture-unicode-object-key"
        );
        assert!(
            !decoded
                .as_object()
                .unwrap()
                .keys()
                .any(|key| key.contains(secret))
        );
    }
    Ok(())
}

#[tokio::test]
async fn desktop_transport_masks_unchecked_unicode_json_structures() -> anyhow::Result<()> {
    let secret = r#"fixture-"\\-protected-key"#;
    let quoted = serde_json::to_string(secret)?;
    let object = format!(r#"{{"\u0066{}":"fixture"}}"#, &quoted[2..quoted.len() - 1]);
    for payload in [
        format!("{}{}{}", "[".repeat(130), object, "]".repeat(130)),
        format!("{object} incomplete JSON"),
        format!(r#"unwrapped \u0066{}"#, &quoted[2..quoted.len() - 1]),
    ] {
        let mut bytes = Vec::new();
        transport::write_json(&mut bytes, &json!({"payload":payload}), &[secret]).await?;
        let output: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(output["payload"], "[protected credential]");
    }
    Ok(())
}
