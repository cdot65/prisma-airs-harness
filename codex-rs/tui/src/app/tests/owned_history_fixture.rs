//! Persisted two-page fixture for owned-history clearing and cancellation.
use super::*;
use app_test_support::create_fake_paginated_rollout;
use app_test_support::rollout_path;
use chrono::TimeZone;
use codex_protocol::items::AgentMessageContent;
use codex_protocol::items::AgentMessageItem;
use codex_protocol::items::TurnItem;
use codex_protocol::items::UserMessageItem;
use codex_protocol::protocol::ItemCompletedEvent;
use codex_protocol::protocol::TurnCompleteEvent;
use codex_protocol::user_input::UserInput as CoreUserInput;
use codex_state::SqliteConfig;
pub(super) async fn completed_history_app(
    names: &[&str],
) -> Result<(Box<App>, tempfile::TempDir, ThreadId)> {
    let mut app = make_test_app().await;
    let codex_home = tempdir()?;
    app.config.codex_home = codex_home.path().to_path_buf().abs();
    app.config.sqlite = SqliteConfig::new_for_testing(codex_home.path().abs());
    // The inline scrollback row cap fixes the page boundary used by this regression.
    app.local_settings.transcript_mode = crate::transcript_mode::TranscriptMode::Terminal;
    app.local_settings.tui.alternate_screen = codex_config::types::AltScreenMode::Never;
    app.local_settings.tui.terminal_resize_reflow_max_rows = Some(2);
    let completed_at = chrono::Local
        .with_ymd_and_hms(
            /*year*/ 2000, /*month*/ 9, /*day*/ 6, /*hour*/ 14,
            /*min*/ 32, /*sec*/ 0,
        )
        .single()
        .expect("unambiguous local completion time");
    let timestamp = completed_at.to_rfc3339();
    let filename_timestamp = "2000-09-06T14-32-00";
    let thread_id = create_fake_paginated_rollout(
        codex_home.path(),
        filename_timestamp,
        &timestamp,
        "completed pagination",
        Some(app.config.model_provider_id.as_str()),
        /*git_info*/ None,
    )
    .map_err(|error| color_eyre::eyre::eyre!(error))?;
    let path = rollout_path(codex_home.path(), filename_timestamp, &thread_id);
    let thread_id = ThreadId::from_string(&thread_id)?;
    let mut records = std::fs::read_to_string(&path)?
        .lines()
        .take(/*n*/ 1)
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()?;
    for (index, name) in names.iter().enumerate() {
        let turn_id = format!("turn-{index}");
        let finished = completed_at.timestamp() + index as i64 * 60;
        let mut events = vec![EventMsg::TurnStarted(TurnStartedEvent {
            turn_id: turn_id.clone(),
            trace_id: None,
            started_at: Some(finished - 125),
            model_context_window: None,
            collaboration_mode_kind: Default::default(),
        })];
        let items = [
            TurnItem::UserMessage(UserMessageItem {
                id: format!("prompt-{index}"),
                client_id: None,
                content: vec![CoreUserInput::Text {
                    text: format!("{name} prompt"),
                    text_elements: Vec::new(),
                }],
            }),
            TurnItem::AgentMessage(AgentMessageItem {
                id: format!("answer-{index}"),
                content: vec![AgentMessageContent::Text {
                    text: format!("{name} answer"),
                }],
                phase: None,
                memory_citation: None,
                delivery: None,
                questions: None,
            }),
        ];
        events.extend(items.into_iter().map(|item| {
            EventMsg::ItemCompleted(ItemCompletedEvent {
                thread_id,
                turn_id: turn_id.clone(),
                item,
                started_at_ms: None,
                completed_at_ms: finished * 1_000,
            })
        }));
        events.push(EventMsg::TurnComplete(TurnCompleteEvent {
            turn_id,
            last_agent_message: Some(format!("{name} answer")),
            error: None,
            started_at: Some(finished - 125),
            completed_at: Some(finished),
            duration_ms: Some(125_000),
            time_to_first_token_ms: None,
        }));
        for event in events {
            records.push(serde_json::json!({
                "timestamp": timestamp,
                "ordinal": records.len(),
                "type": "event_msg",
                "payload": event,
            }));
        }
    }
    let records = records
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(path, format!("{records}\n"))?;
    Ok((app, codex_home, thread_id))
}
