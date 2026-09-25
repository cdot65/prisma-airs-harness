//! Gateway routing is transport metadata, including on remote compaction.
use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn compaction_uses_the_pinned_saved_gateway_config() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses/compact"))
        .respond_with(ResponseTemplate::new(/*status*/ 200).set_body_json(json!({"output": []})))
        .expect(/*requests*/ 1)
        .mount(&server)
        .await;
    let mut client = test_model_client(SessionSource::Cli);
    let mut provider = client.state.provider.info().clone();
    provider.base_url = Some(format!("{}/v1", server.uri()));
    provider.gateway = Some(codex_model_provider_info::GatewayRouting {
        default_route: test_model_info().slug,
    });
    Arc::get_mut(&mut client.state).unwrap().provider =
        create_model_provider(provider, /*auth_manager*/ None);
    let prompt = Prompt {
        gateway_config: Some("pc-compact".into()),
        input: vec![ResponseItem::Message {
            id: None,
            role: "user".into(),
            content: vec![ContentItem::InputText {
                text: "compact this".into(),
            }],
            phase: None,
            internal_chat_message_metadata_passthrough: None,
        }],
        ..Default::default()
    };
    let metadata = test_responses_metadata_for_client(
        &client,
        /*turn_id*/ None,
        format!("{}:0", client.state.thread_id),
        /*parent_thread_id*/ None,
        TestCodexResponsesRequestKind::Turn,
    );
    client
        .compact_conversation_history(
            &prompt,
            &test_model_info(),
            /*turn_state*/ None,
            CompactConversationRequestSettings {
                effort: None,
                summary: codex_protocol::config_types::ReasoningSummary::None,
                service_tier: None,
            },
            &test_session_telemetry(),
            &CompactionTraceContext::disabled(),
            &metadata,
        )
        .await?;
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].headers["x-portkey-config"], "pc-compact");
    let body: serde_json::Value = requests[0].body_json()?;
    assert_eq!(body.get("model"), None);
    assert!(!body.to_string().contains("pc-compact"));
    Ok(())
}
