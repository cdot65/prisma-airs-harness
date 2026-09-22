use super::*;
use crate::airs_doctor::Check;
use crate::airs_doctor::Event;
use crate::airs_doctor::views;
use crate::app_event::AppEvent;
use pretty_assertions::assert_eq;

#[test]
fn report_ignores_private_fields_and_uses_pessimistic_known_checks() {
    let mut report = Report {
        schema_version: 1,
        product: "PRIVATE-PRODUCT".into(),
        authentication: "PRIVATE-AUTH\u{202e}\x1b".into(),
        checks: vec![
            Check {
                name: "credential_service".into(),
                passed: false,
                detail: "PRIVATE-KEYRING-ERROR".into(),
            },
            Check {
                name: "configuration".into(),
                passed: true,
                detail: "https://PRIVATE-GATEWAY/path?code=PRIVATE-CODE".into(),
            },
            Check {
                name: "gateway_health".into(),
                passed: true,
                detail: "PRIVATE-HEADER".into(),
            },
            Check {
                name: "gateway_health".into(),
                passed: false,
                detail: "PRIVATE-ENVIRONMENT".into(),
            },
            Check {
                name: "PRIVATE-CHECK".into(),
                passed: false,
                detail: "PRIVATE-CONVERSATION".into(),
            },
        ],
    };
    let rendered = render(Some(&report));
    assert!(!rendered.contains("PRIVATE"));
    assert!(!rendered.contains("https://"));
    assert!(!rendered.contains('\u{202e}'));
    let normalized = rendered
        .replace(codex_utils_home_dir::AIRS_HARNESS_VERSION, "[VERSION]")
        .replace(
            &format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
            "[PLATFORM]",
        );
    insta::assert_snapshot!("airs_report_redacted_failure", normalized);
    for check in &mut report.checks {
        check.detail = "different secret /home/private/token".into();
    }
    report.authentication = "another unexpected auth value".into();
    report.product = "another private product".into();
    assert_eq!(render(Some(&report)), rendered);
}

#[test]
fn explicit_access_and_authentication_are_distinct_from_health() {
    let report = Report {
        schema_version: 1,
        product: "Prisma AIRS Harness".into(),
        authentication: "Workspace API key".into(),
        checks: vec![Check {
            name: "gateway_access".into(),
            passed: true,
            detail: "private correlation ID".into(),
        }],
    };
    let text = render(Some(&report));
    assert!(text.contains("Authentication: Workspace API key\n"));
    assert!(text.contains("gateway_access: PASS\n"));
    assert!(!text.contains("Not verified"));
    let unavailable = render(None);
    assert!(unavailable.contains("Authentication: Unknown\n"));
    assert!(unavailable.contains("gateway_access: Not verified\n"));
    assert!(unavailable.contains("Diagnostics unavailable. Retry /doctor."));
}

#[test]
fn report_actions_keep_snapshot_and_expire_with_view_or_thread() {
    let thread = ThreadId::new();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let tx = crate::app_event_sender::AppEventSender::new(tx);
    let text = render(None);
    let view = views::report_actions(text.clone(), Some(thread), "");
    for item in &view.items {
        assert!(!item.dismiss_on_select);
        (item.actions[0])(&tx);
    }
    let mut sessions = Vec::new();
    for expected in [Action::Preview, Action::Copy, Action::Save, Action::Close] {
        let AppEvent::AirsDoctor(Event::Report { session, action }) = rx.try_recv().unwrap() else {
            panic!("report must never request inspection, inference or MCP operations");
        };
        assert!(std::mem::discriminant(&expected) == std::mem::discriminant(&action));
        assert!(Arc::ptr_eq(&session.text, &text));
        assert!(session.allows(Some(thread)));
        assert!(!session.allows(Some(ThreadId::new())));
        assert!(!session.allows(None));
        sessions.push(session);
    }
    assert!(rx.try_recv().is_err());
    drop(view);
    assert!(sessions.iter().all(|session| !session.allows(Some(thread))));
}

#[test]
fn overview_replacement_expires_queued_export_action() {
    let thread = ThreadId::new();
    let view = views::overview(
        "private-environment",
        Err("private error".into()),
        Vec::new(),
        Some(thread),
    );
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let tx = crate::app_event_sender::AppEventSender::new(tx);
    let export = view
        .items
        .iter()
        .find(|item| item.name == "Diagnostic report")
        .unwrap();
    (export.actions[0])(&tx);
    let AppEvent::AirsDoctor(Event::Report {
        session,
        action: Action::Open,
    }) = rx.try_recv().unwrap()
    else {
        panic!("expected private report action")
    };
    assert!(session.allows(Some(thread)));
    assert!(!session.text.contains("private"));
    drop(view);
    assert!(!session.allows(Some(thread)));
}

#[test]
fn local_save_preserves_existing_state_and_rejects_invalid_destination() {
    let directory = tempfile::tempdir().unwrap();
    let sentinel = directory.path().join("config.toml");
    std::fs::write(&sentinel, "private existing configuration").unwrap();
    let text = render(None);
    let first = save(directory.path(), &text).unwrap();
    let second = save(directory.path(), &text).unwrap();
    assert_ne!(first, second);
    assert_eq!(std::fs::read_to_string(&first).unwrap(), text.as_ref());
    assert_eq!(std::fs::read_to_string(&second).unwrap(), text.as_ref());
    assert_eq!(
        std::fs::read_to_string(&sentinel).unwrap(),
        "private existing configuration"
    );
    assert!(
        first
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("diagnostic-report-")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&first).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    assert!(save(&sentinel, &text).is_err());
    assert!(save(directory.path(), &"x".repeat(MAX_REPORT + 1)).is_err());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 3);
}

#[test]
fn threadless_report_callbacks_work_only_while_their_originating_view_survives() {
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let sender = crate::app_event_sender::AppEventSender::new(sender);
    let overview = views::overview(
        "work",
        Err("No conversation loaded".into()),
        Vec::new(),
        /*thread*/ None,
    );
    let export = overview
        .items
        .iter()
        .find(|item| item.name == "Diagnostic report")
        .unwrap();
    (export.actions[0])(&sender);
    let AppEvent::AirsDoctor(Event::Report {
        session,
        action: Action::Open,
    }) = receiver.try_recv().unwrap()
    else {
        panic!("expected report action");
    };
    assert!(session.allows(/*thread*/ None));
    let actions = views::report_actions(session.text.clone(), /*thread*/ None, "");
    drop(overview);
    assert!(!session.allows(/*thread*/ None));
    for item in &actions.items {
        (item.actions[0])(&sender);
    }
    let mut pending = Vec::new();
    for _ in &actions.items {
        let AppEvent::AirsDoctor(Event::Report { session, .. }) = receiver.try_recv().unwrap()
        else {
            panic!("expected report action");
        };
        assert!(session.allows(/*thread*/ None));
        assert!(!session.allows(Some(ThreadId::new())));
        pending.push(session);
    }
    drop(actions);
    assert!(
        pending
            .iter()
            .all(|session| !session.allows(/*thread*/ None))
    );
    assert!(receiver.try_recv().is_err());
}

#[test]
fn report_preview_wraps_in_existing_pager_at_narrow_width() {
    let text = render(None)
        .replace(codex_utils_home_dir::AIRS_HARNESS_VERSION, "[VERSION]")
        .replace(
            &format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH),
            "[PLATFORM]",
        );
    let crate::pager_overlay::Overlay::Static(mut overlay) =
        crate::pager_overlay::Overlay::new_static_with_lines(
            text.lines()
                .map(|line| ratatui::text::Line::from(line.to_owned()))
                .collect(),
            "Diagnostic report".into(),
            crate::keymap::RuntimeKeymap::defaults().pager,
        )
    else {
        panic!("expected static preview")
    };
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(
        /*width*/ 55, /*height*/ 24,
    ))
    .unwrap();
    terminal
        .draw(|frame| overlay.render(frame.area(), frame.buffer_mut()))
        .unwrap();
    insta::assert_snapshot!("airs_report_preview_narrow", terminal.backend());
}
