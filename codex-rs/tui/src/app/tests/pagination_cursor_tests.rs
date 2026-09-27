//! Exercise bounded metadata requests through the real app-server history API.
use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn metadata_pagination_stays_with_item_backed_turns() -> Result<()> {
    let (mut app, codex_home) = make_history_test_app().await?;
    app.local_settings.tui.terminal_resize_reflow_max_rows = Some(1);
    let thread_id =
        create_history_rollout(&app.config, ThreadHistoryMode::Paginated, "metadata bounds")?;
    let path = rollout_path(
        codex_home.path(),
        "2026-01-02T00-00-00",
        &thread_id.to_string(),
    );
    let mut records = std::fs::read_to_string(&path)?
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()?;
    for index in 0..12 {
        let turn_id = format!("metadata-turn-{index}");
        let events = [
            EventMsg::TurnStarted(TurnStartedEvent {
                turn_id: turn_id.clone(),
                trace_id: None,
                started_at: None,
                model_context_window: None,
                collaboration_mode_kind: Default::default(),
            }),
            EventMsg::ItemCompleted(ItemCompletedEvent {
                thread_id,
                turn_id,
                item: TurnItem::AgentMessage(AgentMessageItem {
                    id: format!("metadata-item-{index}"),
                    content: vec![AgentMessageContent::Text {
                        text: format!("metadata output {index}"),
                    }],
                    phase: None,
                    memory_citation: None,
                    delivery: None,
                    questions: None,
                }),
                started_at_ms: None,
                completed_at_ms: 0,
            }),
        ];
        for event in events {
            records.push(serde_json::json!({
                "timestamp": "2026-01-02T00:00:00Z", "ordinal": records.len(),
                "type": "event_msg", "payload": serde_json::to_value(event)?,
            }));
        }
    }
    std::fs::write(
        path,
        format!(
            "{}\n",
            records
                .into_iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        ),
    )?;
    let (mut server, requests, proxy) = start_recording_app_server(
        &app.config,
        /*blocked_thread_list*/ None,
        /*failed_thread_name*/ None,
    )
    .await?;
    let started = server
        .resume_thread(
            &app.local_settings,
            app.config.clone(),
            thread_id,
            crate::app_server_session::ResumeModelSettings::RestoreFromThread,
        )
        .await?;
    let mut turns = started.turns;
    let initial_requests = recorded_params(&requests, "thread/turns/list").len();
    let mut item_ids = Vec::new();
    // Fetch one output at a time past the initially loaded five turn headers.
    while let Some(cursor) = server.begin_older_history_page(thread_id) {
        let page = server
            .thread_items_page(
                thread_id,
                /*turn_id*/ None,
                Some(cursor.clone()),
                /*limit*/ 1,
            )
            .await?;
        let items = server
            .apply_older_history_page(thread_id, &cursor, page, &mut turns)
            .await?;
        item_ids.extend(items.into_iter().map(|item| item.id().to_string()));
    }
    let later_requests = recorded_params(&requests, "thread/turns/list");
    assert!(later_requests.len() > initial_requests);
    assert!(
        later_requests[initial_requests..]
            .iter()
            .all(|params| params["limit"] == 1)
    );
    assert!(item_ids.iter().any(|id| id == "metadata-item-0"));
    let unique = item_ids.iter().collect::<std::collections::HashSet<_>>();
    assert_eq!(unique.len(), item_ids.len());
    assert!(!server.has_older_history(thread_id));
    server.shutdown().await?;
    proxy.await??;
    Ok(())
}
