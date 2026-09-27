//! Copy status stays local to the authorization dialog and never submits a callback.

use super::*;
use crate::clipboard_copy::ClipboardLease;
use crate::clipboard_copy::CopyOutcome;
use pretty_assertions::assert_eq;

#[test]
fn authorization_copy_reports_delivery_certainty_and_retains_private_input() {
    let (sender, mut receiver) = oneshot::channel();
    let mut view = AuthorizationView::new(
        "https://auth.example/authorize".into(),
        sender,
        BrowserMode::OtherDevice,
    );
    view.handle_paste("http://127.0.0.1/callback?code=PRIVATE-CODE".into());
    view.copy_authorization_link_with(|url| {
        assert_eq!(url, "https://auth.example/authorize");
        Ok(CopyOutcome::Copied(Some(ClipboardLease::test())))
    });
    assert_eq!(view.notice, "Authorization link copied.");
    view.copy_authorization_link_with(|_| Ok(CopyOutcome::Requested));
    assert!(view.clipboard.is_some());
    assert!(view.notice.contains("not confirmed"));
    assert_eq!(view.input, "http://127.0.0.1/callback?code=PRIVATE-CODE");
    assert_eq!(
        receiver.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    );
    let area = Rect::new(0, 0, 76, view.desired_height(76));
    let mut buffer = Buffer::empty(area);
    view.render(area, &mut buffer);
    let rendered = buffer
        .content
        .chunks(76)
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
    insta::assert_snapshot!("authorization_copy_unconfirmed", rendered);
    view.copy_authorization_link_with(|_| Err("PRIVATE-BACKEND-ERROR".into()));
    assert!(view.clipboard.is_some());
    assert!(!view.notice.contains("PRIVATE-BACKEND-ERROR"));
    assert_eq!(
        receiver.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    );
}
