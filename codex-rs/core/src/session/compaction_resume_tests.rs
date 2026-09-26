//! Compaction snapshots resume metadata and the owning AIRS routing pair together.
use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn compaction_persists_resume_metadata_and_current_gateway_settings() {
    let (mut session, _, _rx) = make_session_and_context_with_auth_and_config_and_rx(
        CodexAuth::from_api_key("fixture-key"),
        Vec::new(),
        |config| {
            config.model_provider.gateway = Some(codex_model_provider_info::GatewayRouting {
                default_route: get_model_offline_for_tests(config.model.as_deref()),
            });
        },
    )
    .await;
    let path = attach_thread_persistence(Arc::get_mut(&mut session).unwrap()).await;
    let previous = PreviousTurnSettings {
        model: "previous-model".into(),
        comp_hash: Some("previous-producer".into()),
        realtime_active: None,
    };
    session
        .set_previous_turn_settings(Some(previous.clone()))
        .await;
    let runtime = session.set_multi_agent_version_if_unset(MultiAgentVersion::V2);
    let current = session
        .update_settings(SessionSettingsUpdate {
            step_settings: StepSettingsUpdate {
                gateway_config: Some(Some("pc-resume-fixture".into())),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap()
        .snapshot;
    let (window_number, window_ids) = session.advance_auto_compact_window().await;
    session
        .replace_compacted_history(
            vec![ResponseItemEnvelope::new(user_message("checkpoint"))],
            /*reference_context_item*/ None,
            /*world_state_baseline*/ None,
            CompactedHistoryMetadata {
                message: "summary".into(),
                window_number,
                window_ids,
                compaction_response_id: None,
                compaction_model_hash: None,
            },
        )
        .await;
    session.flush_rollout().await.unwrap();
    let (items, _, _) = RolloutRecorder::load_rollout_items(&path).await.unwrap();
    let index = items
        .iter()
        .rposition(|item| matches!(item, RolloutItem::Compacted(_)))
        .unwrap();
    let RolloutItem::Compacted(compacted) = &items[index] else {
        unreachable!()
    };
    assert_eq!(
        compacted.resume_metadata,
        Some(CompactionResumeMetadata {
            multi_agent_version: Some(runtime),
            last_started_turn_id: None,
            previous_turn_settings: Some(previous),
        })
    );
    let routing = items[index + 1..].iter().find_map(|item| match item {
        RolloutItem::EventMsg(EventMsg::ThreadSettingsApplied(event)) => {
            Some((event.thread_id, event.thread_settings.clone()))
        }
        _ => None,
    });
    assert_eq!(routing, Some((Some(session.thread_id), current)));
}
