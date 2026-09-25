//! Restore only the owning thread's saved gateway selection, never copied history.
use crate::config::Config;
use codex_history::InitialHistory;
use codex_model_provider_info::GatewayRouting;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result;
use codex_protocol::protocol::EventMsg;
use codex_rollout::RolloutItem;

pub(super) fn restored_config(history: &InitialHistory, config: &Config) -> Result<Option<String>> {
    let (items, owner) = match history {
        InitialHistory::New | InitialHistory::Cleared => return Ok(None),
        InitialHistory::Resumed(resumed) => (resumed.history.as_slice(), resumed.conversation_id),
        InitialHistory::Forked(items) => {
            let Some(parent) = history.forked_from_id() else {
                return Ok(None);
            };
            (items.as_slice(), parent)
        }
    };
    let snapshot = items.iter().rev().find_map(|item| match item {
        RolloutItem::EventMsg(EventMsg::ThreadSettingsApplied(event))
            if event.thread_id == Some(owner) =>
        {
            Some(&event.thread_settings)
        }
        _ => None,
    });
    let Some(snapshot) = snapshot else {
        return Ok(None);
    };
    let Some(id) = &snapshot.gateway_config else {
        return Ok(None);
    };
    if config.model_provider.gateway.is_none()
        || snapshot.model_provider_id != config.model_provider_id
    {
        return Err(CodexErr::InvalidRequest("Saved gateway routing belongs to a different provider; resume with its original provider".into()));
    }
    GatewayRouting::validate_saved_config(id).map_err(CodexErr::InvalidRequest)?;
    Ok(Some(id.clone()))
}

/// Materialize the first real input with its current routing checkpoint.
/// Hold the settings permit through persistence so a concurrent edit cannot be lost.
pub(super) async fn persist_after_input(
    session: &super::Session,
    context: codex_thread_store::PersistContext,
) {
    use std::sync::atomic::Ordering;
    if session
        .gateway_settings_checkpointed
        .load(Ordering::Acquire)
    {
        session.ensure_rollout_materialized(context).await;
        return;
    }
    let _guard = super::thread_settings::acquire_persistence_lock(session).await;
    if !session
        .gateway_settings_checkpointed
        .load(Ordering::Acquire)
    {
        let snapshot = session.thread_settings_snapshot().await;
        if snapshot.gateway_config.is_some() {
            session
                .persist_rollout_items(&[RolloutItem::EventMsg(EventMsg::ThreadSettingsApplied(
                    codex_protocol::protocol::ThreadSettingsAppliedEvent {
                        thread_id: Some(session.thread_id()),
                        thread_settings: snapshot,
                    },
                ))])
                .await;
        }
        session.ensure_rollout_materialized(context).await;
        session
            .gateway_settings_checkpointed
            .store(true, Ordering::Release);
    } else {
        session.ensure_rollout_materialized(context).await;
    }
}
