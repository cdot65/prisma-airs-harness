//! Older pages preserve live events, repeated prompts and invisible activity boundaries.
use super::*;
use crate::app::test_support::make_test_app;
use crate::exec_cell::ExecCell;
use codex_app_server_protocol::TurnItemsView;
use codex_app_server_protocol::UserInput;
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
        items_view: TurnItemsView::Full,
        status,
        error: None,
        started_at: None,
        completed_at: None,
        duration_ms: None,
    }
}

#[test]
fn overlapping_history_keeps_live_turn_state_and_newer_items() {
    let mut current = vec![turn("shared", TurnStatus::InProgress, &["overlap", "live"])];
    merge_older_turns(
        &mut current,
        vec![
            turn("old", TurnStatus::Completed, &["first"]),
            turn("shared", TurnStatus::Completed, &["before", "overlap"]),
        ],
    );
    assert_eq!(
        current,
        vec![
            turn("old", TurnStatus::Completed, &["first"]),
            turn(
                "shared",
                TurnStatus::InProgress,
                &["before", "overlap", "live"]
            ),
        ]
    );
}

fn user_cell(message: &str) -> Arc<dyn HistoryCell> {
    Arc::new(UserHistoryCell {
        message: message.to_string(),
        text_elements: Vec::new(),
        local_image_paths: Vec::new(),
        remote_image_urls: Vec::new(),
    })
}

#[test]
fn repeated_prompt_matching_removes_only_the_hidden_occurrence() {
    let cells = vec![user_cell("repeat"), user_cell("other"), user_cell("repeat")];
    let persisted = vec![
        ("unloaded".to_string(), user_cell("repeat")),
        ("hidden".to_string(), user_cell("repeat")),
        ("middle".to_string(), user_cell("other")),
        ("visible".to_string(), user_cell("repeat")),
    ];
    assert_eq!(
        hidden_transcript_indices(&cells, persisted, &HashSet::from(["hidden", "unloaded"])),
        vec![0]
    );
}

#[tokio::test]
async fn hidden_prompts_and_turns_separate_compatible_exploration_groups() {
    let app = make_test_app().await;
    let cwd = app.config.cwd.clone();
    let command = |id: &str| {
        serde_json::from_value(serde_json::json!({
        "type": "commandExecution", "id": id,
        "command": "cat file.rs", "cwd": cwd, "processId": null,
        "source": "agent", "status": "completed",
        "commandActions": [{"type": "read", "command": "cat file.rs", "name": "file.rs", "path": cwd.join("file.rs")}],
        "aggregatedOutput": "fn main() {}", "exitCode": 0, "durationMs": 5
    })).expect("valid exploration command")
    };
    for status in [
        TurnStatus::Completed,
        TurnStatus::Failed,
        TurnStatus::Interrupted,
        TurnStatus::InProgress,
    ] {
        let mut first = turn("first", status, &[]);
        first.items = vec![
            ThreadItem::EnteredReviewMode {
                id: "enter-review".to_string(),
                review: "changes".to_string(),
            },
            command("before-hidden"),
            ThreadItem::UserMessage {
                id: "hidden".to_string(),
                client_id: None,
                content: vec![UserInput::Text {
                    text: "internal prompt".to_string(),
                    text_elements: Vec::new(),
                }],
            },
            command("after-hidden"),
        ];
        let mut second = turn("second", TurnStatus::InProgress, &[]);
        second.items = vec![command("next-turn")];
        let turns = vec![first, second];
        let hidden = hidden_review_item_ids(&turns);
        assert_eq!(hidden, HashSet::from(["hidden"]));
        let cells = app.project_older_history_cells(
            turns.iter().flat_map(|turn| turn.items.clone()).collect(),
            &turns,
            &hidden,
            ThreadId::new(),
            &cwd,
            RawReasoningVisibility::Hidden,
        );
        let groups = cells
            .iter()
            .filter_map(|cell| cell.as_any().downcast_ref::<ExecCell>())
            .map(|group| {
                group
                    .iter_calls()
                    .map(|call| call.call_id.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            groups,
            vec![
                vec!["before-hidden"],
                vec!["after-hidden"],
                vec!["next-turn"]
            ]
        );
        assert!(
            !cells
                .iter()
                .any(|cell| cell.as_any().is::<UserHistoryCell>())
        );
        let visible = cells
            .iter()
            .flat_map(|cell| cell.compact_hyperlink_lines(80))
            .map(|line| line.line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!("older_page_groups", visible);
    }
}

#[tokio::test]
async fn prepended_page_stays_after_initial_header_in_both_views() {
    for overlay_open in [false, true] {
        let mut app = make_test_app().await;
        let header: Arc<dyn HistoryCell> = Arc::new(SessionHeaderHistoryCell::new(
            "AI Gateway — default".to_string(),
            /*reasoning_effort*/ None,
            /*show_fast_status*/ false,
            app.config.cwd.to_path_buf(),
            "test",
        ));
        let recent = user_cell("recent prompt");
        let older = user_cell("older prompt");
        app.transcript_cells = vec![Arc::clone(&header), Arc::clone(&recent)];
        if overlay_open {
            let mut tui = crate::tui::test_support::make_test_tui().expect("test terminal");
            app.open_transcript_overlay(&mut tui);
        }
        app.prepend_older_transcript_cells(vec![Arc::clone(&older)], /*width*/ 80);
        assert_eq!(app.transcript_cells.len(), 3);
        for (actual, expected) in app.transcript_cells.iter().zip([&header, &older, &recent]) {
            assert!(Arc::ptr_eq(actual, expected));
        }
    }
}
