//! Join only persisted activity identities within the same turn, retaining reasoning details.
use crate::exec_cell::ExecCell;
use crate::history_cell::HistoryCell;
use crate::history_cell::ReasoningSummaryCell;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::Turn;
use std::sync::Arc;

pub(crate) fn is_hidden_activity_detail(cell: &Arc<dyn HistoryCell>) -> bool {
    cell.as_any()
        .downcast_ref::<ReasoningSummaryCell>()
        .is_some_and(ReasoningSummaryCell::is_transcript_only)
}

/// Rebuild completed exploration only after every trailing detail proves its source identity.
pub(crate) fn fold_trailing_activity_details(
    older: &Arc<dyn HistoryCell>,
    details: &[Arc<dyn HistoryCell>],
    turns: &[Turn],
) -> Option<Arc<dyn HistoryCell>> {
    let group = older.as_any().downcast_ref::<ExecCell>()?;
    if !group.is_exploring_cell() || group.should_flush() {
        return None;
    }
    let mut ids = group
        .iter_calls()
        .map(|call| call.call_id.as_str())
        .collect::<Vec<_>>();
    let count = ids.len();
    for detail in details {
        let detail = detail.as_any().downcast_ref::<ReasoningSummaryCell>()?;
        if !detail.is_transcript_only() {
            return None;
        }
        ids.push(detail.source_item_id()?);
    }
    let items = adjacent_activity_items(&ids, turns)?;
    let mut retained = group.group.details.clone();
    for detail in details {
        retained.push(count, Arc::clone(detail));
    }
    let mut group = super::exploration_groups::completed_group(items)?;
    group.group.details = retained;
    Some(Arc::new(group))
}

/// Validate source adjacency without mistaking invisible reasoning for an activity boundary.
pub(super) fn adjacent_activity_items<'a>(
    ids: &[&str],
    turns: &'a [Turn],
) -> Option<&'a [ThreadItem]> {
    let first = *ids.first()?;
    let (turn, start) = turns.iter().find_map(|turn| {
        turn.items
            .iter()
            .position(|item| item.id() == first)
            .map(|start| (turn, start))
    })?;
    let mut matched = 0;
    for (offset, item) in turn.items[start..].iter().enumerate() {
        if item.id() != ids[matched] {
            if matches!(item, ThreadItem::Reasoning { .. }) {
                continue;
            }
            return None;
        }
        matched += 1;
        if matched == ids.len() {
            return Some(&turn.items[start..=start + offset]);
        }
    }
    None
}
