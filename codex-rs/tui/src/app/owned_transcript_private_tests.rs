//! Private AIRS dialogs retain input ownership above the fullscreen transcript.
use super::*;
use pretty_assertions::assert_eq;

#[derive(Clone, Copy)]
enum SearchState {
    Inactive,
    Active,
}

#[tokio::test]
async fn fullscreen_private_dialogs_do_not_copy_or_submit_credentials() -> Result<()> {
    check_private_dialogs(SearchState::Inactive).await
}

#[tokio::test]
async fn fullscreen_private_dialogs_keep_keys_and_callbacks_out_of_active_find() -> Result<()> {
    check_private_dialogs(SearchState::Active).await
}

async fn check_private_dialogs(search: SearchState) -> Result<()> {
    let (mut app, mut events, mut operations) =
        crate::app::tests::make_test_app_with_channels().await;
    let mut server = Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    tui.set_owned_screen(/*owned*/ true)?;
    app.chat_widget.insert_str("preserve this unsent draft");
    app.transcript_cells = vec![Arc::new(history_cell::PlainHistoryCell::new(vec![
        "public transcript content".into(),
    ]))];
    let size = tui.terminal.size()?;
    app.render_owned_transcript(&mut tui, size)?;
    let end = tui.terminal.last_known_cursor_pos;
    let start = end.x - "preserve this unsent draft".len() as u16;
    for (kind, column) in [
        (
            crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
            start,
        ),
        (
            crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Left),
            start + 8,
        ),
        (
            crossterm::event::MouseEventKind::Up(crossterm::event::MouseButton::Left),
            start + 8,
        ),
    ] {
        app.handle_owned_transcript_event(
            &mut tui,
            &mut server,
            &TuiEvent::Mouse(crossterm::event::MouseEvent {
                kind,
                column,
                row: end.y,
                modifiers: KeyModifiers::NONE,
            }),
        )?;
    }
    let copy_event = TuiEvent::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::SUPER));
    assert!(
        app.handle_composer_copy_event(&mut tui, &copy_event, |_, text| {
            assert_eq!(text, "preserve");
            Ok(crate::clipboard_copy::CopyStatus::Confirmed)
        })
    );
    if matches!(search, SearchState::Active) {
        app.transcript_view.begin_search();
        app.transcript_view.paste_search("public transcript");
    }
    for mcp in [false, true] {
        let (sender, mut receiver) = tokio::sync::oneshot::channel();
        let private = if mcp {
            "http://127.0.0.1:54321/callback?code=fixture-private-code&state=fixture-state"
        } else {
            "fixture-private-typesafe-key"
        };
        let view: Box<dyn crate::bottom_pane::BottomPaneView> = if mcp {
            Box::new(crate::airs_mcp_manager::AuthorizationView::new(
                "https://gateway.example.invalid/authorize".into(),
                sender,
                crate::airs_mcp_manager::BrowserMode::OtherDevice,
            ))
        } else {
            Box::new(crate::airs_typesafe::input::KeyView::new(sender))
        };
        app.chat_widget.show_bottom_pane_view(view);
        app.handle_tui_event(&mut tui, &mut server, TuiEvent::Paste(private.into()))
            .await?;
        assert!(
            !app.handle_composer_copy_event(&mut tui, &copy_event, |_, _| {
                panic!("private dialog must block the underlying draft clipboard")
            })
        );
        // Selection's fixed shortcut must not steal a private dialog's input.
        app.handle_tui_event(
            &mut tui,
            &mut server,
            TuiEvent::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL)),
        )
        .await?;
        assert_eq!(
            app.transcript_view.has_active_interaction(),
            matches!(search, SearchState::Active)
        );
        assert!(
            app.transcript_view
                .selected_text(&app.transcript_cells)
                .is_none()
        );
        assert!(matches!(
            receiver.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
        let bottom = app.chat_widget.bottom_pane_renderable(/*footer*/ None);
        let area = Rect::new(
            /*x*/ 0, /*y*/ 0, /*width*/ 80, /*height*/ 24,
        );
        let mut buffer = ratatui::buffer::Buffer::empty(area);
        bottom.render(area, &mut buffer);
        let rendered = buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(!rendered.contains(private));
        drop(bottom);
        app.handle_tui_event(
            &mut tui,
            &mut server,
            TuiEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        )
        .await?;
        assert_eq!(receiver.await?, private);
        if matches!(search, SearchState::Active) {
            assert!(app.transcript_view.is_search_active());
            let footer = app
                .transcript_view
                .footer_with_navigation(
                    /*width*/ 100,
                    crate::motion::MotionMode::Reduced,
                    "esc latest",
                )
                .expect("Find footer");
            let text = footer.text.to_string();
            assert!(text.contains("public transcript"));
            assert!(!text.contains(private));
        }
        assert_eq!(
            app.chat_widget.composer_text_with_pending(),
            "preserve this unsent draft"
        );
        assert!(
            operations.try_recv().is_err(),
            "private input must not submit an agent operation"
        );
        while let Ok(event) = events.try_recv() {
            assert!(!format!("{event:?}").contains(private));
        }
        let transcript = app
            .transcript_cells
            .iter()
            .flat_map(|cell| cell.transcript_lines(/*width*/ 80))
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!transcript.contains(private));
    }
    tui.set_owned_screen(/*owned*/ false)?;
    server.shutdown().await?;
    Ok(())
}
