use super::*;
use codex_app_server_protocol::AccountTokenUsageSummary;
use pretty_assertions::assert_eq;

#[test]
fn loaded_state_freezes_chart_anchor_date_at_completion() {
    let state = Arc::new(RwLock::new(TokenActivityState::Loading));
    let handle = TokenActivityHandle {
        state: Arc::clone(&state),
    };
    let today =
        NaiveDate::from_ymd_opt(/*year*/ 2026, /*month*/ 5, /*day*/ 29).expect("valid date");

    handle.finish_with_today(
        Ok(GetAccountTokenUsageResponse {
            summary: AccountTokenUsageSummary {
                lifetime_tokens: None,
                peak_daily_tokens: None,
                longest_running_turn_sec: None,
                current_streak_days: None,
                longest_streak_days: None,
            },
            daily_usage_buckets: None,
            thread_usage: None,
        }),
        today,
    );

    let state = state.read().expect("token activity state poisoned");
    match &*state {
        TokenActivityState::Loaded {
            today: loaded_today,
            ..
        } => {
            assert_eq!(*loaded_today, today);
        }
        other => panic!("expected loaded state, got {other:?}"),
    }
}

#[tokio::test]
async fn shared_usage_completion_refreshes_overlay_without_revision_change() {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    let (mut widget, _sender, _events, _operations) =
        crate::chatwidget::tests::make_chatwidget_manual_with_sender().await;
    widget.add_token_activity_output(TokenActivityView::Daily);
    let handle = widget
        .refreshing_token_activity_output
        .as_ref()
        .expect("pending usage")
        .handle
        .clone();
    let key = widget.active_cell_transcript_key();
    assert!(!key.expect("live key").cacheable);
    let mut overlay = crate::pager_overlay::TranscriptOverlay::new(
        Vec::new(),
        crate::keymap::RuntimeKeymap::defaults().pager,
    );
    let area = Rect::new(
        /*x*/ 0, /*y*/ 0, /*width*/ 60, /*height*/ 12,
    );
    let mut frames = Vec::new();
    for completed in [false, true] {
        if completed {
            handle.finish(Err("fixture unavailable".to_string()));
        }
        assert_eq!(widget.active_cell_transcript_key(), key);
        overlay.sync_live_tail(area.width, key, |width| {
            widget.active_cell_transcript_hyperlink_lines(width)
        });
        let mut buffer = Buffer::empty(area);
        overlay.render(area, &mut buffer);
        frames.push(
            buffer
                .content
                .iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>(),
        );
    }
    assert!(frames[0].contains("Loading..."));
    assert!(frames[1].contains("Token activity unavailable"));
    assert!(!frames[1].contains("Loading..."));
}
