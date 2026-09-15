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
                .map(ratatui::buffer::Cell::symbol)
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

#[test]
fn mcp_recovery_recognizes_the_core_error_wrapper() {
    let message = "MCP sign-in required: prisma-airs. Sign in at the bound gateway.";
    let wire = codex_protocol::error::CodexErr::Fatal(message.into()).to_string();
    assert_eq!(
        super::super::airs_recovery::mcp_sign_in_message(&wire),
        Some(message)
    );
    assert_eq!(
        super::super::airs_recovery::mcp_sign_in_message(message),
        Some(message)
    );
    assert_eq!(
        super::super::airs_recovery::mcp_sign_in_message("Fatal error: backend unavailable"),
        None
    );
}

#[tokio::test]
async fn mcp_recovery_preserves_draft_and_does_not_unblock_on_company_login() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(None).await;
    chat.thread_id = Some(ThreadId::new());
    chat.config
        .mcp_servers
        .set(std::collections::HashMap::from([(
            "mcp-server-1".into(),
            serde_json::from_value(serde_json::json!({"url": "https://gateway.example/mcp"}))
                .unwrap(),
        )]))
        .unwrap();
    chat.insert_str("Keep this unsent draft");
    chat.input_queue.authentication_pending = true;
    chat.open_airs_mcp_recovery("MCP sign-in required: mcp-server-1. Recovery required.");
    assert_eq!(
        chat.input_queue.mcp_authentication_pending.as_deref(),
        Some("mcp-server-1")
    );
    insta::assert_snapshot!("airs_mcp_sign_in", render_recovery(&chat));
    chat.airs_sign_in_completed(Ok(()));
    assert!(chat.input_queue.authentication_pending);
    chat.airs_mcp_sign_in_completed("mcp-server-1".into(), 8, chat.thread_id.unwrap());
    insta::assert_snapshot!("airs_mcp_continue", render_recovery(&chat));
    assert!(chat.input_queue.authentication_pending);
    assert_eq!(chat.bottom_pane.composer_text(), "Keep this unsent draft");
    while let Ok(event) = rx.try_recv() {
        assert!(!matches!(
            event,
            AppEvent::CodexOp(_) | AppEvent::AirsMcpSignIn { .. } | AppEvent::NewSession { .. }
        ));
    }
}

#[tokio::test]
async fn mcp_recovery_rejects_unconfigured_names_and_other_credential_paths() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(None).await;
    chat.config.mcp_servers.set(std::collections::HashMap::from([(
        "configured".into(),
        serde_json::from_value(serde_json::json!({"url": "https://gateway.example/mcp", "bearer_token_env_var": "OTHER_CREDENTIAL"})).unwrap(),
    )])).unwrap();
    for message in [
        "MCP sign-in required: attacker. https://attacker.example/login",
        "MCP sign-in required: configured. Sign in.",
    ] {
        chat.open_airs_mcp_recovery(message);
        assert_eq!(chat.input_queue.mcp_authentication_pending, None);
    }
    while let Ok(event) = rx.try_recv() {
        assert!(!matches!(event, AppEvent::AirsMcpSignIn { .. }));
    }
}

fn render_recovery(chat: &ChatWidget) -> String {
    let width = 100;
    let mut terminal =
        ratatui::Terminal::new(TestBackend::new(width, chat.desired_height(width))).unwrap();
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

#[test]
fn mcp_recovery_actions_keep_the_origin_thread_and_require_an_explicit_choice() {
    let thread_id = ThreadId::new();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let tx = crate::app_event_sender::AppEventSender::new(tx);
    let sign_in =
        super::super::airs_mcp_recovery::mcp_sign_in_view("mcp-server-1".into(), thread_id);
    assert!(rx.try_recv().is_err());
    (sign_in.items[0].actions[0])(&tx);
    assert!(
        matches!(rx.try_recv().unwrap(), AppEvent::AirsMcpSignIn { server, thread_id: actual }
        if server == "mcp-server-1" && actual == thread_id)
    );
    (sign_in.items[1].actions[0])(&tx);
    assert!(matches!(rx.try_recv().unwrap(), AppEvent::AirsSignInCancel));
    let continuation =
        super::super::airs_mcp_recovery::mcp_continue_view("mcp-server-1".into(), thread_id);
    assert!(rx.try_recv().is_err());
    assert!(continuation.items[1].actions.is_empty());
    (continuation.items[0].actions[0])(&tx);
    assert!(
        matches!(rx.try_recv().unwrap(), AppEvent::AirsMcpNewConversation { thread_id: actual }
        if actual == thread_id)
    );
    assert!(rx.try_recv().is_err());
}
