use super::*;
use crate::app::tests::session_lifecycle_requests::HistoryCapabilities;
use crate::app::tests::session_lifecycle_requests::recorded_params;
use crate::app::tests::session_lifecycle_requests::start_recording_app_server_with_history;
use crate::tui::test_support::make_test_tui;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn blank_task_navigation_preserves_unsubmitted_draft_and_live_subscription() -> Result<()> {
    let (mut app, _events, _ops) = make_test_app_with_channels().await;
    crate::legacy_core::config::set_project_trust_level(
        app.config.codex_home.as_path(),
        app.config.cwd.as_path(),
        codex_protocol::config_types::TrustLevel::Trusted,
    )
    .map_err(|error| color_eyre::eyre::eyre!(error.to_string()))?;
    app.app_server_target = AppServerTarget::Remote {
        endpoint: crate::resolve_remote_addr("ws://127.0.0.1:8765")?,
    };
    let (mut server, requests, proxy) = start_recording_app_server_with_history(
        &app.config,
        HistoryCapabilities::Current,
        /*blocked_thread_list*/ None,
        /*failed_thread_name*/ None,
        crate::app_server_session::ThreadParamsMode::Remote,
        LoaderOverrides::default(),
    )
    .await?;
    let mut tui = make_test_tui()?;
    tui.pause_events();
    let started = server.start_thread(&app.config).await?;
    let first = started.session.thread_id;
    app.pending_startup_thread_start = true;
    app.handle_startup_thread_started(&mut server, Ok(started))
        .await?;
    app.chat_widget.insert_str("Keep this unsent draft");
    server
        .thread_set_name(first, "Blank draft task".into())
        .await?;
    app.start_fresh_session_with_summary_hint(
        &mut tui,
        &mut server,
        /*session_start_source*/ None,
        /*initial_user_message*/ None,
        /*new_thread_name*/ None,
    )
    .await;
    let second = app.chat_widget.thread_id().unwrap();
    assert_ne!(first, second);
    app.select_agents_overview_thread(&mut tui, &mut server, first)
        .await?;
    assert_eq!(app.chat_widget.thread_id(), Some(first));
    assert_eq!(
        app.chat_widget.thread_name().as_deref(),
        Some("Blank draft task")
    );
    assert_eq!(
        app.chat_widget.composer_text_with_pending(),
        "Keep this unsent draft"
    );
    insta::assert_snapshot!(
        crate::chatwidget::tests::helpers::render_bottom_popup(&app.chat_widget, /*width*/ 80)
            .lines().next().unwrap(), @"› Keep this unsent draft");
    app.select_agents_overview_thread(&mut tui, &mut server, second)
        .await?;
    app.select_agents_overview_thread(&mut tui, &mut server, first)
        .await?;
    assert_eq!(
        app.chat_widget.composer_text_with_pending(),
        "Keep this unsent draft"
    );
    assert!(recorded_params(&requests, "thread/resume").is_empty());
    assert!(
        recorded_params(&requests, "thread/unsubscribe")
            .iter()
            .all(|params| params["threadId"] != first.to_string())
    );
    assert!(recorded_params(&requests, "turn/start").is_empty());
    server.shutdown().await?;
    proxy.await??;
    Ok(())
}
