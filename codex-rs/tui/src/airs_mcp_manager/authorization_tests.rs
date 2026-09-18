use super::*;
use pretty_assertions::assert_eq;

#[test]
fn callback_is_hidden_in_rendering_and_sent_only_to_its_private_receiver() {
    let (sender, mut receiver) = oneshot::channel();
    let mut view = AuthorizationView::new(
        "https://auth.example/authorize?state=fixture".into(),
        sender,
    );
    view.handle_paste("http://127.0.0.1/callback?code=PRIVATE-CODE&state=fixture\n".into());
    let area = Rect::new(0, 0, 80, view.desired_height(80));
    let mut buffer = Buffer::empty(area);
    view.render(area, &mut buffer);
    let rendered = buffer
        .content
        .chunks(80)
        .map(|row| {
            row.iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!rendered.contains("PRIVATE-CODE"));
    insta::assert_snapshot!("airs_mcp_private_callback", rendered);
    view.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(
        receiver.try_recv().unwrap(),
        "http://127.0.0.1/callback?code=PRIVATE-CODE&state=fixture"
    );
    assert_eq!(view.completion(), Some(ViewCompletion::Accepted));
}

#[test]
fn cancel_closes_callback_channel_without_submitting_and_oversize_paste_is_rejected() {
    let (sender, mut receiver) = oneshot::channel();
    let mut view = AuthorizationView::new("https://auth.example/authorize".into(), sender);
    view.handle_paste("secret\nsecond-line".into());
    assert!(view.input.is_empty());
    view.handle_paste("x".repeat(65537));
    assert!(view.input.is_empty());
    view.handle_paste("sensitive-callback".into());
    view.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(view.completion(), Some(ViewCompletion::Cancelled));
    assert_eq!(
        receiver.try_recv(),
        Err(oneshot::error::TryRecvError::Closed)
    );
    assert!(view.input.is_empty());
}

#[test]
fn narrow_terminal_keeps_callback_instructions_and_full_link_action_visible() {
    let (sender, _receiver) = oneshot::channel();
    let view = AuthorizationView::new(
        format!(
            "https://auth.example/authorize?state={}",
            "long-query".repeat(50)
        ),
        sender,
    );
    let lines = view.lines(36);
    let rendered = lines
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("airs_mcp_narrow_authorization", rendered);
    assert!(rendered.contains("Esc cancel"));
    assert!(rendered.contains("full link"));
    assert!(view.desired_height(36) < 25);
}
