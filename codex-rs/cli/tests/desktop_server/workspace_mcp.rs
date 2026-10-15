//! Workspace MCP servers granted at bootstrap: memory-only bearer, native approval.
use super::*;
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use wiremock::Request;
use wiremock::Respond;

const MCP_TOKEN: &str = "fixture-workspace-mcp-bearer-never-persisted";
const MCP_RESULT: &str = "FIXTURE_WORKSPACE_MCP_RESULT";

/// A minimal Streamable HTTP MCP server that requires the granted bearer.
struct McpFixture;

impl Respond for McpFixture {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let authorization = request
            .headers
            .get("authorization")
            .and_then(|value| value.to_str().ok());
        if authorization != Some(format!("Bearer {MCP_TOKEN}").as_str()) {
            return ResponseTemplate::new(401).set_body_json(json!({"error":"unauthorized"}));
        }
        if request.method.as_str() != "POST" {
            return ResponseTemplate::new(405);
        }
        let body: Value = serde_json::from_slice(&request.body).unwrap_or_default();
        let Some(id) = body.get("id").cloned() else {
            return ResponseTemplate::new(202);
        };
        let result = match body["method"].as_str() {
            Some("initialize") => json!({
                "protocolVersion": body["params"]["protocolVersion"],
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "workspace-fixture", "version": "1.0.0"}
            }),
            Some("tools/list") => json!({"tools": [{
                "name": "lookup",
                "description": "Look up a workspace record",
                "inputSchema": {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"]}
            }]}),
            Some("tools/call") => json!({
                "content": [{"type": "text", "text": MCP_RESULT}],
                "isError": false
            }),
            _ => json!({}),
        };
        ResponseTemplate::new(200)
            .insert_header("content-type", "application/json")
            .set_body_json(json!({"jsonrpc": "2.0", "id": id, "result": result}))
    }
}

/// Calls the workspace tool at the start of each turn, then completes.
struct ModelFixture {
    turns: AtomicUsize,
}

impl Respond for ModelFixture {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = serde_json::from_slice(&request.body).unwrap_or_default();
        let starts_turn = body["input"]
            .as_array()
            .and_then(|input| input.last())
            .is_some_and(|item| item["role"] == "user");
        let events = if starts_turn {
            let turn = self.turns.fetch_add(1, Ordering::SeqCst);
            let call_id = format!("workspace-mcp-call-{turn}");
            let arguments = json!({"query": format!("record-{turn}")}).to_string();
            let call = workspace_tool_call(&body["tools"], &call_id, &arguments)
                .unwrap_or_else(|| json!({"type":"message","role":"assistant","id":"missing-tool","content":[{"type":"output_text","text":"MISSING_WORKSPACE_TOOL","annotations":[]}]}));
            sse(vec![
                json!({"type":"response.created","response":{"id":format!("mcp-call-{turn}")}}),
                json!({"type":"response.output_item.done","item":call}),
                json!({"type":"response.completed","response":{"id":format!("mcp-call-{turn}"),"usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
            ])
        } else {
            completed()
        };
        match events {
            Ok(events) => ResponseTemplate::new(200).set_body_raw(events, "text/event-stream"),
            Err(_) => ResponseTemplate::new(500),
        }
    }
}

/// Finds the workspace tool in the model catalog in either flat or namespace form.
fn workspace_tool_call(tools: &Value, call_id: &str, arguments: &str) -> Option<Value> {
    tools.as_array()?.iter().find_map(|tool| {
        let name = tool["name"].as_str()?;
        if !name.contains("workspace_tools") {
            return None;
        }
        match tool["type"].as_str()? {
            "function" if name.ends_with("lookup") => Some(json!({
                "type":"function_call","id":format!("{call_id}-item"),"call_id":call_id,
                "name":name,"arguments":arguments
            })),
            "namespace" => tool["tools"].as_array()?.iter().find_map(|child| {
                (child["name"] == "lookup").then(|| {
                    json!({
                        "type":"function_call","id":format!("{call_id}-item"),"call_id":call_id,
                        "namespace":name,"name":"lookup","arguments":arguments
                    })
                })
            }),
            _ => None,
        }
    })
}

impl Desktop {
    /// Returns the first settled startup status for `server`, including already-seen events.
    async fn startup(&mut self, server: &str) -> Result<Value> {
        let settled = |value: &Value| {
            value["method"] == "mcpServer/startupStatus/updated"
                && value["params"]["name"] == server
                && value["params"]["status"] != "starting"
        };
        if let Some(value) = self.seen.iter().find(|value| settled(value)) {
            return Ok(value.clone());
        }
        loop {
            let value = self.next().await?;
            if settled(&value) {
                return Ok(value);
            }
        }
    }

    async fn method(&mut self, name: &str) -> Result<Value> {
        loop {
            let value = self.next().await?;
            if value["method"] == name {
                return Ok(value);
            }
        }
    }
}

#[tokio::test]
async fn desktop_server_workspace_mcp_uses_memory_bearer_and_native_approval() -> Result<()> {
    let gateway = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ModelFixture {
            turns: AtomicUsize::new(0),
        })
        .mount(&gateway)
        .await;
    let workspace = MockServer::start().await;
    Mock::given(path("/workspace/mcp"))
        .respond_with(McpFixture)
        .mount(&workspace)
        .await;
    let mcp_url = format!("{}/workspace/mcp", workspace.uri());
    let temp = tempfile::tempdir()?;
    let mut desktop = Desktop::launch(
        temp.path(),
        json!({
            "v":1,"tenantId":"11111111-1111-4111-8111-111111111111",
            "workspaceId":"22222222-2222-4222-8222-222222222222",
            "credentialId":"33333333-3333-4333-8333-333333333333","credentialEpoch":7,
            "gatewayEndpoint":format!("{}/v1", gateway.uri()),"apiKey":KEY,"projectPath":null,
            "localAccess":"readOnly","gatewayConfig":null,"model":null,
            "mcpServers":[{"name":"workspace_tools","serverId":"srv-0001","url":mcp_url,"accessToken":MCP_TOKEN}]
        }),
    )
    .await?;
    let receipt = desktop
        .seen
        .iter()
        .find(|value| value["id"] == 0)
        .cloned()
        .context("bootstrap receipt")?;
    eprintln!("WIRE bootstrap receipt: {receipt}");
    assert_eq!(
        receipt["result"]["mcpServers"],
        json!([{"name":"workspace_tools","serverId":"srv-0001"}])
    );
    assert_eq!(receipt["result"]["runtimeVersion"], "0.1.7-desktop.2");
    // Neither child processes nor the runtime itself carry the token in their environment.
    #[cfg(target_os = "macos")]
    {
        let environment = std::process::Command::new("/bin/ps")
            .args(["-wwE", "-o", "command=", "-p"])
            .arg(desktop.child.id().context("desktop pid")?.to_string())
            .output()?;
        let environment = String::from_utf8_lossy(&environment.stdout);
        anyhow::ensure!(
            environment.contains("desktop-server"),
            "environment probe must observe the runtime"
        );
        anyhow::ensure!(
            !environment.contains(MCP_TOKEN),
            "MCP token in process environment"
        );
    }

    let thread = desktop.thread().await?;
    let settled = desktop.startup("workspace_tools").await?;
    eprintln!("WIRE startup status: {settled}");
    assert_eq!(settled["params"]["status"], "ready");
    desktop
        .send(json!({"id":5,"method":"mcpServerStatus/list","params":{"threadId":thread}}))
        .await?;
    let status = desktop.response(5).await?;
    eprintln!("WIRE mcpServerStatus/list: {status}");
    let server = status["result"]["data"]
        .as_array()
        .context("status list")?
        .iter()
        .find(|server| server["name"] == "workspace_tools")
        .context("workspace server status")?;
    assert_eq!(server["runtimeStatus"], "connected");
    assert_eq!(server["authStatus"], "bearerToken");
    assert_eq!(server["tools"]["lookup"]["name"], "lookup");
    for (id, method, params) in [
        (
            6,
            "mcpServer/oauth/login",
            json!({"threadId":thread,"name":"workspace_tools"}),
        ),
        (7, "config/mcpServer/reload", json!({"threadId":thread})),
        (
            8,
            "mcpServer/tool/call",
            json!({"threadId":thread,"server":"workspace_tools","tool":"lookup","arguments":{"query":"direct"}}),
        ),
    ] {
        desktop
            .send(json!({"id":id,"method":method,"params":params}))
            .await?;
        let response = desktop.response(id).await?;
        assert_eq!(
            response,
            json!({"id":id,"error":{"code":-32602,"message":"Desktop request rejected"}})
        );
    }

    // Turn 1: Desktop declines; the MCP server is never called.
    desktop.send(json!({"id":10,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Look up record 0","text_elements":[]}]}})).await?;
    anyhow::ensure!(
        desktop.response(10).await?.get("result").is_some(),
        "turn rejected"
    );
    let approval = desktop.method("mcpServer/elicitation/request").await?;
    eprintln!("WIRE elicitation request: {approval}");
    assert_eq!(approval["params"]["threadId"], thread);
    assert_eq!(approval["params"]["serverName"], "workspace_tools");
    assert_eq!(approval["params"]["mode"], "form");
    assert_eq!(
        approval["params"]["_meta"]["codex_approval_kind"],
        "mcp_tool_call"
    );
    desktop
        .send(
            json!({"id":approval["id"],"result":{"action":"decline","content":null,"_meta":null}}),
        )
        .await?;
    loop {
        let value = desktop.next().await?;
        if value["method"] == "item/completed" && value["params"]["item"]["type"] == "mcpToolCall" {
            eprintln!("WIRE declined mcpToolCall completed: {value}");
        }
        if value["method"] == "turn/completed" {
            break;
        }
    }
    let calls = |requests: &[Request]| {
        requests
            .iter()
            .filter(|request| String::from_utf8_lossy(&request.body).contains("tools/call"))
            .count()
    };
    let requests = workspace
        .received_requests()
        .await
        .context("MCP requests")?;
    assert_eq!(calls(&requests), 0, "declined tool reached the MCP server");

    // Turn 2: Desktop accepts; the call reaches the server with the memory bearer.
    desktop.send(json!({"id":11,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Look up record 1","text_elements":[]}]}})).await?;
    anyhow::ensure!(
        desktop.response(11).await?.get("result").is_some(),
        "turn rejected"
    );
    let approval = desktop.method("mcpServer/elicitation/request").await?;
    desktop
        .send(json!({"id":approval["id"],"result":{"action":"accept"}}))
        .await?;
    desktop.method("turn/completed").await?;
    // The started item precedes the approval request; keep the observed order.
    let call_events: Vec<Value> = desktop
        .seen
        .iter()
        .filter(|value| {
            value["params"]["item"]["id"] == "workspace-mcp-call-1"
                || value["method"] == "item/mcpToolCall/progress"
                || (value["method"] == "mcpServer/elicitation/request"
                    && value["params"]["_meta"]["tool_params"]["query"] == "record-1")
        })
        .cloned()
        .collect();
    for value in &call_events {
        eprintln!("WIRE accepted call event: {value}");
    }
    let find = |method: &str| {
        call_events
            .iter()
            .find(|value| value["method"] == method)
            .cloned()
            .with_context(|| format!("missing {method}"))
    };
    let started = find("item/started")?;
    let completed_item = find("item/completed")?;
    assert_eq!(started["params"]["item"]["server"], "workspace_tools");
    assert_eq!(started["params"]["item"]["tool"], "lookup");
    assert_eq!(started["params"]["item"]["status"], "inProgress");
    assert_eq!(completed_item["params"]["item"]["status"], "completed");
    assert_eq!(
        completed_item["params"]["item"]["arguments"],
        json!({"query":"record-1"})
    );
    assert_eq!(
        completed_item["params"]["item"]["result"]["content"][0]["text"],
        MCP_RESULT
    );

    let requests = workspace
        .received_requests()
        .await
        .context("MCP requests")?;
    assert_eq!(calls(&requests), 1);
    for request in &requests {
        assert_eq!(
            request.headers.get("authorization").context("bearer")?,
            format!("Bearer {MCP_TOKEN}").as_str()
        );
        anyhow::ensure!(
            request.headers.get("x-portkey-api-key").is_none(),
            "inference credential sent to MCP server"
        );
    }
    let inference = gateway
        .received_requests()
        .await
        .context("model requests")?;
    anyhow::ensure!(
        inference
            .iter()
            .any(|request| String::from_utf8_lossy(&request.body).contains(MCP_RESULT)),
        "MCP result did not continue the model loop"
    );
    for request in &inference {
        assert_eq!(
            request.headers.get("x-portkey-api-key").context("key")?,
            KEY
        );
        anyhow::ensure!(
            request.headers.get("authorization").is_none()
                && !String::from_utf8_lossy(&request.body).contains(MCP_TOKEN),
            "MCP token reached inference"
        );
    }
    // No generated file in the runtime home or account directory carries the token.
    scan(temp.path(), &[KEY, MCP_TOKEN])?;
    let config = std::fs::read_dir(temp.path())?
        .flatten()
        .flat_map(|entry| walk(&entry.path()))
        .find(|path| path.file_name().is_some_and(|name| name == "config.toml"))
        .context("generated config")?;
    let config: toml::Value = toml::from_str(&std::fs::read_to_string(config)?)?;
    eprintln!("WIRE generated mcp_servers: {}", config["mcp_servers"]);
    assert_eq!(
        config["mcp_servers"]["workspace_tools"]["default_tools_approval_mode"].as_str(),
        Some("prompt")
    );
    assert_eq!(
        config["mcp_servers"]["workspace_tools"]["required"].as_bool(),
        Some(false)
    );
    assert_eq!(config["mcp_oauth_credentials_store"].as_str(), Some("file"));
    desktop.stop().await?;
    Ok(())
}

#[tokio::test]
async fn desktop_server_unavailable_workspace_mcp_never_blocks_inference() -> Result<()> {
    let gateway = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(completed()?, "text/event-stream"))
        .mount(&gateway)
        .await;
    // Every request is rejected as unauthenticated, like an expired Gateway token.
    let workspace = MockServer::start().await;
    Mock::given(path("/workspace/mcp"))
        .respond_with(
            ResponseTemplate::new(401)
                .insert_header("www-authenticate", "Bearer error=\"invalid_token\"")
                .set_body_json(json!({"error":"unauthorized"})),
        )
        .mount(&workspace)
        .await;
    let temp = tempfile::tempdir()?;
    let mut desktop = Desktop::launch(
        temp.path(),
        json!({
            "v":1,"tenantId":"11111111-1111-4111-8111-111111111111",
            "workspaceId":"22222222-2222-4222-8222-222222222222",
            "credentialId":"33333333-3333-4333-8333-333333333333","credentialEpoch":7,
            "gatewayEndpoint":format!("{}/v1", gateway.uri()),"apiKey":KEY,"projectPath":null,
            "localAccess":"readOnly","gatewayConfig":null,"model":null,
            "mcpServers":[{"name":"expired_tools","serverId":"srv-0002","url":format!("{}/workspace/mcp", workspace.uri()),"accessToken":MCP_TOKEN}]
        }),
    )
    .await?;
    let thread = desktop.thread().await?;
    desktop.send(json!({"id":10,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Inference continues","text_elements":[]}]}})).await?;
    anyhow::ensure!(
        desktop.response(10).await?.get("result").is_some(),
        "turn rejected"
    );
    let completed_turn = desktop.method("turn/completed").await?;
    assert_eq!(completed_turn["params"]["turn"]["status"], "completed");
    let failed = desktop.startup("expired_tools").await?;
    eprintln!("WIRE failed startup status: {failed}");
    assert_eq!(failed["params"]["status"], "failed");
    desktop
        .send(json!({"id":11,"method":"mcpServerStatus/list","params":{"threadId":thread}}))
        .await?;
    let status = desktop.response(11).await?;
    eprintln!("WIRE failed mcpServerStatus/list: {status}");
    assert_eq!(status["result"]["data"][0]["name"], "expired_tools");
    assert_eq!(status["result"]["data"][0]["authStatus"], "bearerToken");
    for request in workspace
        .received_requests()
        .await
        .context("MCP requests")?
    {
        anyhow::ensure!(
            !request.url.path().contains("well-known") && !request.url.path().contains("oauth"),
            "runtime attempted OAuth discovery or login"
        );
    }
    scan(temp.path(), &[KEY, MCP_TOKEN])?;
    desktop.stop().await?;
    Ok(())
}

#[tokio::test]
async fn desktop_server_rejects_remembered_or_form_mcp_approval_replies() -> Result<()> {
    for reply in [
        json!({"action":"accept","_meta":{"persist":"always"}}),
        json!({"action":"accept","content":{"mcp_tool_call_approval":"Allow for this session"}}),
        json!({"action":"approve"}),
    ] {
        let gateway = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/responses"))
            .respond_with(ModelFixture {
                turns: AtomicUsize::new(0),
            })
            .mount(&gateway)
            .await;
        let workspace = MockServer::start().await;
        Mock::given(path("/workspace/mcp"))
            .respond_with(McpFixture)
            .mount(&workspace)
            .await;
        let temp = tempfile::tempdir()?;
        let mut desktop = Desktop::launch(
            temp.path(),
            json!({
                "v":1,"tenantId":"11111111-1111-4111-8111-111111111111",
                "workspaceId":"22222222-2222-4222-8222-222222222222",
                "credentialId":"33333333-3333-4333-8333-333333333333","credentialEpoch":7,
                "gatewayEndpoint":format!("{}/v1", gateway.uri()),"apiKey":KEY,"projectPath":null,
                "localAccess":"readOnly","gatewayConfig":null,"model":null,
                "mcpServers":[{"name":"workspace_tools","serverId":"srv-0001","url":format!("{}/workspace/mcp", workspace.uri()),"accessToken":MCP_TOKEN}]
            }),
        )
        .await?;
        let thread = desktop.thread().await?;
        desktop.startup("workspace_tools").await?;
        desktop.send(json!({"id":10,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Look up record 0","text_elements":[]}]}})).await?;
        let approval = desktop.method("mcpServer/elicitation/request").await?;
        desktop
            .send(json!({"id":approval["id"],"result":reply}))
            .await?;
        drop(desktop.input);
        let status = tokio::time::timeout(Duration::from_secs(40), desktop.child.wait()).await??;
        anyhow::ensure!(
            !status.success(),
            "unsupported MCP approval reply did not retire the runtime"
        );
        for request in workspace
            .received_requests()
            .await
            .context("MCP requests")?
        {
            anyhow::ensure!(
                !String::from_utf8_lossy(&request.body).contains("tools/call"),
                "unsupported approval reply reached the MCP server"
            );
        }
        scan(temp.path(), &[KEY, MCP_TOKEN])?;
    }
    Ok(())
}

fn walk(path: &Path) -> Vec<std::path::PathBuf> {
    if path.is_dir() {
        std::fs::read_dir(path)
            .into_iter()
            .flatten()
            .flatten()
            .flat_map(|entry| walk(&entry.path()))
            .collect()
    } else {
        vec![path.to_path_buf()]
    }
}
