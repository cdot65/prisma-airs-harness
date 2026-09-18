use super::*;
use pretty_assertions::assert_eq;

#[test]
fn airs_resume_summary_retains_environment_and_user_supplied_name() {
    let thread_id = ThreadId::from_string("01a0b3f4-1b0f-7cf2-8189-01f3dc2bc318").unwrap();
    let info = AppExitInfo {
        token_usage: TokenUsage::default(),
        thread_id: Some(thread_id),
        resume_hint: Some(ResumableThread {
            thread_id,
            thread_name: Some("codex and airs-harness migration".to_string()),
        }),
        disconnect_info: None,
        update_action: None,
        exit_reason: ExitReason::UserRequested,
    };
    let command = crate::airs_branding::command_for_environment("airs", Some("work"));
    let output = info.format_exit_messages_with_command(/*color_enabled*/ false, &command);
    insta::assert_snapshot!(output.join("\n"), @"
    To continue this session, run:
      airs --environment work resume 01a0b3f4-1b0f-7cf2-8189-01f3dc2bc318
    Or run airs --environment work resume and select codex and airs-harness migration.
    ");
}

#[test]
fn airs_resume_summary_quotes_environment_before_coloring_command() {
    let thread_id = ThreadId::from_string("01a0b3f4-1b0f-7cf2-8189-01f3dc2bc318").unwrap();
    let info = AppExitInfo {
        token_usage: TokenUsage::default(),
        thread_id: Some(thread_id),
        resume_hint: Some(ResumableThread {
            thread_id,
            thread_name: None,
        }),
        disconnect_info: None,
        update_action: None,
        exit_reason: ExitReason::UserRequested,
    };
    let command = crate::airs_branding::command_for_environment("airs", Some("work team"));
    assert_eq!(
        info.format_exit_messages_with_command(/*color_enabled*/ true, &command),
        vec![
            "To continue this session, run:".to_string(),
            "  \u{1b}[36mairs --environment 'work team' resume 01a0b3f4-1b0f-7cf2-8189-01f3dc2bc318\u{1b}[39m".to_string(),
        ],
    );
}
