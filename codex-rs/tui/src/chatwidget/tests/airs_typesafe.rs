use super::*;
use crate::airs_typesafe::Event;
use crate::airs_typesafe::VIEW_ID;
use crate::airs_typesafe::views;
use crate::render::renderable::Renderable;
use ratatui::backend::TestBackend;

#[tokio::test]
async fn typesafe_views_preserve_draft_and_keep_key_out_of_model_events() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(None).await;
    chat.insert_str("Keep this unsent request");
    for (name, view) in [
        (
            "airs_typesafe_settings",
            views::overview("work", "No key configured. No requests sent."),
        ),
        ("airs_typesafe_remove", views::confirm_clear()),
    ] {
        chat.show_airs_typesafe(view);
        insta::assert_snapshot!(name, render(&chat, 60));
        chat.bottom_pane.dismiss_view_by_id(VIEW_ID);
        assert_eq!(chat.bottom_pane.composer_text(), "Keep this unsent request");
    }
    let (sender, mut receiver) = tokio::sync::oneshot::channel();
    chat.enter_airs_typesafe_key(sender);
    chat.bottom_pane.handle_paste("PRIVATE-KEY".into());
    chat.bottom_pane
        .handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(receiver.try_recv().unwrap(), "PRIVATE-KEY");
    assert_eq!(chat.bottom_pane.composer_text(), "Keep this unsent request");
    while let Ok(event) = rx.try_recv() {
        assert!(!matches!(
            event,
            AppEvent::CodexOp(_) | AppEvent::AirsTypeSafe(_)
        ));
        assert!(!format!("{event:?}").contains("PRIVATE-KEY"));
    }
    chat.input_queue.user_turn_pending_start = true;
    assert!(!chat.airs_typesafe_ready());
}

#[test]
fn remove_requires_confirmation_and_settings_events_are_private() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let sender = AppEventSender::new(tx);
    let view = views::overview("work", "Configured");
    view.items[2].actions[0](&sender);
    assert!(matches!(
        rx.try_recv().unwrap(),
        AppEvent::AirsTypeSafe(Event::ConfirmClear)
    ));
    let view = views::confirm_clear();
    view.items[1].actions[0](&sender);
    assert!(matches!(
        rx.try_recv().unwrap(),
        AppEvent::AirsTypeSafe(Event::Clear)
    ));
    let event = Event::Done {
        attempt: 1,
        thread: None,
        home: std::path::PathBuf::from("private-home"),
        result: Ok("PRIVATE-STATUS".into()),
    };
    assert_eq!(format!("{event:?}"), "AirsTypeSafeEvent [private settings]");
}

fn render(chat: &ChatWidget, width: u16) -> String {
    let mut terminal =
        ratatui::Terminal::new(TestBackend::new(width, chat.desired_height(width).min(40)))
            .unwrap();
    terminal
        .draw(|frame| chat.render(frame.area(), frame.buffer_mut()))
        .unwrap();
    terminal
        .backend()
        .buffer()
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
