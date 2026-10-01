use super::*;
use pretty_assertions::assert_eq;

#[test]
fn environment_and_explicit_settings_respect_terminal_ownership() {
    for (ssh, terminal_owns_paste, wsl_unknown_terminal) in [
        (false, false, false),
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let environment = PasteEnvironment {
            platform_default: true,
            ssh,
            terminal_owns_paste,
            wsl_unknown_terminal,
        };
        assert_eq!(environment.allows(RightClickPaste::Off), false);
        assert_eq!(
            environment.allows(RightClickPaste::Auto),
            !ssh && !terminal_owns_paste && !wsl_unknown_terminal
        );
        assert_eq!(
            environment.allows(RightClickPaste::On),
            !ssh && !terminal_owns_paste && !cfg!(target_os = "android")
        );
    }
}

#[tokio::test]
async fn delayed_reads_preserve_new_input_and_bound_workers() -> Result<()> {
    for invalidate in [false, true] {
        let mut app = crate::app::test_support::make_test_app().await;
        let mut tui = crate::tui::test_support::make_test_tui()?;
        tui.set_owned_screen(/*owned*/ true)?;
        app.local_settings.tui.right_click_paste = RightClickPaste::On;
        app.right_click_paste_environment = PasteEnvironment {
            platform_default: true,
            ssh: false,
            terminal_owns_paste: false,
            wsl_unknown_terminal: false,
        };
        app.chat_widget.insert_str("existing draft");
        let target = app.right_click_paste_target(&tui).expect("editable draft");
        let (sender, result) = mpsc::channel();
        app.pending_right_click_paste = Some(PendingPaste {
            target: Some(target),
            deadline: Instant::now() + Duration::from_secs(2),
            result,
        });
        if invalidate {
            app.invalidate_right_click_paste(&TuiEvent::Paste("new input".into()));
            assert!(app.pending_right_click_paste.is_some());
            app.chat_widget.insert_str(" changed");
        }
        sender.send(Ok("clipboard text".into()))?;
        let event = app.finish_right_click_paste(&mut tui, TuiEvent::Draw);
        assert_eq!(
            matches!(event, TuiEvent::Paste(ref text) if text == "clipboard text"),
            !invalidate
        );
        assert!(app.pending_right_click_paste.is_none());
        assert_eq!(
            app.chat_widget.composer_text_with_pending(),
            if invalidate {
                "existing draft changed"
            } else {
                "existing draft"
            }
        );
        tui.set_owned_screen(/*owned*/ false)?;
    }
    Ok(())
}

#[tokio::test]
async fn timeout_discards_late_text_without_starting_another_reader() -> Result<()> {
    let mut app = crate::app::test_support::make_test_app().await;
    let mut tui = crate::tui::test_support::make_test_tui()?;
    tui.set_owned_screen(/*owned*/ true)?;
    app.local_settings.tui.right_click_paste = RightClickPaste::On;
    app.right_click_paste_environment = PasteEnvironment {
        platform_default: true,
        ssh: false,
        terminal_owns_paste: false,
        wsl_unknown_terminal: false,
    };
    let (sender, result) = mpsc::channel();
    app.pending_right_click_paste = Some(PendingPaste {
        target: app.right_click_paste_target(&tui),
        deadline: Instant::now(),
        result,
    });
    assert!(matches!(
        app.finish_right_click_paste(&mut tui, TuiEvent::Draw),
        TuiEvent::Draw
    ));
    assert!(
        app.pending_right_click_paste
            .as_ref()
            .unwrap()
            .target
            .is_none()
    );
    sender.send(Ok("late text".into()))?;
    assert!(matches!(
        app.finish_right_click_paste(&mut tui, TuiEvent::Draw),
        TuiEvent::Draw
    ));
    assert_eq!(app.chat_widget.composer_text_with_pending(), "");
    assert!(app.pending_right_click_paste.is_none());
    tui.set_owned_screen(/*owned*/ false)?;
    Ok(())
}
