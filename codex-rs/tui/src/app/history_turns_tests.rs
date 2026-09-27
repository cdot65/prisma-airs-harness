//! Turn boundaries must survive partial pages and unsuccessful turns.

use super::*;
use codex_app_server_protocol::TurnItemsView;
use codex_app_server_protocol::TurnStatus;
use pretty_assertions::assert_eq;

fn turn(id: &str, status: TurnStatus, item_ids: &[&str]) -> Turn {
    Turn {
        id: id.to_string(),
        items: item_ids
            .iter()
            .map(|id| ThreadItem::UserMessage {
                id: id.to_string(),
                client_id: None,
                content: Vec::new(),
            })
            .collect(),
        items_view: TurnItemsView::Summary,
        status,
        error: None,
        started_at: None,
        completed_at: Some(1_700_000_000),
        duration_ms: Some(125_000),
    }
}

#[test]
fn split_turn_retains_only_the_items_on_each_page() {
    let turns = vec![turn(
        "turn",
        TurnStatus::Completed,
        &["first", "second", "last"],
    )];
    assert_eq!(
        group_turn_items(turns[0].items[1..].to_vec(), &turns),
        vec![turns[0].items[1..].to_vec()],
    );
    assert_eq!(
        group_turn_items(turns[0].items[..1].to_vec(), &turns),
        vec![turns[0].items[..1].to_vec()],
    );
}

#[test]
fn multiple_turns_keep_item_groups_in_order() {
    let turns = vec![
        turn("first", TurnStatus::Completed, &["a", "b"]),
        turn("second", TurnStatus::Completed, &["c", "d"]),
    ];
    let items = turns.iter().flat_map(|turn| turn.items.clone()).collect();
    assert_eq!(
        group_turn_items(items, &turns),
        vec![turns[0].items.clone(), turns[1].items.clone(),],
    );
}

#[test]
fn unsuccessful_and_running_turns_separate_groups() {
    let turns = vec![
        turn("failed", TurnStatus::Failed, &["a"]),
        turn("interrupted", TurnStatus::Interrupted, &["b"]),
        turn("running", TurnStatus::InProgress, &["c"]),
    ];
    let items = turns
        .iter()
        .flat_map(|turn| turn.items.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        group_turn_items(items, &turns),
        turns
            .iter()
            .map(|turn| turn.items.clone())
            .collect::<Vec<_>>(),
    );
}
