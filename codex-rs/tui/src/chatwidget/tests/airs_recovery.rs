use super::*;
use crate::chatwidget::airs_recovery::sign_in_view;
use crate::render::renderable::Renderable;
use pretty_assertions::assert_eq;
use ratatui::backend::TestBackend;

#[test]
fn sign_in_menu_cancel_targets_only_the_attempt_displayed_when_it_opened() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let tx = crate::app_event_sender::AppEventSender::new(tx);
    let idle = sign_in_view(/*attempt*/ None);
    (idle.items[1].actions[0])(&tx);
    assert!(rx.try_recv().is_err());
    let active = sign_in_view(Some(41));
    (active.items[1].actions[0])(&tx);
    assert!(matches!(
        rx.try_recv().unwrap(),
        AppEvent::AirsSignInCancel(41)
    ));
    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn recovery_view_preserves_a_draft_and_exposes_explicit_actions() {
    let (mut chat, _rx, _ops) = make_chatwidget_manual(None).await;
    chat.insert_str("Keep this unfinished request");
    chat.bottom_pane
        .show_selection_view(sign_in_view(/*attempt*/ None));
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
    chat.airs_mcp_manager_changed(
        "mcp-server-1: connected · 8 tools.".into(),
        chat.thread_id.unwrap(),
    );
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
    for action in &sign_in.items[1].actions {
        action(&tx);
    }
    assert!(rx.try_recv().is_err());
    let continuation = crate::airs_mcp_manager::views::continue_after_change(thread_id);
    assert!(rx.try_recv().is_err());
    (continuation.items[0].actions[0])(&tx);
    assert!(
        matches!(rx.try_recv().unwrap(), AppEvent::AirsMcpNewConversation { thread_id: actual }
        if actual == thread_id)
    );
    assert!(rx.try_recv().is_err());
}

#[tokio::test]
async fn mcp_manager_menus_preserve_draft_and_cancel_without_model_actions() {
    use crate::airs_mcp_manager::Connection;
    use crate::airs_mcp_manager::Event;
    use crate::airs_mcp_manager::Operation;
    use crate::airs_mcp_manager::views;
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(None).await;
    chat.thread_id = Some(ThreadId::new());
    chat.insert_str("Keep this independent draft");
    let connection = Connection {
        name: "service-now".into(),
        url: "https://gateway.example/service-now/mcp".into(),
        status: "Not connected".into(),
        can_login: true,
    };
    chat.show_airs_mcp_menu(views::overview("work", vec![connection.clone()]));
    insta::assert_snapshot!("airs_mcp_manager_overview", render_recovery(&chat));
    chat.show_airs_mcp_menu(views::connection(connection));
    insta::assert_snapshot!("airs_mcp_manager_connection", render_recovery(&chat));
    chat.show_airs_mcp_menu(views::confirm(Operation::Remove("service-now".into())));
    insta::assert_snapshot!("airs_mcp_manager_remove", render_recovery(&chat));
    for (name, stage) in [
        (
            "airs_mcp_exchange_progress",
            crate::airs_mcp_manager::Progress::ExchangingCode,
        ),
        (
            "airs_mcp_storage_progress",
            crate::airs_mcp_manager::Progress::SavingCredential,
        ),
        (
            "airs_mcp_discovery_progress",
            crate::airs_mcp_manager::Progress::DiscoveringTools,
        ),
    ] {
        chat.show_airs_mcp_menu(views::progress(/*attempt*/ 7, stage));
        insta::assert_snapshot!(name, render_recovery(&chat));
    }

    chat.dismiss_airs_mcp_manager();
    chat.input_queue.user_turn_pending_start = true;
    assert!(!chat.airs_mcp_manager_ready());
    assert!(!chat.prepare_airs_mcp_sign_in("service-now"));
    assert!(!chat.input_queue.authentication_pending);
    chat.input_queue.user_turn_pending_start = false;
    assert!(chat.prepare_airs_mcp_sign_in("service-now"));
    assert!(!chat.maybe_send_next_queued_input());
    assert_eq!(
        chat.bottom_pane.composer_text(),
        "Keep this independent draft"
    );
    while let Ok(event) = rx.try_recv() {
        assert!(!matches!(
            event,
            AppEvent::CodexOp(_) | AppEvent::AirsMcpManager(_)
        ));
    }
    let waiting = views::waiting(/*attempt*/ 7);
    waiting.on_cancel.unwrap()(&chat.app_event_tx);
    assert!(matches!(
        rx.try_recv().unwrap(),
        AppEvent::AirsMcpManager(Event::Cancel(7))
    ));
    let confirmation = views::confirm(Operation::Remove("service-now".into()));
    (confirmation.items[0].actions[0])(&chat.app_event_tx);
    assert!(matches!(
        rx.try_recv().unwrap(),
        AppEvent::AirsMcpManager(Event::Open)
    ));
    (confirmation.items[1].actions[0])(&chat.app_event_tx);
    assert!(
        matches!(rx.try_recv().unwrap(), AppEvent::AirsMcpManager(Event::Run(Operation::Remove(name))) if name == "service-now")
    );
}

#[tokio::test]
async fn mcp_typed_failure_renders_recovery_without_changing_draft() {
    let (mut chat, mut rx, mut ops) = make_chatwidget_manual(None).await;
    chat.insert_str("Keep this unfinished request");
    let failure: codex_protocol::airs_mcp_failure::McpFailure =
        serde_json::from_str(r#"{"code":"permission_denied","connection":"saved"}"#).unwrap();
    chat.add_error_message(failure.to_string());
    let rendered = drain_insert_history(&mut rx)
        .iter()
        .map(|lines| lines_to_single_string(lines))
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("airs_mcp_permission_denied_recovery", rendered);
    assert_eq!(
        chat.bottom_pane.composer_text(),
        "Keep this unfinished request"
    );
    assert!(ops.try_recv().is_err());
}

#[tokio::test]
async fn mcp_onboarding_explains_gateway_endpoint_and_preserves_draft_on_cancel() {
    let (mut chat, mut rx, mut ops) = make_chatwidget_manual(None).await;
    chat.insert_str("Keep my ServiceNow question");
    for (name, snapshot) in [
        (None, "airs_mcp_onboarding_name"),
        (
            Some("service-now".into()),
            "airs_mcp_onboarding_gateway_url",
        ),
    ] {
        chat.prompt_airs_mcp_connection(name);
        insta::assert_snapshot!(snapshot, render_recovery(&chat));
        chat.bottom_pane
            .handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(
            chat.bottom_pane.composer_text(),
            "Keep my ServiceNow question"
        );
    }
    while let Ok(event) = rx.try_recv() {
        assert!(!matches!(
            event,
            AppEvent::AirsMcpManager(_) | AppEvent::CodexOp(_)
        ));
    }
    assert!(ops.try_recv().is_err());
}

#[tokio::test]
async fn auth_recovery_flushes_partial_output_once_without_submitting_queued_input() {
    for plan in [false, true] {
        let (mut chat, mut events, mut ops) = make_chatwidget_manual(None).await;
        if plan {
            chat.set_feature_enabled(Feature::CollaborationModes, /*enabled*/ true);
            chat.set_collaboration_mask(CollaborationModeMask {
                name: "Plan".into(),
                mode: Some(ModeKind::Plan),
                model: None,
                reasoning_effort: None,
                developer_instructions: None,
            });
        }
        chat.on_task_started();
        chat.insert_str("Preserve this unsent draft");
        chat.input_queue
            .queued_user_messages
            .push_back(UserMessage::from("Do not replay this request").into());
        let source = "Received partial output: λ and 日本語";
        if plan {
            chat.on_plan_delta(source.into());
        } else {
            chat.on_agent_message_delta(source.into());
        }
        // Both AIRS recovery branches set this gate before finalizing the turn.
        chat.input_queue.authentication_pending = true;
        chat.finalize_turn();
        chat.finalize_turn();
        assert!(!chat.maybe_send_next_queued_input());
        assert!(ops.try_recv().is_err());
        assert_eq!(chat.input_queue.queued_user_messages.len(), 1);
        assert_eq!(
            chat.bottom_pane.composer_text(),
            "Preserve this unsent draft"
        );
        let saved = std::iter::from_fn(|| events.try_recv().ok())
            .filter_map(|event| match event {
                AppEvent::ConsolidateAgentMessage { source, .. }
                | AppEvent::ConsolidateProposedPlan(source) => {
                    Some(source.trim_end_matches('\n').to_owned())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(saved, vec![source.to_owned()]);
    }
}

#[tokio::test]
async fn application_policy_failure_preserves_draft_without_offering_sign_in() {
    let (mut chat, mut events, mut ops) = make_chatwidget_manual(None).await;
    chat.insert_str("Keep my unsent question");
    let error = codex_protocol::error::CodexErr::Fatal(
        codex_login::auth::CredentialRecovery::PolicyDenied.to_string(),
    );
    chat.add_error_message(error.to_string());
    let rendered = drain_insert_history(&mut events)
        .iter()
        .map(|lines| lines_to_single_string(lines))
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("airs_application_policy_failure", rendered);
    assert_eq!(chat.bottom_pane.composer_text(), "Keep my unsent question");
    assert!(!chat.input_queue.authentication_pending);
    assert!(!rendered.contains("/signin"));
    assert!(ops.try_recv().is_err());
}
