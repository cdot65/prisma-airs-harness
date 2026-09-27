//! Unsafe remote prompt restoration must not modify the draft or start a branch.
use super::session_lifecycle_requests::recorded_params;
use super::session_lifecycle_requests::start_recording_app_server;
use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn prompt_edit_remote_commands_and_local_images_leave_draft_and_backend_untouched()
-> Result<()> {
    let (mut app, mut events, mut ops) = make_test_app_with_channels().await;
    let (mut server, requests, proxy) = start_recording_app_server(
        &app.config,
        /*blocked_thread_list*/ None,
        /*failed_thread_name*/ None,
    )
    .await?;
    let thread_id = ThreadId::new();
    app.chat_widget
        .handle_thread_session(test_thread_session(thread_id, app.config.cwd.to_path_buf()));
    app.app_server_target = crate::AppServerTarget::Remote {
        endpoint: crate::resolve_remote_addr("ws://127.0.0.1:8765")?,
    };
    app.chat_widget
        .apply_external_edit("keep this draft".into());
    while events.try_recv().is_ok() {}
    while ops.try_recv().is_ok() {}
    let mut tui = crate::tui::test_support::make_test_tui()?;
    for (text, local_images) in [
        (" /model", Vec::new()),
        (" !echo example", Vec::new()),
        (
            "image prompt",
            vec![crate::bottom_pane::LocalImageAttachment {
                placeholder: "[Image #1]".into(),
                path: PathBuf::from("/tmp/remote-image.png"),
            }],
        ),
    ] {
        let mut prompt = crate::chatwidget::UserMessage::from(text);
        prompt.local_images = local_images;
        let selected_cell: Arc<dyn HistoryCell> = Arc::new(crate::history_cell::new_user_prompt(
            text.into(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ));
        app.transcript_cells = vec![Arc::clone(&selected_cell)];
        app.handle_event(
            &mut tui,
            &mut server,
            AppEvent::ForkSessionForPromptEdit {
                thread_id,
                selected_cell,
                prompt,
            },
        )
        .await?;
        assert_eq!(app.chat_widget.thread_id(), Some(thread_id));
        assert_eq!(
            app.chat_widget.composer_text_with_pending(),
            "keep this draft"
        );
        assert!(ops.try_recv().is_err());
        assert!(next_history_message(&mut events).contains("cannot be restored safely"));
    }
    assert_eq!(
        recorded_params(&requests, "thread/fork"),
        Vec::<serde_json::Value>::new()
    );
    assert_eq!(
        recorded_params(&requests, "thread/start"),
        Vec::<serde_json::Value>::new()
    );
    server.shutdown().await?;
    proxy.await??;
    Ok(())
}
