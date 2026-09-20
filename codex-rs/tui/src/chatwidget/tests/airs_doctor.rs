use super::*;
use crate::airs_doctor::Check;
use crate::airs_doctor::Report;
use crate::airs_doctor::views;
use crate::render::renderable::Renderable;
use ratatui::backend::TestBackend;

#[tokio::test]
async fn doctor_mcp_status_keeps_runtime_failure_separate_from_cached_discovery() {
    use codex_app_server_protocol::McpServerConnectionStatus as State;
    let (mut chat, _rx, _ops) = make_chatwidget_manual(None).await;
    chat.config
        .mcp_servers
        .set(std::collections::HashMap::from([(
            "service-now".into(),
            serde_json::from_value(serde_json::json!({"url":"https://gateway.example/mcp"}))
                .unwrap(),
        )]))
        .unwrap();
    let mut status: codex_app_server_protocol::McpServerStatus =
        serde_json::from_value(serde_json::json!({
            "name":"service-now", "serverInfo":{"name":"gateway", "version":"1"},
            "tools":{}, "resources":[], "resourceTemplates":[], "authStatus":"unknown",
        }))
        .unwrap();
    for (runtime, error, expected) in [
        (None, None, "0 tools discovered · runtime unchecked"),
        (Some(State::Failed), None, "Not connected · select to retry"),
        (
            Some(State::AuthenticationRequired),
            None,
            "Sign-in required",
        ),
        (
            Some(State::Connected),
            Some("PRIVATE-DISCOVERY-ERROR"),
            "Connected · tool discovery failed",
        ),
        (Some(State::Connected), None, "Connected · 0 tools"),
    ] {
        status.runtime_status = runtime;
        status.tools_error = error.map(str::to_owned);
        let connections = crate::airs_mcp_manager::connections(&chat.config, &[status.clone()]);
        assert_eq!(
            connections[0].status,
            format!("{expected} · Authentication unknown")
        );
    }
}

#[tokio::test]
async fn doctor_views_preserve_draft_and_do_not_submit_model_actions() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(None).await;
    chat.insert_str("Keep this unsent request");
    for (name, width, report) in [
        ("airs_doctor_healthy", 110, Ok(Report {
            schema_version: 1, product: "Prisma AIRS Harness".into(), authentication: "Company SSO".into(),
            checks: vec![Check { name: "gateway_access".into(), passed: true, detail: "Gateway access verified. Client correlation ID: fixture".into() }],
        })),
        ("airs_doctor_degraded_narrow", 55, Ok(Report {
            schema_version: 1, product: "Prisma AIRS Harness".into(), authentication: "Workspace API key".into(),
            checks: vec![Check { name: "credential_service".into(), passed: false, detail: "Credential service unavailable in this session. Check your signed-in user's credential service and session bus, then retry.".into() }],
        })),
        ("airs_doctor_unavailable", 88, Err("Diagnostics timed out. Retry /doctor.".into())),
    ] {
        chat.show_airs_doctor(views::overview("work", report, vec![crate::airs_mcp_manager::Connection {
            name: "service-now".into(), url: "https://gateway.example/mcp".into(),
            status: "Not verified · select to reconnect".into(), can_login: true,
        }], /*thread*/ None));
        insta::assert_snapshot!(name, render_doctor(&chat, width));
        chat.dismiss_airs_doctor();
        assert_eq!(chat.bottom_pane.composer_text(), "Keep this unsent request");
    }
    for (name, view) in [
        ("airs_doctor_consent", views::confirm_verify()),
        ("airs_doctor_waiting", views::waiting(7)),
    ] {
        chat.show_airs_doctor(view);
        insta::assert_snapshot!(name, render_doctor(&chat, 55));
        chat.dismiss_airs_doctor();
        assert_eq!(chat.bottom_pane.composer_text(), "Keep this unsent request");
    }
    while let Ok(event) = rx.try_recv() {
        assert!(!matches!(
            event,
            AppEvent::CodexOp(_)
                | AppEvent::AirsSignIn
                | AppEvent::AirsMcpManager(_)
                | AppEvent::AirsDoctor(_)
        ));
    }
    chat.input_queue.user_turn_pending_start = true;
    assert!(!chat.airs_doctor_ready());
}

fn render_doctor(chat: &ChatWidget, width: u16) -> String {
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

#[tokio::test]
async fn report_copy_failure_keeps_lease_draft_and_local_fallback() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(None).await;
    chat.insert_str("Private unsent draft");
    let text = crate::airs_doctor::report::render(None);
    let status = chat.copy_airs_report_with(&text, |copied| {
        assert_eq!(copied, text.as_ref());
        Ok(Some(crate::clipboard_copy::ClipboardLease::test()))
    });
    assert!(chat.clipboard_lease.is_some());
    chat.show_airs_doctor(views::report_actions(
        text.clone(),
        chat.thread_id(),
        status,
    ));
    insta::assert_snapshot!("airs_report_clipboard_sent", render_doctor(&chat, 88));
    let status = chat.copy_airs_report_with(&text, |_| Err("PRIVATE-BACKEND-ERROR".into()));
    assert!(chat.clipboard_lease.is_some());
    chat.show_airs_doctor(views::report_actions(text, chat.thread_id(), status));
    insta::assert_snapshot!(
        "airs_report_clipboard_failure_narrow",
        render_doctor(&chat, 55)
    );
    assert!(!render_doctor(&chat, 55).contains("PRIVATE-BACKEND-ERROR"));
    chat.dismiss_airs_doctor();
    assert_eq!(chat.bottom_pane.composer_text(), "Private unsent draft");
    assert!(!chat.airs_report_view_active());
    while let Ok(event) = rx.try_recv() {
        assert!(!matches!(
            event,
            AppEvent::InsertHistoryCell(_)
                | AppEvent::CodexOp(_)
                | AppEvent::AirsDoctor(_)
                | AppEvent::AirsMcpManager(_)
        ));
    }
}

#[tokio::test]
async fn report_actions_and_local_save_status_remain_private_views() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(None).await;
    let text = crate::airs_doctor::report::render(None);
    for (name, width, status) in [
        ("airs_report_actions", 88, ""),
        ("airs_report_actions_narrow", 55, ""),
        (
            "airs_report_saved",
            88,
            "Saved locally: /fixture/diagnostic-report-fixture.txt",
        ),
        (
            "airs_report_save_failed",
            55,
            "Could not save the report. Preview it or retry in a writable environment directory.",
        ),
    ] {
        chat.show_airs_doctor(views::report_actions(
            text.clone(),
            chat.thread_id(),
            status,
        ));
        insta::assert_snapshot!(name, render_doctor(&chat, width));
        chat.dismiss_airs_doctor();
    }
    while let Ok(event) = rx.try_recv() {
        assert!(!matches!(
            event,
            AppEvent::InsertHistoryCell(_)
                | AppEvent::CodexOp(_)
                | AppEvent::AirsDoctor(_)
                | AppEvent::AirsMcpManager(_)
        ));
    }
}
