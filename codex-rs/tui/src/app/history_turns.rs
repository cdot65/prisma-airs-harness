//! Separate older history at persisted turn boundaries without inventing completion metadata.

use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::Turn;
use std::collections::HashSet;

pub(super) fn group_turn_items(items: Vec<ThreadItem>, turns: &[Turn]) -> Vec<Vec<ThreadItem>> {
    let boundaries: HashSet<_> = turns
        .iter()
        .filter_map(|turn| turn.items.last().map(ThreadItem::id))
        .collect();
    let mut groups = Vec::new();
    let mut pending = Vec::new();
    for item in items {
        let boundary = boundaries.contains(item.id());
        pending.push(item);
        if boundary {
            groups.push(std::mem::take(&mut pending));
        }
    }
    if !pending.is_empty() {
        groups.push(pending);
    }
    groups
}

#[cfg(test)]
#[path = "history_turns_tests.rs"]
mod tests;
