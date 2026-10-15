use anyhow::Context;
use anyhow::Result;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::process::Child;
use tokio::process::ChildStdin;
use tokio::process::ChildStdout;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

const KEY: &str = r#"fixture-desktop-credential-"\\-never-in-context"#;

#[cfg(target_os = "macos")]
#[path = "desktop_server/shutdown.rs"]
mod shutdown;

#[cfg(target_os = "macos")]
#[path = "desktop_server/admission.rs"]
mod admission;

#[path = "desktop_server/workspace_mcp.rs"]
mod workspace_mcp;

struct Desktop {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    seen: Vec<Value>,
}

impl Desktop {
    async fn start(temp: &Path, endpoint: &str, project: Option<&Path>) -> Result<Self> {
        Self::start_access(temp, endpoint, project, "readOnly").await
    }

    async fn start_access(
        temp: &Path,
        endpoint: &str,
        project: Option<&Path>,
        access: &str,
    ) -> Result<Self> {
        Self::start_selection(temp, endpoint, project, access, None, None).await
    }

    async fn start_selection(
        temp: &Path,
        endpoint: &str,
        project: Option<&Path>,
        access: &str,
        gateway_config: Option<&str>,
        model: Option<&str>,
    ) -> Result<Self> {
        Self::launch(
            temp,
            json!({
                "v":1,"tenantId":"11111111-1111-4111-8111-111111111111",
                "workspaceId":"22222222-2222-4222-8222-222222222222",
                "credentialId":"33333333-3333-4333-8333-333333333333","credentialEpoch":7,
                "gatewayEndpoint":endpoint,"apiKey":KEY,"projectPath":project,
                "localAccess":access,"gatewayConfig":gateway_config,"model":model
            }),
        )
        .await
    }

    async fn launch(temp: &Path, bootstrap: Value) -> Result<Self> {
        let mut command =
            tokio::process::Command::new(codex_utils_cargo_bin::cargo_bin("airs-harness")?);
        command
            .args(["desktop-server", "--allow-http-loopback"])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", temp)
            .env("TMPDIR", temp)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command.spawn()?;
        let input = child.stdin.take().context("stdin")?;
        let output = BufReader::new(child.stdout.take().context("stdout")?);
        let mut desktop = Self {
            child,
            input,
            output,
            seen: Vec::new(),
        };
        desktop
            .send(json!({"id":0,"method":"desktop/bootstrap","params":bootstrap}))
            .await?;
        let response = desktop.response(0).await?;
        anyhow::ensure!(response.get("result").is_some(), "bootstrap rejected");
        desktop.send(json!({"id":1,"method":"initialize","params":{"clientInfo":{"name":"desktop_fixture","title":"fixture","version":"1"},"capabilities":{"experimentalApi":true}}})).await?;
        anyhow::ensure!(
            desktop.response(1).await?.get("result").is_some(),
            "initialize rejected"
        );
        desktop.send(json!({"method":"initialized"})).await?;
        Ok(desktop)
    }

    async fn send(&mut self, value: Value) -> Result<()> {
        self.input
            .write_all(format!("{value}\n").as_bytes())
            .await?;
        self.input.flush().await?;
        Ok(())
    }

    async fn next(&mut self) -> Result<Value> {
        let mut line = String::new();
        anyhow::ensure!(
            tokio::time::timeout(Duration::from_secs(30), self.output.read_line(&mut line))
                .await??
                > 0,
            "desktop EOF"
        );
        anyhow::ensure!(
            line.len() <= 4 * 1024 * 1024 && self.seen.len() < 512,
            "desktop output bound"
        );
        anyhow::ensure!(!line.contains(KEY), "credential leaked to transport");
        let value: Value = serde_json::from_str(&line)?;
        anyhow::ensure!(
            !value.to_string().contains(&serde_json::to_string(KEY)?),
            "escaped credential leaked to transport"
        );
        self.seen.push(value.clone());
        Ok(value)
    }

    async fn response(&mut self, id: i64) -> Result<Value> {
        loop {
            let value = self.next().await?;
            if value.get("id") == Some(&json!(id)) {
                return Ok(value);
            }
        }
    }

    async fn thread(&mut self) -> Result<String> {
        self.send(json!({"id":2,"method":"thread/start","params":{"ephemeral":true,"approvalPolicy":"on-request","approvalsReviewer":"user","experimentalRawEvents":true}})).await?;
        let response = self.response(2).await?;
        response["result"]["thread"]["id"]
            .as_str()
            .map(str::to_owned)
            .context("thread/start rejected")
    }

    async fn stop(mut self) -> Result<()> {
        drop(self.input);
        anyhow::ensure!(
            tokio::time::timeout(Duration::from_secs(40), self.child.wait())
                .await??
                .success(),
            "desktop shutdown failed"
        );
        Ok(())
    }
}

#[tokio::test]
async fn desktop_server_rejects_oversized_native_tool_reply_before_model_continuation() -> Result<()>
{
    let server = MockServer::start().await;
    let tool = sse(vec![
        json!({"type":"response.created","response":{"id":"bounded-tool-response"}}),
        json!({"type":"response.output_item.done","item":{"type":"function_call","id":"bounded-tool-item","call_id":"bounded-native-call","name":"airs_desktop__inventory","arguments":"{}"}}),
        json!({"type":"response.completed","response":{"id":"bounded-tool-response","usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
    ])?;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(tool, "text/event-stream"))
        .with_priority(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(completed()?, "text/event-stream"))
        .with_priority(2)
        .mount(&server)
        .await;
    let temp = tempfile::tempdir()?;
    let mut desktop = Desktop::start(temp.path(), &format!("{}/v1", server.uri()), None).await?;
    desktop.send(json!({"id":2,"method":"thread/start","params":{
        "ephemeral":true,"approvalPolicy":"on-request","approvalsReviewer":"user",
        "dynamicTools":[{"type":"namespace","name":"airs_desktop","description":"Native AIRS host tools","tools":[{"type":"function","name":"inventory","description":"Read native Runtime inventory","inputSchema":{"type":"object","properties":{},"additionalProperties":false}}]}]
    }})).await?;
    let thread = desktop.response(2).await?["result"]["thread"]["id"]
        .as_str()
        .context("bounded tool thread")?
        .to_owned();
    desktop.send(json!({"id":3,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Read native inventory","text_elements":[]}]}})).await?;
    desktop.response(3).await?;
    let mut call = desktop
        .seen
        .iter()
        .find(|v| v["method"] == "item/tool/call")
        .cloned();
    while call.is_none() {
        let value = desktop.next().await?;
        if value["method"] == "item/tool/call" {
            call = Some(value);
        }
    }
    let call = call.context("bounded native tool call")?;
    let reply = json!({"id":call["id"],"result":{"contentItems":[{"type":"inputText","text":format!("OVERSIZED_NATIVE_TOOL_OUTPUT{}","x".repeat(33*1024))}],"success":true}});
    anyhow::ensure!(
        reply.to_string().len() < 64 * 1024,
        "fixture must exercise the inner text limit"
    );
    desktop.send(reply).await?;
    drop(desktop.input);
    let status = tokio::time::timeout(Duration::from_secs(40), desktop.child.wait()).await??;
    anyhow::ensure!(
        !status.success(),
        "oversized ordinary reply did not close the owned runtime"
    );
    for request in server
        .received_requests()
        .await
        .context("bounded fixture requests")?
    {
        anyhow::ensure!(
            !String::from_utf8_lossy(&request.body).contains("OVERSIZED_NATIVE_TOOL_OUTPUT"),
            "oversized native tool output reached the model"
        );
    }
    scan(temp.path(), &[KEY, "OVERSIZED_NATIVE_TOOL_OUTPUT"])?;
    Ok(())
}

#[tokio::test]
async fn desktop_server_dynamic_airs_tool_roundtrip_keeps_key_out_of_model_context() -> Result<()> {
    let server = MockServer::start().await;
    let tool = sse(vec![
        json!({"type":"response.created","response":{"id":"native-tool-response"}}),
        json!({"type":"response.output_item.done","item":{"type":"function_call","id":"native-tool-item","call_id":"native-inventory-call","name":"airs_desktop__inventory","arguments":"{\"product\":\"runtime\",\"resource\":\"profiles\"}"}}),
        json!({"type":"response.completed","response":{"id":"native-tool-response","usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
    ])?;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(tool, "text/event-stream"))
        .with_priority(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(completed()?, "text/event-stream"))
        .with_priority(2)
        .mount(&server)
        .await;
    let temp = tempfile::tempdir()?;
    let mut desktop = Desktop::start(temp.path(), &format!("{}/v1", server.uri()), None).await?;
    desktop.send(json!({"id":2,"method":"thread/start","params":{
        "ephemeral":true,"approvalPolicy":"on-request","approvalsReviewer":"user",
        "dynamicTools":[{"type":"namespace","name":"airs_desktop","description":"Native AIRS host tools","tools":[{"type":"function","name":"inventory","description":"Read native Runtime inventory","inputSchema":{"type":"object","properties":{"product":{"const":"runtime"},"resource":{"const":"profiles"}},"required":["product","resource"],"additionalProperties":false}}]}]
    }})).await?;
    let thread = desktop.response(2).await?["result"]["thread"]["id"]
        .as_str()
        .context("native tool thread")?
        .to_owned();
    desktop.send(json!({"id":3,"method":"turn/start","params":{
        "threadId":thread,"input":[{"type":"text","text":format!("Inspect native Runtime profiles. Accidental protected echo: {KEY}"),"text_elements":[]}]
    }})).await?;
    let turn = desktop.response(3).await?["result"]["turn"]["id"]
        .as_str()
        .context("native tool turn")?
        .to_owned();
    let mut call = desktop
        .seen
        .iter()
        .find(|value| value["method"] == "item/tool/call")
        .cloned();
    while call.is_none() {
        let value = desktop.next().await?;
        if value["method"] == "item/tool/call" {
            call = Some(value);
        }
    }
    let call = call.context("native host call")?;
    assert_eq!(
        call["params"],
        json!({"threadId":thread,"turnId":turn,
        "callId":"native-inventory-call","namespace":"airs_desktop","tool":"inventory",
        "arguments":{"product":"runtime","resource":"profiles"}})
    );
    let quoted = serde_json::to_string(KEY)?;
    let mut unicode_echo = format!(
        r#"{{"\u0066{}":"OFFLINE_UNICODE_HOST_RESULT"}}"#,
        &quoted[2..quoted.len() - 1]
    );
    // Eight wrappers plus the enclosing serialized host JSON require more
    // than the old eight decoded-JSON layers. The complete text stays below
    // the inherited function-output truncation boundary.
    for _ in 0..8 {
        unicode_echo = serde_json::to_string(&unicode_echo)?;
    }
    let echoed = serde_json::to_string(
        &json!({"marker":"OFFLINE_HOST_READ_RESULT","safe_label":KEY,KEY:"accidental host key","unicode_echo":unicode_echo}),
    )?;
    assert!(
        echoed.len() < 10_000,
        "entire fixture must survive output truncation"
    );
    desktop.send(json!({"id":call["id"],"result":{"contentItems":[{"type":"inputText","text":echoed}],"success":true}})).await?;
    loop {
        if desktop.next().await?["method"] == "turn/completed" {
            break;
        }
    }
    let requests = server
        .received_requests()
        .await
        .context("fixture requests")?;
    assert_eq!(requests.len(), 2);
    for request in &requests {
        assert_eq!(
            request
                .headers
                .get("x-portkey-api-key")
                .context("memory credential header")?,
            KEY
        );
        let body = String::from_utf8_lossy(&request.body);
        anyhow::ensure!(
            !body.contains("fixture-desktop-credential"),
            "protected inference identity entered model context"
        );
    }
    let first: Value = requests[0].body_json()?;
    anyhow::ensure!(
        first["tools"]
            .as_array()
            .context("model tool catalog")?
            .iter()
            .any(|tool| tool["type"] == "function" && tool["name"] == "airs_desktop__inventory"),
        "native tool missing from model tools"
    );
    let next: Value = requests[1].body_json()?;
    let output = next["input"]
        .as_array()
        .context("next model input")?
        .iter()
        .find(|item| {
            item["type"] == "function_call_output" && item["call_id"] == "native-inventory-call"
        })
        .context("host output did not continue the actual model loop")?;
    let output = output.to_string();
    fn check_decoded(value: &Value) -> Result<()> {
        match value {
            Value::String(text) => {
                anyhow::ensure!(
                    !text.contains(KEY),
                    "decoded host credential entered model context"
                );
                if let Ok(decoded) = serde_json::from_str::<Value>(text) {
                    check_decoded(&decoded)?;
                }
            }
            Value::Array(values) => {
                for value in values {
                    check_decoded(value)?;
                }
            }
            Value::Object(values) => {
                for (key, value) in values {
                    check_decoded(&Value::String(key.clone()))?;
                    check_decoded(value)?;
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
        Ok(())
    }
    check_decoded(&serde_json::from_str::<Value>(&output)?)?;
    anyhow::ensure!(
        output.contains("OFFLINE_HOST_READ_RESULT")
            && output.contains("OFFLINE_UNICODE_HOST_RESULT")
            && output.contains("[protected credential]"),
        "host result/redaction missing from next model request"
    );
    scan(
        temp.path(),
        &[KEY, "OFFLINE_HOST_READ_RESULT", "Offline fixture completed"],
    )?;
    desktop.stop().await?;
    Ok(())
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn desktop_server_approved_commands_stay_inside_selected_project() -> Result<()> {
    for access in ["readOnly", "workspaceWrite"] {
        let temp = tempfile::tempdir_in("/private/tmp")?;
        let project = temp.path().join("project");
        std::fs::create_dir(&project)?;
        let outside = temp.path().join("private-outside-sentinel");
        let escaped = temp.path().join("must-not-be-created");
        std::fs::write(&outside, "OUTSIDE_SECRET_SENTINEL")?;
        std::fs::write(project.join("allowed.txt"), "INSIDE_READ_CONFIRMED")?;
        let script = format!(
            "IFS= read -r inside < allowed.txt; printf '%s\\n' \"$inside\"; if IFS= read -r outside < '{}' 2>/dev/null; then echo OUTSIDE_READ_BREACH; else echo OUTSIDE_READ_DENIED; fi; if echo escaped > '{}' 2>/dev/null; then echo OUTSIDE_WRITE_BREACH; else echo OUTSIDE_WRITE_DENIED; fi; if echo local > local-write.txt 2>/dev/null; then echo PROJECT_WRITE_ALLOWED; else echo PROJECT_WRITE_DENIED; fi; export -p",
            outside.display(),
            escaped.display(),
        );
        let command = sse(vec![
            json!({"type":"response.created","response":{"id":"command-response"}}),
            json!({"type":"response.output_item.done","item":{"type":"function_call","id":"command-item","call_id":"scope-probe","name":"exec_command","arguments":json!({"cmd":script,"yield_time_ms":1000,"max_output_tokens":2000,"login":false}).to_string()}}),
            json!({"type":"response.completed","response":{"id":"command-response","usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
        ])?;
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/responses"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(command, "text/event-stream"))
            .with_priority(1)
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v1/responses"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(completed()?, "text/event-stream"),
            )
            .with_priority(2)
            .mount(&server)
            .await;
        let mut desktop = Desktop::start_access(
            temp.path(),
            &format!("{}/v1", server.uri()),
            Some(&project),
            access,
        )
        .await?;
        let thread = desktop.thread().await?;
        desktop.send(json!({"id":20,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Run the local scope probe","text_elements":[]}]}})).await?;
        anyhow::ensure!(
            desktop.response(20).await?.get("result").is_some(),
            "turn rejected"
        );
        let mut approved = false;
        loop {
            let event = desktop.next().await?;
            if event["method"] == "item/commandExecution/requestApproval" {
                approved = true;
                desktop
                    .send(json!({"id":event["id"],"result":{"decision":"accept"}}))
                    .await?;
            }
            if event["method"] == "turn/completed" {
                break;
            }
        }
        anyhow::ensure!(approved, "actual local command approval missing");
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
        let output = String::from_utf8_lossy(&requests[1].body);
        let response: Value = serde_json::from_slice(&requests[1].body)?;
        let command_output = response["input"]
            .as_array()
            .context("fixture input")?
            .iter()
            .filter(|item| item["type"] == "function_call_output")
            .cloned()
            .collect::<Vec<_>>();
        for marker in [
            "INSIDE_READ_CONFIRMED",
            "OUTSIDE_READ_DENIED",
            "OUTSIDE_WRITE_DENIED",
        ] {
            anyhow::ensure!(
                output.contains(marker),
                "missing command probe evidence: {marker}; fixture command output={command_output:?}"
            );
        }
        anyhow::ensure!(
            !output.contains("OUTSIDE_SECRET_SENTINEL") && !output.contains(KEY),
            "private data escaped command boundary"
        );
        anyhow::ensure!(!escaped.exists(), "unrelated temporary file was writable");
        assert_eq!(
            project.join("local-write.txt").exists(),
            access == "workspaceWrite"
        );
        let marker = if access == "workspaceWrite" {
            "PROJECT_WRITE_ALLOWED"
        } else {
            "PROJECT_WRITE_DENIED"
        };
        anyhow::ensure!(output.contains(marker), "project authority probe missing");
        desktop.stop().await?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn desktop_server_model_escalation_cannot_escape_project_after_native_accept() -> Result<()> {
    let temp = tempfile::tempdir_in("/private/tmp")?;
    let project = temp.path().join("project");
    std::fs::create_dir(&project)?;
    let outside = temp.path().join("must-not-be-created-by-escalation");
    let script = format!(
        "if echo breach > '{}'; then echo ESCALATION_OUTSIDE_BREACH; else echo ESCALATION_OUTSIDE_DENIED; fi",
        outside.display()
    );
    let command = sse(vec![
        json!({"type":"response.created","response":{"id":"escalation-response"}}),
        json!({"type":"response.output_item.done","item":{"type":"function_call","id":"escalation-item","call_id":"escalation-probe","name":"exec_command","arguments":json!({"cmd":script,"sandbox_permissions":"require_escalated","justification":"Fixture asks for broader access","login":false}).to_string()}}),
        json!({"type":"response.completed","response":{"id":"escalation-response","usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
    ])?;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(command, "text/event-stream"))
        .with_priority(1)
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(completed()?, "text/event-stream"))
        .with_priority(2)
        .mount(&server)
        .await;
    let mut desktop = Desktop::start_access(
        temp.path(),
        &format!("{}/v1", server.uri()),
        Some(&project),
        "workspaceWrite",
    )
    .await?;
    let thread = desktop.thread().await?;
    desktop.send(json!({"id":20,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Probe denied escalation","text_elements":[]}]}})).await?;
    anyhow::ensure!(
        desktop.response(20).await?.get("result").is_some(),
        "turn rejected"
    );
    loop {
        let event = desktop.next().await?;
        if event["method"] == "item/commandExecution/requestApproval" {
            desktop
                .send(json!({"id":event["id"],"result":{"decision":"accept"}}))
                .await?;
        }
        if event["method"] == "turn/completed" {
            break;
        }
    }
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 2);
    let request: Value = requests[1].body_json()?;
    let output = request["input"]
        .as_array()
        .context("fixture input")?
        .iter()
        .find(|item| item["type"] == "function_call_output")
        .context("escalation result")?["output"]
        .as_str()
        .context("escalation output")?;
    anyhow::ensure!(
        !output.contains("ESCALATION_OUTSIDE_BREACH")
            && (output.contains("ESCALATION_OUTSIDE_DENIED") || output.contains("escalat")),
        "missing denied escalation evidence: {output}"
    );
    anyhow::ensure!(
        !outside.exists(),
        "native accept broadened project permissions"
    );
    desktop.stop().await?;
    Ok(())
}

fn sse(items: Vec<Value>) -> Result<String> {
    items
        .into_iter()
        .map(|item| {
            Ok(format!(
                "event: {}\ndata: {item}\n\n",
                item["type"].as_str().context("SSE fixture event type")?
            ))
        })
        .collect()
}

fn completed() -> Result<String> {
    sse(vec![
        json!({"type":"response.created","response":{"id":"fixture-response"}}),
        json!({"type":"response.output_item.done","item":{"id":"fixture-message","type":"message","role":"assistant","content":[{"type":"output_text","text":"Offline fixture completed","annotations":[]}]}}),
        json!({"type":"response.completed","response":{"id":"fixture-response","usage":{"input_tokens":1,"output_tokens":1,"total_tokens":2}}}),
    ])
}

fn scan(path: &Path, forbidden: &[&str]) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            scan(&entry.path(), forbidden)?;
        } else if entry.file_type()?.is_file() {
            let bytes = std::fs::read(entry.path())?;
            let text = String::from_utf8_lossy(&bytes);
            anyhow::ensure!(
                !forbidden.iter().any(|marker| text.contains(marker)),
                "plaintext marker in owned runtime files"
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn desktop_server_preserves_selected_routing_and_passively_restores_project_history()
-> Result<()> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(completed()?, "text/event-stream"))
        .mount(&server)
        .await;
    let temp = tempfile::tempdir()?;
    let project = temp.path().join("project");
    std::fs::create_dir(&project)?;
    let endpoint = format!("{}/v1", server.uri());
    let selected_model = "@fixture/selected-model";
    let selected_config = "pc-fixture-selected";
    let mut desktop = Desktop::start_selection(
        temp.path(),
        &endpoint,
        Some(&project),
        "readOnly",
        Some(selected_config),
        Some(selected_model),
    )
    .await?;
    let thread = desktop.thread().await?;
    desktop.send(json!({"id":20,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"PROTECTED_SELECTED_ROUTE_HISTORY","text_elements":[]}]}})).await?;
    anyhow::ensure!(
        desktop.response(20).await?.get("result").is_some(),
        "turn rejected"
    );
    while desktop.next().await?["method"] != "turn/completed" {}
    desktop
        .send(json!({"id":21,"method":"desktop/history/read","params":{"threadId":thread}}))
        .await?;
    let response = desktop.response(21).await?;
    let mut envelope = response
        .get("result")
        .cloned()
        .context("project RAM export failed")?;
    envelope["dynamicTools"] = json!([]);
    desktop.stop().await?;
    let mut desktop = Desktop::start_selection(
        temp.path(),
        &endpoint,
        Some(&project),
        "readOnly",
        Some(selected_config),
        Some(selected_model),
    )
    .await?;
    desktop
        .send(json!({"id":22,"method":"desktop/history/restore","params":envelope}))
        .await?;
    let restored = desktop.response(22).await?;
    anyhow::ensure!(
        restored.get("result").is_some(),
        "project protected restore failed: {restored}"
    );
    assert_eq!(restored["result"]["thread"]["id"], thread);
    assert_eq!(restored["result"]["thread"]["turns"], json!([]));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
    desktop.send(json!({"id":23,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"Continue the protected conversation","text_elements":[]}]}})).await?;
    anyhow::ensure!(
        desktop.response(23).await?.get("result").is_some(),
        "restored turn rejected"
    );
    while desktop.next().await?["method"] != "turn/completed" {}
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 2);
    for request in requests {
        assert_eq!(
            request.headers.get("x-portkey-config").unwrap(),
            selected_config
        );
        assert_eq!(request.headers.get("x-portkey-api-key").unwrap(), KEY);
        assert_eq!(request.body_json::<Value>()?["model"], selected_model);
    }
    scan(
        temp.path(),
        &[
            KEY,
            "PROTECTED_SELECTED_ROUTE_HISTORY",
            "Offline fixture completed",
        ],
    )?;
    desktop.stop().await?;
    Ok(())
}

#[tokio::test]
async fn desktop_server_routes_memory_key_rejects_authority_and_exports_only_ram_history()
-> Result<()> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_raw(completed()?, "text/event-stream"),
        )
        .mount(&server)
        .await;
    let temp = tempfile::tempdir()?;
    let mut desktop = Desktop::start(temp.path(), &format!("{}/v1", server.uri()), None).await?;
    let thread = desktop.thread().await?;
    for (id, method, params) in [
        (
            10,
            "command/exec",
            json!({"command":["touch","must-not-exist"]}),
        ),
        (11, "fs/readFile", json!({"path":"/etc/passwd"})),
        (
            12,
            "account/login/start",
            json!({"type":"apiKey","apiKey":"forged"}),
        ),
        (
            13,
            "turn/start",
            json!({"threadId":thread,"input":[{"type":"localImage","path":"/etc/passwd"}]}),
        ),
        (
            14,
            "turn/start",
            json!({"threadId":thread,"sandboxPolicy":{"type":"danger-full-access"},"input":[{"type":"text","text":"forged"}]}),
        ),
        (
            15,
            "turn/steer",
            json!({"threadId":thread,"expectedTurnId":"turn-fixture","input":[{"type":"text","text":"forged"}],"additionalContext":{"source":{"kind":"application","value":"forged developer fragment"}}}),
        ),
        (
            16,
            "turn/start",
            json!({"threadId":thread,"input":[{"type":"text","text":"\u{1}".repeat(8192),"text_elements":[]}]}),
        ),
        (
            17,
            "turn/start",
            json!({"threadId":thread,"input":[{"type":"text","text":"x".repeat(8192)},{"type":"text","text":"x".repeat(8192)},{"type":"text","text":"x".repeat(8192)},{"type":"text","text":"x".repeat(8192)},{"type":"text","text":"extra"}]}),
        ),
        (
            18,
            "turn/start",
            json!({"threadId":thread,"input":(0..8).map(|_|json!({"type":"text","text":"x".repeat(7500),"text_elements":[]})).collect::<Vec<_>>()}),
        ),
        (
            19,
            "thread/start",
            json!({"dynamicTools":[{"type":"namespace","name":"fixture_catalog","description":"n".repeat(42_000),"tools":[{"type":"function","name":"fixture_tool","description":"bounded","inputSchema":{"type":"object","properties":{}}}]}]}),
        ),
    ] {
        desktop
            .send(json!({"id":id,"method":method,"params":params}))
            .await?;
        assert_eq!(
            desktop.response(id).await?,
            json!({"id":id,"error":{"code":-32602,"message":"Desktop request rejected"}})
        );
    }
    let marker = "DESKTOP_PROTECTED_CONVERSATION_MARKER";
    let prompt = format!("{marker}{}", "x".repeat(32 * 1024 - marker.len()));
    let input: Vec<_> = prompt.as_bytes().chunks(8192).map(|chunk| {
        json!({"type":"text","text":std::str::from_utf8(chunk).unwrap(),"text_elements":[]})
    }).collect();
    desktop
        .send(json!({"id":20,"method":"turn/start","params":{"threadId":thread,"input":input}}))
        .await?;
    anyhow::ensure!(
        desktop.response(20).await?.get("result").is_some(),
        "turn rejected"
    );
    loop {
        if desktop.next().await?["method"] == "turn/completed" {
            break;
        }
    }
    desktop
        .send(json!({"id":21,"method":"desktop/history/read","params":{"threadId":thread}}))
        .await?;
    let history = desktop.response(21).await?;
    anyhow::ensure!(
        history["result"]["replay"]["history"].is_array(),
        "RAM history missing: {history}"
    );
    let requests = server
        .received_requests()
        .await
        .context("fixture requests")?;
    assert_eq!(requests.len(), 1, "rejected turns must not reach inference");
    let model_request: Value = serde_json::from_slice(&requests[0].body)?;
    let delivered = model_request["input"]
        .as_array()
        .context("model input")?
        .iter()
        .filter(|item| item["role"] == "user")
        .flat_map(|item| item["content"].as_array().into_iter().flatten())
        .filter_map(|content| content["text"].as_str())
        .collect::<String>();
    assert!(
        delivered.contains(&prompt),
        "the complete 32KiB prompt must survive admission"
    );
    for (index, (field, changed)) in [
        ("tenantId", json!("44444444-4444-4444-8444-444444444444")),
        ("workspaceId", json!("44444444-4444-4444-8444-444444444444")),
        (
            "credentialId",
            json!("44444444-4444-4444-8444-444444444444"),
        ),
        ("credentialEpoch", json!(8)),
        ("gatewayEndpoint", json!("https://other.example/v1")),
        ("projectPath", json!("/private/tmp")),
        ("localAccess", json!("workspaceWrite")),
        ("gatewayConfig", json!("pc-forged")),
        ("model", json!("@forged/model")),
    ]
    .into_iter()
    .enumerate()
    {
        let id = 30 + index as i64;
        let mut replay = history["result"].clone();
        replay["dynamicTools"] = json!([]);
        replay["context"][field] = changed;
        desktop
            .send(json!({"id":id,"method":"desktop/history/restore","params":replay}))
            .await?;
        assert_eq!(
            desktop.response(id).await?,
            json!({"id":id,"error":{"code":-32602,"message":"Desktop request rejected"}})
        );
    }
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].headers.get("x-portkey-api-key").unwrap(), KEY);
    assert!(!String::from_utf8_lossy(&requests[0].body).contains(KEY));
    scan(
        temp.path(),
        &[
            KEY,
            "DESKTOP_PROTECTED_CONVERSATION_MARKER",
            "Offline fixture completed",
        ],
    )?;
    desktop.stop().await?;
    assert!(!temp.path().join(".airs-harness").exists());
    Ok(())
}

#[tokio::test]
async fn desktop_server_bounded_bootstrap_never_echoes_secret_or_creates_saved_credentials()
-> Result<()> {
    let temp = tempfile::tempdir()?;
    let mut command = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("airs-harness")?);
    let output = command
        .arg("desktop-server")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", temp.path())
        .env("TMPDIR", temp.path())
        .write_stdin(format!(
            "{{\"apiKey\":\"{KEY}\",\"padding\":\"{}\"}}\n",
            "x".repeat(65536)
        ))
        .output()?;
    assert!(!output.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout)?,
        json!({"id":0,"error":{"code":-32602,"message":"Desktop bootstrap rejected"}})
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains(KEY));
    assert!(!temp.path().join(".airs-harness").exists());
    Ok(())
}
