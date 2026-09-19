use super::*;
use crate::airs_doctor::Event;
use crate::airs_doctor::report;
use crate::airs_doctor::views;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn report_save_dispatch_rejects_wrong_thread_and_stale_view() -> Result<()> {
    let (mut app, mut events, mut ops) = make_test_app_with_channels().await;
    let directory = tempfile::tempdir()?;
    app.config.codex_home = directory.path().to_path_buf().abs();
    let thread = ThreadId::new();
    app.active_thread_id = Some(thread);
    app.chat_widget.insert_str("Keep this private draft");
    let draft = app.chat_widget.airs_mcp_draft();
    while events.try_recv().is_ok() {}
    let text = report::render(None);
    let view = views::report_actions(text.clone(), Some(thread), "");
    (view.items[2].actions[0])(&app.app_event_tx);
    app.chat_widget.show_airs_doctor(view);
    let session = std::iter::from_fn(|| events.try_recv().ok())
        .find_map(|event| match event {
            AppEvent::AirsDoctor(Event::Report {
                session,
                action: report::Action::Save,
            }) => Some(session),
            _ => None,
        })
        .expect("save action from live view");
    let mut tui = crate::tui::test_support::make_test_tui()?;

    app.handle_airs_report(&mut tui, session.clone(), report::Action::Preview);
    let overlay = app.overlay.as_mut().expect("report preview");
    overlay.handle_event(
        &mut tui,
        crate::tui::TuiEvent::Key(KeyEvent::from(KeyCode::Esc)),
    )?;
    assert!(overlay.is_done());
    app.overlay = None;
    assert!(app.chat_widget.airs_report_view_active());
    assert!(session.allows(Some(thread)));

    app.active_thread_id = Some(ThreadId::new());
    app.handle_airs_report(&mut tui, session.clone(), report::Action::Save);
    assert_eq!(std::fs::read_dir(directory.path())?.count(), 0);

    app.active_thread_id = Some(thread);
    app.handle_airs_report(&mut tui, session.clone(), report::Action::Save);
    let files = std::fs::read_dir(directory.path())?.collect::<std::io::Result<Vec<_>>>()?;
    assert_eq!(files.len(), 1);
    assert_eq!(std::fs::read_to_string(files[0].path())?, text.as_ref());

    // Successful save replaces the popup. An old queued event must not save again.
    app.handle_airs_report(&mut tui, session, report::Action::Save);
    assert_eq!(std::fs::read_dir(directory.path())?.count(), 1);
    app.chat_widget.dismiss_airs_doctor();
    assert_eq!(app.chat_widget.airs_mcp_draft(), draft);
    while let Ok(event) = events.try_recv() {
        assert!(!matches!(
            event,
            AppEvent::InsertHistoryCell(_)
                | AppEvent::CodexOp(_)
                | AppEvent::AirsDoctor(_)
                | AppEvent::AirsMcpManager(_)
        ));
    }
    assert!(ops.try_recv().is_err());
    Ok(())
}
