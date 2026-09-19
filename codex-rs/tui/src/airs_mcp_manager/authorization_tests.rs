use super::*;
use pretty_assertions::assert_eq;

#[test]
fn callback_is_hidden_in_rendering_and_sent_only_to_its_private_receiver() {
    let (sender, mut receiver) = oneshot::channel();
    let mut view = AuthorizationView::new(
        "https://auth.example/authorize?state=fixture".into(),
        sender,
        BrowserMode::OtherDevice,
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
    let mut view = AuthorizationView::new(
        "https://auth.example/authorize".into(),
        sender,
        BrowserMode::OtherDevice,
    );
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
        BrowserMode::OtherDevice,
    );
    let rendered = render(&view, 36, 24);
    insta::assert_snapshot!("airs_mcp_narrow_authorization", rendered);
    assert!(rendered.contains("Esc cancel"));
    assert!(view.desired_height(36) < 25);
}

fn render(view: &AuthorizationView, width: u16, height: u16) -> String {
    let area = Rect::new(0, 0, width, height);
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
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn ssh_detection_takes_precedence_over_a_forwarded_display() {
    for ssh in ["SSH_CONNECTION", "SSH_CLIENT", "SSH_TTY"] {
        assert_eq!(
            BrowserMode::detect(|name| name == ssh || name == "DISPLAY"),
            BrowserMode::OtherDevice
        );
    }
    assert_eq!(
        BrowserMode::detect(|name| name == "DISPLAY"),
        BrowserMode::Desktop
    );
    assert_eq!(
        BrowserMode::detect(|name| name == "WAYLAND_DISPLAY"),
        BrowserMode::Desktop
    );
    #[cfg(target_os = "linux")]
    assert_eq!(BrowserMode::detect(|_| false), BrowserMode::OtherDevice);
}

#[test]
fn desktop_launch_success_and_failure_have_actionable_instructions() {
    let (sender, _receiver) = oneshot::channel();
    let mut view = AuthorizationView::new(
        "https://auth.example/authorize".into(),
        sender,
        BrowserMode::Desktop,
    );
    let mut opened = Vec::new();
    view.open_browser(|url| {
        opened.push(url.to_string());
        Ok(())
    });
    let rendered = render(&view, 80, 20);
    assert_eq!(opened, vec!["https://auth.example/authorize"]);
    insta::assert_snapshot!("airs_mcp_desktop_authorization", rendered);
    view.open_browser(|_| Err(()));
    assert_eq!(view.mode, BrowserMode::OtherDevice);
    insta::assert_snapshot!("airs_mcp_browser_unavailable", render(&view, 80, 20));
}

#[test]
fn long_links_are_scrollable_and_callback_paste_returns_to_hidden_input() {
    let (sender, _receiver) = oneshot::channel();
    let mut view = AuthorizationView::new(
        format!(
            "https://auth.example/authorize?state={}THE-END",
            "x".repeat(400)
        ),
        sender,
        BrowserMode::OtherDevice,
    );
    let before = render(&view, 36, 14);
    for _ in 0..20 {
        view.handle_key_event(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        let _ = render(&view, 36, 14);
    }
    let after = render(&view, 36, 14);
    assert!(after.contains("THE-END"));
    assert!(after.contains("Esc cancel"));
    assert_ne!(before, after);
    view.handle_paste("http://127.0.0.1/callback?code=PRIVATE&state=fixture".into());
    assert_eq!(view.scroll.get(), 0);
    assert!(!render(&view, 36, 24).contains("PRIVATE"));
}
