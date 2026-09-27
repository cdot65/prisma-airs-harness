//! Private AIRS dialogs retain input ownership above the fullscreen transcript.
use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn fullscreen_private_dialogs_do_not_copy_or_submit_credentials() -> Result<()> {
    let (mut app, mut events, mut operations) =
        crate::app::tests::make_test_app_with_channels().await;
    let mut server = Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    tui.set_owned_screen(/*owned*/ true)?;
    app.chat_widget.insert_str("preserve this unsent draft");
    app.transcript_cells = vec![Arc::new(history_cell::PlainHistoryCell::new(vec![
        "public transcript content".into(),
    ]))];
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
        // Selection's fixed shortcut must not steal a private dialog's input.
        app.handle_tui_event(
            &mut tui,
            &mut server,
            TuiEvent::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL)),
        )
        .await?;
        assert!(!app.transcript_view.has_active_interaction());
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
