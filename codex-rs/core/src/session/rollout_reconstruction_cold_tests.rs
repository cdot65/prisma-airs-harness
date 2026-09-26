//! AIRS compatibility regressions for bounded checkpoint reconstruction.
use super::*;
use codex_protocol::openai_models::ReasoningEffort;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn rollback_after_current_checkpoint_restores_surviving_turn() {
    let (session, turn_context) = make_session_and_context().await;
    let previous_context = turn_context.to_turn_context_item();
    let kept = vec![
        RolloutItem::ResponseItem(user_message("keep this input").into()),
        RolloutItem::ResponseItem(assistant_message("keep this answer").into()),
    ];
    let mut items = completed_user_turn_rollout(previous_context, kept);
    let expected = session
        .reconstruct_history_from_rollout(&turn_context, &items)
        .await;
    let mut newer = turn_context.to_turn_context_item();
    newer.turn_id = Some("discarded-turn".into());
    newer.model = "discarded-model".into();
    items.extend(completed_user_turn_rollout(
        newer,
        vec![
            RolloutItem::ResponseItem(user_message("discard this input").into()),
            RolloutItem::Compacted(object!({
                "message": "discard this checkpoint",
                "replacement_history": [],
                "window_number": 1,
                "resume_metadata": {"previous_turn_settings": {"model": "discarded-model"}}
            })),
        ],
    ));
    items.push(RolloutItem::EventMsg(EventMsg::ThreadRolledBack(
        codex_protocol::protocol::ThreadRolledBackEvent { num_turns: 1 },
    )));
    let actual = session
        .reconstruct_history_from_rollout(&turn_context, &items)
        .await;
    assert_eq!(actual.history, expected.history);
    assert_eq!(
        actual.previous_turn_settings,
        expected.previous_turn_settings
    );
    assert_eq!(
        actual.reference_context_item,
        expected.reference_context_item
    );
}

#[tokio::test]
async fn interrupted_companion_preserves_authoritative_previous_settings() {
    let (session, turn_context) = make_session_and_context().await;
    let mut interrupted = turn_context.to_turn_context_item();
    interrupted.model = "interrupted-model".into();
    let items = vec![
        RolloutItem::Compacted(object!({
            "message": "summary", "replacement_history": [], "window_number": 1,
            "resume_metadata": {"previous_turn_settings": {"model": "completed-model"}}
        })),
        RolloutItem::WorldState(WorldStateItem::full(object!({}))),
        RolloutItem::TurnContext(interrupted.clone()),
    ];
    let actual = session
        .reconstruct_history_from_rollout(&turn_context, &items)
        .await;
    assert_eq!(
        actual.previous_turn_settings,
        Some(PreviousTurnSettings {
            model: "completed-model".into(),
            comp_hash: None,
            realtime_active: None,
        })
    );
    assert_eq!(actual.reference_context_item, Some(interrupted));
}

#[tokio::test]
async fn rollback_preserves_surviving_checkpoint_resume_settings() {
    let (session, turn_context) = make_session_and_context().await;
    let checkpoint = RolloutItem::Compacted(object!({
        "message": "summary", "replacement_history": [], "window_number": 1,
        "resume_metadata": {"previous_turn_settings": {"model": "completed-model"}}
    }));
    let mut newer = turn_context.to_turn_context_item();
    newer.turn_id = Some("rolled-back-turn".into());
    let mut items = vec![checkpoint];
    items.extend(completed_user_turn_rollout(
        newer,
        vec![RolloutItem::ResponseItem(
            user_message("discard this input").into(),
        )],
    ));
    items.push(RolloutItem::EventMsg(EventMsg::ThreadRolledBack(
        codex_protocol::protocol::ThreadRolledBackEvent { num_turns: 1 },
    )));
    let actual = session
        .reconstruct_history_from_rollout(&turn_context, &items)
        .await;
    assert_eq!(
        actual.previous_turn_settings,
        Some(PreviousTurnSettings {
            model: "completed-model".into(),
            comp_hash: None,
            realtime_active: None,
        })
    );
}

#[tokio::test]
async fn rollback_preserves_explicitly_empty_checkpoint_settings() {
    let (session, turn_context) = make_session_and_context().await;
    let mut old_context = turn_context.to_turn_context_item();
    old_context.model = "stale-pre-checkpoint-model".into();
    let mut items = completed_user_turn_rollout(old_context, Vec::new());
    let mut checkpoint_context = turn_context.to_turn_context_item();
    checkpoint_context.turn_id = Some("checkpoint-turn".into());
    let start = completed_user_turn_rollout(checkpoint_context, Vec::new())
        .into_iter()
        .next()
        .unwrap();
    items.push(start);
    items.push(RolloutItem::Compacted(object!({
        "message": "summary", "replacement_history": [], "window_number": 1,
        "resume_metadata": {}
    })));
    let mut newer = turn_context.to_turn_context_item();
    newer.turn_id = Some("rolled-back-turn".into());
    items.extend(completed_user_turn_rollout(
        newer,
        vec![RolloutItem::ResponseItem(
            user_message("discard this input").into(),
        )],
    ));
    items.push(RolloutItem::EventMsg(EventMsg::ThreadRolledBack(
        codex_protocol::protocol::ThreadRolledBackEvent { num_turns: 1 },
    )));
    let actual = session
        .reconstruct_history_from_rollout(&turn_context, &items)
        .await;
    assert_eq!(actual.previous_turn_settings, None);
    assert_eq!(actual.reference_context_item, None);
}

#[tokio::test]
async fn legacy_resume_mode_preserves_current_model_and_effort_overrides() {
    let (_session, turn_context) = make_session_and_context().await;
    let saved = CollaborationMode {
        mode: ModeKind::Plan,
        settings: Settings {
            model: "old-model".into(),
            reasoning_effort: Some(ReasoningEffort::High),
            developer_instructions: Some("saved plan instructions".into()),
        },
    };
    let mut context = turn_context.to_turn_context_item();
    context.collaboration_mode = Some(saved.clone());
    let items = vec![RolloutItem::TurnContext(context)];
    let configured = CollaborationMode {
        mode: ModeKind::Default,
        settings: Settings {
            model: "current-model".into(),
            reasoning_effort: None,
            developer_instructions: None,
        },
    };
    let resumed = InitialHistory::Resumed(ResumedHistory {
        conversation_id: ThreadId::default(),
        history: Arc::new(items.clone()),
        rollout_path: None,
    });
    assert_eq!(
        super::super::resume_settings::collaboration_mode(&resumed, configured.clone()),
        CollaborationMode {
            mode: saved.mode,
            settings: Settings {
                model: "current-model".into(),
                reasoning_effort: None,
                developer_instructions: saved.settings.developer_instructions,
            }
        }
    );
    // Fresh/forked sessions keep the caller's mode rather than inheriting a parent plan.
    for history in [InitialHistory::New, InitialHistory::Forked(items)] {
        assert_eq!(
            super::super::resume_settings::collaboration_mode(&history, configured.clone()),
            configured
        );
    }
}
