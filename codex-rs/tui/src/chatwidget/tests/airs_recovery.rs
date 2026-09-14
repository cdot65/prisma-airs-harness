use super::*;
use crate::chatwidget::airs_recovery::sign_in_view;
use crate::render::renderable::Renderable;
use pretty_assertions::assert_eq;
use ratatui::backend::TestBackend;

#[tokio::test]
async fn recovery_view_preserves_a_draft_and_exposes_explicit_actions() {
    let (mut chat, _rx, _ops) = make_chatwidget_manual(None).await;
    chat.insert_str("Keep this unfinished request");
    chat.bottom_pane.show_selection_view(sign_in_view());
    let width = 88;
    let mut terminal =
        ratatui::Terminal::new(TestBackend::new(width, chat.desired_height(width))).unwrap();
    terminal
        .draw(|frame| chat.render(frame.area(), frame.buffer_mut()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let lines: Vec<String> = buffer
        .content
        .chunks(usize::from(width))
        .map(|row| {
            row.iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect();
    insta::assert_snapshot!("airs_sign_in_recovery", lines.join("\n"));
    chat.input_queue.authentication_pending = true;
    assert!(!chat.maybe_send_next_queued_input());
    chat.airs_sign_in_completed(Err("Sign-in cancelled".into()));
    assert!(chat.input_queue.authentication_pending);
    chat.airs_sign_in_completed(Ok(()));
    assert_eq!(chat.input_queue.authentication_pending, false);
    assert_eq!(
        chat.bottom_pane.composer_text(),
        "Keep this unfinished request"
    );
}
