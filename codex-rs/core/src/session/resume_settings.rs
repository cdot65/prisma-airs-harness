//! Restore conversation mode without reviving stale startup model or permission settings.
use codex_history::InitialHistory;
use codex_history::RolloutItem;
use codex_protocol::config_types::CollaborationMode;
use codex_protocol::protocol::EventMsg;

pub(super) fn collaboration_mode(
    history: &InitialHistory,
    configured: CollaborationMode,
) -> CollaborationMode {
    let InitialHistory::Resumed(resumed) = history else {
        return configured;
    };
    let saved = resumed
        .history
        .iter()
        .rev()
        .find_map(|item| match item {
            RolloutItem::EventMsg(EventMsg::ThreadSettingsApplied(event))
                if event.thread_id == Some(resumed.conversation_id) =>
            {
                Some(&event.thread_settings.collaboration_mode)
            }
            _ => None,
        })
        .or_else(|| {
            resumed
                .history
                .iter()
                .rev()
                .find_map(|item| match item {
                    RolloutItem::TurnContext(context) => Some(context.collaboration_mode.as_ref()),
                    _ => None,
                })
                .flatten()
        });
    let Some(saved) = saved else {
        return configured;
    };
    saved.with_updates(
        Some(configured.settings.model),
        Some(configured.settings.reasoning_effort),
        /*developer_instructions*/ None,
    )
}
