//! Older pages preserve browsing, selection, cancellation and pending request ownership.
use super::owned_history_fixture::completed_history_app;
use super::session_lifecycle_requests::start_recording_app_server;
use super::*;
use crate::pager_overlay::TranscriptHistoryState;
use pretty_assertions::assert_eq;
#[tokio::test]
async fn beginning_navigation_holds_the_view_until_the_last_page_arrives() -> Result<()> {
    for initial_scroll in [0, -1] {
        let (mut app, _codex_home, thread_id) =
            completed_history_app(&["Oldest", "Middle", "Newest"]).await?;
        let (mut app_server, _requests, proxy) = start_recording_app_server(
            &app.config,
            /*blocked_thread_list*/ None,
            /*failed_thread_name*/ None,
        )
        .await?;
        let started = app_server
            .resume_thread(
                &app.local_settings,
                app.config.clone(),
                thread_id,
                crate::app_server_session::ResumeModelSettings::RestoreFromThread,
            )
            .await?;
        app.transcript_cells = crate::thread_transcript::thread_items_to_transcript_cells(
            Some(thread_id),
            &app.config.cwd,
            started.turns.iter().flat_map(|turn| turn.items.clone()),
            crate::thread_transcript::RawReasoningVisibility::Hidden,
            Some(&app.config),
        );
        app.enqueue_primary_thread_session(started.session, started.turns)
            .await?;
        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
        app.app_event_tx = crate::app_event_sender::AppEventSender::new(event_tx);
        app.scrollback_has_older_history = app_server.has_older_history(thread_id);
        app.transcript_view.history = TranscriptHistoryState::Partial;
        let mut tui = crate::tui::test_support::make_test_tui()?;
        tui.set_owned_screen(/*owned*/ true)?;
        let size = ratatui::layout::Size::new(/*width*/ 80, /*height*/ 12);
        let render = |app: &mut App, tui: &mut tui::Tui| -> Result<Buffer> {
            tui.screen_size_for_event(&TuiEvent::Resize(size))?;
            let bottom = app.render_owned_transcript(tui, size)?;
            let area = Rect::new(
                /*x*/ 0,
                /*y*/ 0,
                size.width,
                bottom.y.saturating_sub(/*rhs*/ 1),
            );
            let mut buffer = Buffer::empty(area);
            app.transcript_view
                .render(area, &mut buffer, &app.transcript_cells);
            Ok(buffer)
        };
        render(&mut app, &mut tui)?;
        if initial_scroll != 0 {
            app.transcript_view
                .scroll(&app.transcript_cells, initial_scroll);
        }
        let before = render(&mut app, &mut tui)?;

        // Hold a small first page so the test can inspect each request/completion boundary.
        let cursor = app_server
            .begin_older_history_page(thread_id)
            .expect("older page");
        let first_page = app_server
            .thread_items_page(
                thread_id,
                /*turn_id*/ None,
                Some(cursor.clone()),
                /*limit*/ 1,
            )
            .await?;
        let final_cursor = first_page.next_cursor.clone().expect("another older page");
        tui.screen_size_for_event(&TuiEvent::Resize(size))?;
        assert!(app.handle_owned_transcript_event(
            &mut tui,
            &mut app_server,
            &TuiEvent::Key(KeyEvent::new(KeyCode::Char('<'), KeyModifiers::ALT)),
        )?);
        assert_eq!(
            (render(&mut app, &mut tui)?, app.transcript_view.history),
            (before.clone(), TranscriptHistoryState::LoadingBeginning),
        );
        app.handle_older_history_page(
            &mut tui,
            &mut app_server,
            thread_id,
            &cursor,
            Ok(first_page),
        )
        .await?;
        assert!(app_server.is_older_history_page_pending(thread_id, &final_cursor));
        let final_event = tokio::time::timeout(Duration::from_secs(/*secs*/ 5), event_rx.recv())
            .await?
            .expect("final page completion");
        assert!(matches!(
            &final_event,
            AppEvent::OlderThreadHistoryLoaded { result: Ok(page), .. }
                if page.next_cursor.is_none()
        ));
        // A draw while the completion is waiting must leave the viewport and loading intent intact.
        app.handle_owned_transcript_event(&mut tui, &mut app_server, &TuiEvent::Draw)?;
        assert_eq!(
            (render(&mut app, &mut tui)?, app.transcript_view.history),
            (before, TranscriptHistoryState::LoadingBeginning),
        );
        Box::pin(app.handle_event(&mut tui, &mut app_server, final_event)).await?;
        let final_buffer = render(&mut app, &mut tui)?;
        let visible = final_buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(visible.contains("Oldest prompt"), "visible: {visible}");
        assert_eq!(
            (
                app.transcript_view.history,
                app.transcript_view.is_following(),
                app_server.has_older_history(thread_id),
            ),
            (TranscriptHistoryState::Complete, false, false),
        );
        tui.set_owned_screen(/*owned*/ false)?;
        app_server.shutdown().await?;
        proxy.await??;
    }
    Ok(())
}

#[tokio::test]
async fn returning_to_latest_retains_pending_pages_without_continuing_to_the_beginning()
-> Result<()> {
    let (mut app, _codex_home, thread_id) =
        completed_history_app(&["Oldest", "Middle", "Newest"]).await?;
    let (mut app_server, _requests, proxy) = start_recording_app_server(
        &app.config,
        /*blocked_thread_list*/ None,
        /*failed_thread_name*/ None,
    )
    .await?;
    let started = app_server
        .resume_thread(
            &app.local_settings,
            app.config.clone(),
            thread_id,
            crate::app_server_session::ResumeModelSettings::RestoreFromThread,
        )
        .await?;
    app.transcript_cells = crate::thread_transcript::thread_items_to_transcript_cells(
        Some(thread_id),
        &app.config.cwd,
        started.turns.iter().flat_map(|turn| turn.items.clone()),
        crate::thread_transcript::RawReasoningVisibility::Hidden,
        Some(&app.config),
    );
    app.enqueue_primary_thread_session(started.session, started.turns)
        .await?;
    app.scrollback_has_older_history = app_server.has_older_history(thread_id);
    app.transcript_view.history = TranscriptHistoryState::Partial;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    tui.set_owned_screen(/*owned*/ true)?;

    for (limit, expected_history) in [
        (1, TranscriptHistoryState::Partial),
        (100, TranscriptHistoryState::Complete),
    ] {
        app.transcript_view.handle_key(
            KeyEvent::new(KeyCode::Home, KeyModifiers::CONTROL),
            &app.transcript_cells,
        );
        let cursor = app_server
            .begin_older_history_page(thread_id)
            .expect("older page");
        let page = app_server
            .thread_items_page(
                thread_id,
                /*turn_id*/ None,
                Some(cursor.clone()),
                limit,
            )
            .await?;
        let next_cursor = page.next_cursor.clone();
        let before = app.transcript_cells.len();
        app.transcript_view.handle_key(
            KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL),
            &app.transcript_cells,
        );
        app.handle_older_history_page(&mut tui, &mut app_server, thread_id, &cursor, Ok(page))
            .await?;
        assert!(
            app.transcript_cells.len() > before,
            "received history is retained"
        );
        assert_eq!(
            (
                app.transcript_view.history,
                app.transcript_view.is_following()
            ),
            (expected_history, true),
        );
        if let Some(next_cursor) = next_cursor {
            assert!(!app_server.is_older_history_page_pending(thread_id, &next_cursor));
        }
    }
    tui.set_owned_screen(/*owned*/ false)?;
    app_server.shutdown().await?;
    proxy.await??;
    Ok(())
}

#[tokio::test]
async fn stale_history_completions_preserve_the_current_request_and_failure_can_retry() -> Result<()>
{
    let (mut app, _codex_home, thread_id) =
        completed_history_app(&["Oldest", "Middle", "Newest"]).await?;
    let (mut app_server, _requests, proxy) = start_recording_app_server(
        &app.config,
        /*blocked_thread_list*/ None,
        /*failed_thread_name*/ None,
    )
    .await?;
    let started = app_server
        .resume_thread(
            &app.local_settings,
            app.config.clone(),
            thread_id,
            crate::app_server_session::ResumeModelSettings::RestoreFromThread,
        )
        .await?;
    app.enqueue_primary_thread_session(started.session, started.turns)
        .await?;
    app.scrollback_has_older_history = app_server.has_older_history(thread_id);
    app.transcript_view.history = TranscriptHistoryState::LoadingBeginning;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    tui.set_owned_screen(/*owned*/ true)?;
    let cursor = app_server
        .begin_older_history_page(thread_id)
        .expect("older page");
    let page = app_server
        .thread_items_page(
            thread_id,
            /*turn_id*/ None,
            Some(cursor.clone()),
            /*limit*/ 1,
        )
        .await?;
    for result in [Ok(page.clone()), Err("obsolete request failed".to_string())] {
        app.handle_older_history_page(
            &mut tui,
            &mut app_server,
            thread_id,
            "obsolete-cursor",
            result,
        )
        .await?;
        assert_eq!(
            (
                app.transcript_cells.len(),
                app.transcript_view.history,
                app_server.is_older_history_page_pending(thread_id, &cursor),
            ),
            (0, TranscriptHistoryState::LoadingBeginning, true),
        );
    }
    let failure = AppEvent::OlderThreadHistoryLoaded {
        thread_id,
        cursor: cursor.clone(),
        result: Err("current request failed".to_string()),
    };
    Box::pin(app.handle_event(&mut tui, &mut app_server, failure)).await?;
    app.handle_owned_transcript_event(&mut tui, &mut app_server, &TuiEvent::Draw)?;
    assert_eq!(
        (
            app.transcript_view.history,
            app_server.is_older_history_page_pending(thread_id, &cursor),
        ),
        (TranscriptHistoryState::Failed, false),
    );
    assert_eq!(
        app_server.begin_older_history_page(thread_id),
        Some(cursor.clone())
    );
    app_server.cancel_older_history_page(thread_id, "obsolete-cursor");
    assert!(app_server.is_older_history_page_pending(thread_id, &cursor));
    app.transcript_view.history = TranscriptHistoryState::LoadingOlder;
    // Initialize the viewport as the app's first draw does before accepting Find input.
    let area = Rect::new(
        /*x*/ 0, /*y*/ 0, /*width*/ 80, /*height*/ 1,
    );
    let mut buffer = Buffer::empty(area);
    app.transcript_view
        .render(area, &mut buffer, &app.transcript_cells);
    app.transcript_view
        .jump_to_entry(&app.transcript_cells, /*index*/ 0);
    app.transcript_view.begin_search();
    app.transcript_view.paste_search("Middle answer");
    app.transcript_view.advance_search(&app.transcript_cells);
    let next_cursor = page.next_cursor.clone().expect("another older page");
    app.handle_older_history_page(&mut tui, &mut app_server, thread_id, &cursor, Ok(page))
        .await?;
    assert_eq!(app.transcript_view.history, TranscriptHistoryState::Partial);
    assert!(!app.transcript_cells.is_empty());
    // The received page must be searched before another page can start, even near the top.
    assert!(!app_server.is_older_history_page_pending(thread_id, &next_cursor));
    // Scanning yields after the completion footer before reaching the answer in the same page.
    for _ in 0..8 {
        if !app.transcript_view.advance_search(&app.transcript_cells) {
            break;
        }
    }
    app.transcript_view
        .render(area, &mut buffer, &app.transcript_cells);
    let rendered = buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect::<String>();
    assert!(rendered.contains("Middle answer"), "rendered: {rendered:?}",);

    assert_eq!(
        app_server.begin_older_history_page(thread_id),
        Some(next_cursor.clone())
    );
    app.handle_older_history_page(
        &mut tui,
        &mut app_server,
        thread_id,
        &cursor,
        Err("previous page failed late".to_string()),
    )
    .await?;
    assert!(app_server.is_older_history_page_pending(thread_id, &next_cursor));
    let history = app.transcript_view.history;
    let cell_count = app.transcript_cells.len();
    app.chat_widget.handle_thread_session(test_thread_session(
        ThreadId::new(),
        app.config.cwd.to_path_buf(),
    ));
    app.handle_older_history_page(
        &mut tui,
        &mut app_server,
        thread_id,
        &next_cursor,
        Err("previous thread failed late".to_string()),
    )
    .await?;
    assert_eq!(
        (
            app.transcript_view.history,
            app.transcript_cells.len(),
            app_server.is_older_history_page_pending(thread_id, &next_cursor),
        ),
        (history, cell_count, false),
    );
    tui.set_owned_screen(/*owned*/ false)?;
    app_server.shutdown().await?;
    proxy.await??;
    Ok(())
}
