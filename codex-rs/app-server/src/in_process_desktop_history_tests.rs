use super::*;
use crate::in_process::InProcessServerEvent;
use crate::in_process::InProcessStartArgs;
use crate::in_process::start_desktop_uninitialized;
use codex_app_server_protocol::ClientInfo;
use codex_app_server_protocol::ClientNotification;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::InitializeCapabilities;
use codex_app_server_protocol::InitializeParams;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::ThreadInjectItemsParams;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStatus;
use codex_app_server_protocol::UserInput;
use codex_arg0::Arg0DispatchPaths;
use codex_config::CloudConfigBundleLoader;
use codex_config::LoaderOverrides;
use codex_config::NoopThreadConfigLoader;
use codex_core::config::ConfigBuilder;
use codex_core::config::ConfigOverrides;
use codex_exec_server::EnvironmentManager;
use codex_feedback::CodexFeedback;
use codex_protocol::ThreadId;
use codex_protocol::dynamic_tools::DynamicToolFunctionSpec;
use codex_protocol::dynamic_tools::DynamicToolNamespaceSpec;
use codex_protocol::dynamic_tools::DynamicToolNamespaceTool;
use codex_protocol::dynamic_tools::DynamicToolSpec;
use codex_rollout::RolloutItem;
use codex_thread_store::AppendThreadItemsParams;
use codex_thread_store::InMemoryThreadStore;
use codex_thread_store::ThreadStore;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

fn tool(name: &str) -> DynamicToolSpec {
    DynamicToolSpec::Function(DynamicToolFunctionSpec {
        name: name.to_string(),
        description: "Admitted desktop fixture tool".to_string(),
        input_schema: json!({"type":"object","properties":{},"additionalProperties":false}),
        defer_loading: false,
    })
}

async fn desktop_client(
    home: &Path,
    cwd: &Path,
    server: &str,
) -> anyhow::Result<(InProcessClientHandle, String)> {
    let store_id = uuid::Uuid::now_v7().to_string();
    std::fs::write(
        home.join("config.toml"),
        format!(
            r#"
model = "mock-model"
model_provider = "desktop_fixture"
approval_policy = "never"
sandbox_mode = "workspace-write"
experimental_thread_store = {{ type = "in_memory", id = "{store_id}" }}
project_doc_max_bytes = 0
project_root_markers = []
[features]
skip_host_skill_discovery = true
hooks = false
shell_snapshot = false
shell_snapshot_v2 = false
plugins = false
[history]
persistence = "none"
[analytics]
enabled = false
[feedback]
enabled = false
[model_providers.desktop_fixture]
name = "Desktop fixture"
base_url = "{server}/v1"
wire_api = "responses"
requires_openai_auth = false
request_max_retries = 0
stream_max_retries = 0
supports_websockets = false
"#
        ),
    )?;
    let loader_overrides = LoaderOverrides::without_managed_config_for_tests();
    let config = ConfigBuilder::default()
        .codex_home(home.to_path_buf())
        .harness_overrides(ConfigOverrides {
            cwd: Some(cwd.to_path_buf()),
            ..Default::default()
        })
        .loader_overrides(loader_overrides.clone())
        .build()
        .await?;
    let initialize = InitializeParams {
        client_info: ClientInfo {
            name: "desktop-replay-test".into(),
            title: None,
            version: "0".into(),
        },
        capabilities: Some(InitializeCapabilities {
            experimental_api: true,
            ..Default::default()
        }),
    };
    let client = start_desktop_uninitialized(InProcessStartArgs {
        arg0_paths: Arg0DispatchPaths::default(),
        config: Arc::new(config),
        cli_overrides: Vec::new(),
        loader_overrides,
        strict_config: true,
        cloud_config_bundle: CloudConfigBundleLoader::default(),
        embedded_network_policy: Default::default(),
        thread_config_loader: Arc::new(NoopThreadConfigLoader),
        feedback: CodexFeedback::new(),
        log_db: None,
        state_db: None,
        environment_manager: Arc::new(EnvironmentManager::default_for_tests()),
        config_warnings: Vec::new(),
        session_source: codex_protocol::protocol::SessionSource::Exec,
        enable_codex_api_key_env: false,
        initialize: initialize.clone(),
        channel_capacity: 1024,
    })
    .await?;
    client
        .request(ClientRequest::Initialize {
            request_id: RequestId::Integer(0),
            params: initialize,
        })
        .await?
        .map_err(|_| anyhow::anyhow!("initialize failed"))?;
    client.notify(ClientNotification::Initialized)?;
    Ok((client, store_id))
}

#[tokio::test]
async fn desktop_replay_restores_compaction_into_model_context_without_executing_old_calls()
-> anyhow::Result<()> {
    let server = responses::start_mock_server().await;
    let response_mock = responses::mount_sse_once(
        &server,
        responses::sse(vec![
            responses::ev_response_created("desktop-resume-response"),
            responses::ev_assistant_message("desktop-resume-message", "Restored"),
            responses::ev_completed("desktop-resume-response"),
        ]),
    )
    .await;
    let temp = tempfile::tempdir()?;
    let home = temp.path().join("home");
    let restored_home = temp.path().join("restored-home");
    let cwd = temp.path().join("workspace");
    std::fs::create_dir(&home)?;
    std::fs::create_dir(&restored_home)?;
    std::fs::create_dir(&cwd)?;
    for (root, name, contents) in [
        (
            home.join("skills"),
            "hostile-user",
            "HOSTILE_USER_SKILL_MARKER",
        ),
        (
            restored_home.join("skills"),
            "hostile-user",
            "HOSTILE_USER_SKILL_MARKER",
        ),
        (
            cwd.join(".agents/skills"),
            "hostile-project",
            "HOSTILE_PROJECT_SKILL_MARKER",
        ),
    ] {
        let package = root.join(name);
        std::fs::create_dir_all(&package)?;
        std::fs::write(
            package.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: {contents}\n---\n{contents}\n"),
        )?;
    }
    let marker = cwd.join("historical-tool-executed");
    // The historical command would be permitted here, so its missing marker proves passive replay.
    let (client, first_store_id) = desktop_client(&home, &cwd, &server.uri()).await?;
    let response = client
        .request(ClientRequest::ThreadStart {
            request_id: RequestId::Integer(1),
            params: ThreadStartParams {
                cwd: Some(cwd.to_string_lossy().into_owned()),
                dynamic_tools: Some(vec![tool("historical_desktop_catalog_tool")]),
                experimental_raw_events: true,
                ..Default::default()
            },
        })
        .await?
        .map_err(|_| anyhow::anyhow!("start failed"))?;
    let started: ThreadStartResponse = serde_json::from_value(response)?;
    let call = json!({"type":"function_call", "name":"exec_command", "arguments":
        serde_json::to_string(&json!({"cmd": format!("touch {}", marker.display())}))?, "call_id":"historical-call"});
    let output = json!({"type":"function_call_output", "call_id":"historical-call", "output":"HISTORICAL_TOOL_RESULT"});
    client.request(ClientRequest::ThreadInjectItems {
        request_id: RequestId::Integer(2),
        params: ThreadInjectItemsParams { thread_id: started.thread.id.clone(), items: vec![
            json!({"type":"message","role":"user","content":[{"type":"input_text","text":"OBSOLETE_BEFORE_COMPACTION"}]}),
            call.clone(), output.clone(),
        ] },
    }).await?.map_err(|_| anyhow::anyhow!("inject failed"))?;
    let replacement = json!({"type":"message","role":"user","content":[{"type":"input_text","text":"COMPACTED_DESKTOP_CONTEXT"}]});
    let mut bounded_replacement = replacement.clone();
    bounded_replacement["content"][0]["text"] = json!("");
    let empty_size = serde_json::to_vec(&serde_json::from_value::<
        codex_protocol::models::ResponseItem,
    >(bounded_replacement.clone())?)?
    .len();
    bounded_replacement["content"][0]["text"] = json!("x".repeat(40_000 - empty_size));
    assert_eq!(
        serde_json::to_vec(&serde_json::from_value::<
            codex_protocol::models::ResponseItem,
        >(bounded_replacement.clone())?)?
        .len(),
        40_000
    );
    let compacted = serde_json::from_value(json!({
        "message":"COMPACTED_DESKTOP_CONTEXT", "replacement_history":[replacement, bounded_replacement, call, output],
        "window_number":1, "window_id":uuid::Uuid::now_v7().to_string(),
    }))?;
    ThreadStore::append_items(
        InMemoryThreadStore::for_id(&first_store_id).as_ref(),
        AppendThreadItemsParams {
            thread_id: ThreadId::from_string(&started.thread.id)?,
            items: vec![RolloutItem::Compacted(compacted)],
        },
    )
    .await?;
    let replay = client
        .export_desktop_history(started.thread.id.clone())
        .await?
        .map_err(|error| anyhow::anyhow!(error.message))?;
    let original_history = serde_json::to_value(&replay.history)?;
    client.shutdown().await?;
    InMemoryThreadStore::remove_id(&first_store_id);
    let (mut restored_client, second_store_id) =
        desktop_client(&restored_home, &cwd, &server.uri()).await?;
    let empty_catalog_error = restored_client
        .restore_desktop_history(DesktopRestoreParams {
            replay: replay.clone(),
            current_tools: Vec::new(),
        })
        .await?
        .err()
        .ok_or_else(|| anyhow::anyhow!("empty current catalog revived saved tools"))?;
    assert_eq!(
        empty_catalog_error.message,
        "desktop replay requires a current nonempty tool catalog"
    );
    let current_tools = vec![DynamicToolSpec::Namespace(DynamicToolNamespaceSpec {
        name: "current_desktop_catalog".to_string(),
        description: "The currently admitted tool namespace".to_string(),
        tools: ["current_desktop_catalog_tool", "second_current_tool"]
            .into_iter()
            .map(|name| DynamicToolNamespaceTool::Function(DynamicToolFunctionSpec {
                name: name.to_string(),
                description: "Fixture namespace documentation. ".repeat(70),
                input_schema: json!({"type":"object","properties":{},"additionalProperties":false}),
                defer_loading: false,
            }))
            .collect(),
    })];
    assert!(serde_json::to_vec(&current_tools)?.len() > 4096);
    let mut oversized_catalog = current_tools.clone();
    let DynamicToolSpec::Namespace(namespace) = &mut oversized_catalog[0] else {
        unreachable!()
    };
    namespace.description = "n".repeat(42_000);
    assert!(serde_json::to_vec(&oversized_catalog)?.len() < 48 * 1024);
    let error = restored_client
        .restore_desktop_history(DesktopRestoreParams {
            replay: replay.clone(),
            current_tools: oversized_catalog,
        })
        .await?
        .err()
        .ok_or_else(|| anyhow::anyhow!("oversized namespace fragment was admitted"))?;
    assert_eq!(
        error.message,
        "desktop current tool catalog exceeds its bounds"
    );
    for (mutation, expected_error) in [
        (
            "provider",
            "desktop replay does not match the current root runtime",
        ),
        ("version", "desktop replay version is incompatible"),
        (
            "metadata",
            "desktop replay session metadata is inconsistent",
        ),
        (
            "permission",
            "desktop replay turn settings do not match the current runtime",
        ),
        (
            "response-size",
            "desktop replay context item exceeds its serialized byte bound",
        ),
        (
            "compaction-summary-size",
            "desktop replay context item exceeds its serialized byte bound",
        ),
        (
            "compaction-replacement-size",
            "desktop replay context item exceeds its serialized byte bound",
        ),
    ] {
        let mut changed = replay.clone();
        match mutation {
            "provider" => changed.create.metadata.model_provider = "other".into(),
            "version" => changed.schema_version = 2,
            "metadata" => changed.create.originator = "forged".into(),
            "permission" => {
                changed
                    .history
                    .push(RolloutItem::TurnContext(serde_json::from_value(json!({
                        "cwd":cwd,
                        "approval_policy":"never",
                        "sandbox_policy":{"type":"read-only"},
                        "permission_profile":codex_protocol::models::PermissionProfile::Disabled,
                        "model":"mock-model",
                        "summary":"auto"
                    }))?));
            }
            "response-size" => {
                changed.history.push(RolloutItem::ResponseItem(serde_json::from_value::<codex_protocol::models::ResponseItem>(json!({
                    "type":"message","role":"user","content":[{"type":"input_text","text":"x".repeat(50_000)}]
                }))?.into()));
            }
            "compaction-summary-size" => {
                changed
                    .history
                    .push(RolloutItem::Compacted(serde_json::from_value(json!({
                        "message":"x".repeat(50_000)
                    }))?));
            }
            "compaction-replacement-size" => {
                changed.history.push(RolloutItem::Compacted(serde_json::from_value(json!({
                    "message":"bounded summary", "replacement_history":[{
                        "type":"message","role":"user","content":[{"type":"input_text","text":"x".repeat(50_000)}]
                    }]
                }))?));
            }
            _ => unreachable!(),
        }
        let error = restored_client
            .restore_desktop_history(DesktopRestoreParams {
                replay: changed,
                current_tools: current_tools.clone(),
            })
            .await?
            .err()
            .ok_or_else(|| anyhow::anyhow!("mutated replay was accepted"))?;
        assert_eq!(error.message, expected_error);
    }
    let resumed = restored_client
        .restore_desktop_history(DesktopRestoreParams {
            replay: replay.clone(),
            current_tools: current_tools.clone(),
        })
        .await?
        .map_err(|error| anyhow::anyhow!(error.message))?;
    assert_eq!(resumed.thread.id, started.thread.id);
    assert_eq!(resumed.thread.path, None);
    assert!(!marker.exists());
    assert!(
        server
            .received_requests()
            .await
            .unwrap_or_default()
            .is_empty()
    );
    assert_eq!(
        serde_json::to_value(
            restored_client
                .export_desktop_history(resumed.thread.id.clone())
                .await?
                .map_err(|error| anyhow::anyhow!(error.message))?
                .history
        )?,
        original_history
    );
    restored_client
        .request(ClientRequest::TurnStart {
            request_id: RequestId::Integer(3),
            params: TurnStartParams {
                thread_id: resumed.thread.id.clone(),
                input: vec![UserInput::Text {
                    text: "Continue".into(),
                    text_elements: Vec::new(),
                }],
                ..Default::default()
            },
        })
        .await?
        .map_err(|_| anyhow::anyhow!("turn failed"))?;
    tokio::time::timeout(Duration::from_secs(15), async {
        while let Some(event) = restored_client.next_event().await {
            if let InProcessServerEvent::ServerNotification(notification) = event
                && let ServerNotification::TurnCompleted(completed) = *notification
            {
                assert_eq!(completed.turn.status, TurnStatus::Completed);
                return;
            }
        }
        panic!("restored turn produced no completion");
    })
    .await?;
    let model_request = response_mock.single_request().body_json();
    for item in model_request["input"].as_array().unwrap() {
        assert!(
            serde_json::to_vec(item)?.len() <= 40_000,
            "restored model-visible item exceeds its bound"
        );
    }
    let input = model_request["input"].to_string();
    assert!(input.contains("COMPACTED_DESKTOP_CONTEXT"));
    assert!(input.contains("HISTORICAL_TOOL_RESULT"));
    assert!(input.contains(&"x".repeat(40_000 - empty_size)));
    assert!(!input.contains("OBSOLETE_BEFORE_COMPACTION"));
    assert!(!input.contains("HOSTILE_USER_SKILL_MARKER"));
    assert!(!input.contains("HOSTILE_PROJECT_SKILL_MARKER"));
    let tools = response_mock.single_request().body_json()["tools"].to_string();
    assert!(tools.contains("current_desktop_catalog_tool"));
    assert!(!tools.contains("historical_desktop_catalog_tool"));
    assert!(!tools.contains("skills"));
    assert!(!marker.exists());
    let completed_replay = restored_client
        .export_desktop_history(resumed.thread.id)
        .await?
        .map_err(|error| anyhow::anyhow!(error.message))?;
    assert!(
        completed_replay
            .history
            .iter()
            .any(|item| matches!(item, RolloutItem::TurnContext(_)))
    );
    let serialized_history = serde_json::to_string(&completed_replay)?;
    assert!(!serialized_history.contains("HOSTILE_USER_SKILL_MARKER"));
    assert!(!serialized_history.contains("HOSTILE_PROJECT_SKILL_MARKER"));
    assert_no_plaintext(&home)?;
    assert_no_plaintext(&restored_home)?;
    restored_client.shutdown().await?;
    InMemoryThreadStore::remove_id(&second_store_id);
    Ok(())
}

fn assert_no_plaintext(root: &Path) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            assert_no_plaintext(&path)?;
        } else {
            // Authored hostile fixtures are inputs, never generated conversation artifacts.
            if path.file_name().is_some_and(|name| name == "SKILL.md") {
                continue;
            }
            let bytes = std::fs::read(path)?;
            assert!(
                !bytes
                    .windows(b"COMPACTED_DESKTOP_CONTEXT".len())
                    .any(|part| part == b"COMPACTED_DESKTOP_CONTEXT")
            );
            for marker in [
                b"HOSTILE_USER_SKILL_MARKER".as_slice(),
                b"HOSTILE_PROJECT_SKILL_MARKER".as_slice(),
            ] {
                assert!(!bytes.windows(marker.len()).any(|part| part == marker));
            }
            assert!(
                !bytes
                    .windows(b"HISTORICAL_TOOL_RESULT".len())
                    .any(|part| part == b"HISTORICAL_TOOL_RESULT")
            );
        }
    }
    Ok(())
}
