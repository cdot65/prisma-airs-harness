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
        let mut child =
            tokio::process::Command::new(codex_utils_cargo_bin::cargo_bin("airs-harness")?)
                .args(["desktop-server", "--allow-http-loopback"])
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("HOME", temp)
                .env("TMPDIR", temp)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .kill_on_drop(true)
                .spawn()?;
        let input = child.stdin.take().context("stdin")?;
        let output = BufReader::new(child.stdout.take().context("stdout")?);
        let mut desktop = Self {
            child,
            input,
            output,
            seen: Vec::new(),
        };
        desktop
            .send(json!({"id":0,"method":"desktop/bootstrap","params":{
                "v":1,"tenantId":"11111111-1111-4111-8111-111111111111",
                "workspaceId":"22222222-2222-4222-8222-222222222222",
                "credentialId":"33333333-3333-4333-8333-333333333333","credentialEpoch":7,
                "gatewayEndpoint":endpoint,"apiKey":KEY,"projectPath":project,
                "localAccess":access,"gatewayConfig":gateway_config,"model":model
            }}))
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
            "/bin/cat allowed.txt; if /bin/cat '{}' 2>/dev/null; then echo OUTSIDE_READ_BREACH; else echo OUTSIDE_READ_DENIED; fi; if echo escaped > '{}' 2>/dev/null; then echo OUTSIDE_WRITE_BREACH; else echo OUTSIDE_WRITE_DENIED; fi; if echo local > local-write.txt 2>/dev/null; then echo PROJECT_WRITE_ALLOWED; else echo PROJECT_WRITE_DENIED; fi; /usr/bin/env",
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
    ] {
        desktop
            .send(json!({"id":id,"method":method,"params":params}))
            .await?;
        assert_eq!(
            desktop.response(id).await?,
            json!({"id":id,"error":{"code":-32602,"message":"Desktop request rejected"}})
        );
    }
    desktop.send(json!({"id":20,"method":"turn/start","params":{"threadId":thread,"input":[{"type":"text","text":"DESKTOP_PROTECTED_CONVERSATION_MARKER","text_elements":[]}]}})).await?;
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
