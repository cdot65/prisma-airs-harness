//! Clicking the composer returns typing ownership from prompt browsing.
use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn composer_mouse_click_leaves_browsing_before_h_and_l_are_typed() -> Result<()> {
    let mut app = crate::app::test_support::make_test_app().await;
    attach_thread(&mut app, ThreadId::new());
    app.transcript_cells = vec![user_cell("first prompt"), user_cell("second prompt")];
    let mut server = Box::pin(crate::start_embedded_app_server_for_picker(&app.config)).await?;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    tui.set_owned_screen(/*owned*/ true)?;
    app.handle_backtrack_esc_key(&mut tui);
    app.handle_backtrack_esc_key(&mut tui);
    assert!(app.backtrack.overlay_preview_active);
    let size = tui.terminal.size()?;
    app.render_owned_transcript(&mut tui, size)?;
    let row = row_containing(&tui, "Ask Codex to do anything");
    for kind in [Down(Left), Up(Left)] {
        app.handle_tui_event(&mut tui, &mut server, mouse(kind, /*column*/ 2, row))
            .await?;
    }
    assert!(!app.backtrack.overlay_preview_active);
    // These are browsing shortcuts until the composer takes ownership.
    for character in ['h', 'l'] {
        app.handle_tui_event(
            &mut tui,
            &mut server,
            TuiEvent::Key(KeyCode::Char(character).into()),
        )
        .await?;
    }
    // A navigation key commits the buffered typing before the separate paste event.
    app.handle_tui_event(&mut tui, &mut server, TuiEvent::Key(KeyCode::End.into()))
        .await?;
    app.handle_tui_event(&mut tui, &mut server, TuiEvent::Paste("!".into()))
        .await?;
    assert_eq!(app.chat_widget.composer_text_with_pending(), "hl!");
    assert_eq!(crate::app_backtrack::user_count(&app.transcript_cells), 2);
    server.shutdown().await?;
    tui.set_owned_screen(/*owned*/ false)?;
    Ok(())
}
