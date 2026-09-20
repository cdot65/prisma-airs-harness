use super::*;
use pretty_assertions::assert_eq;

fn render(view: &KeyView, width: u16) -> String {
    let area = Rect::new(0, 0, width, view.desired_height(width));
    let mut buffer = Buffer::empty(area);
    view.render(area, &mut buffer);
    buffer
        .content
        .chunks(usize::from(width))
        .map(|row| {
            row.iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>()
                .trim_end()
                .to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn key_is_hidden_and_reaches_only_the_private_receiver() {
    let (sender, mut receiver) = oneshot::channel();
    let mut view = KeyView::new(sender);
    view.handle_paste("PRIVATE-TYPESAFE-KEY\n".into());
    let wide = render(&view, 80);
    let narrow = render(&view, 36);
    assert!(!wide.contains("PRIVATE-TYPESAFE-KEY") && !narrow.contains("PRIVATE-TYPESAFE-KEY"));
    insta::assert_snapshot!("airs_typesafe_hidden_key", wide);
    insta::assert_snapshot!("airs_typesafe_hidden_key_narrow", narrow);
    view.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(receiver.try_recv().unwrap(), "PRIVATE-TYPESAFE-KEY");
    assert_eq!(view.completion(), Some(ViewCompletion::Accepted));
    assert!(view.input.is_empty());
}

#[test]
fn invalid_pastes_and_cancel_never_submit_a_key() {
    let (sender, mut receiver) = oneshot::channel();
    let mut view = KeyView::new(sender);
    for value in [
        "two\nlines".to_owned(),
        "x".repeat(16385),
        "non-ascii-λ".to_owned(),
    ] {
        view.handle_paste(value);
        assert!(view.input.is_empty());
    }
    view.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(!view.is_complete());
    view.handle_paste("PRIVATE".into());
    view.on_ctrl_c();
    assert_eq!(
        receiver.try_recv(),
        Err(oneshot::error::TryRecvError::Closed)
    );
    assert_eq!(view.completion(), Some(ViewCompletion::Cancelled));
    assert!(view.input.is_empty());
}

#[test]
fn clear_and_backspace_edit_without_exposing_input() {
    let (sender, mut receiver) = oneshot::channel();
    let mut view = KeyView::new(sender);
    view.handle_paste("old-key".into());
    view.handle_key_event(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
    view.handle_paste("new-keyX".into());
    view.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
    view.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(receiver.try_recv().unwrap(), "new-key");
}
