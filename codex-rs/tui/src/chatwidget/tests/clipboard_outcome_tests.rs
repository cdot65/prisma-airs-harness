//! Copy delivery notices must not imply that SSH's terminal acknowledged receipt.

use super::*;
use crate::clipboard_copy::ClipboardLease;
use crate::clipboard_copy::CopyOutcome;

#[tokio::test]
async fn terminal_copy_preserves_native_ownership_draft_and_inference_state() {
    let (mut chat, mut rx, mut ops) = make_chatwidget_manual(None).await;
    chat.clipboard_lease = Some(ClipboardLease::test());
    chat.transcript.last_agent_markdown = Some("assistant reply".into());
    chat.insert_str("Unsent request");
    chat.copy_last_agent_markdown_with(|text| {
        assert_eq!(text, "assistant reply");
        Ok(CopyOutcome::Requested)
    });
    let first = drain_insert_history(&mut rx);
    assert_eq!(first.len(), 1);
    assert!(lines_to_single_string(&first[0]).contains("Copy unconfirmed"));
    assert!(chat.clipboard_lease.is_some());
    chat.copy_selection_with("selected text", "selection", |_| Ok(CopyOutcome::Requested));
    let second = drain_insert_history(&mut rx);
    insta::assert_snapshot!(
        "clipboard_terminal_unconfirmed",
        lines_to_single_string(&second[0])
    );
    assert_eq!(chat.bottom_pane.composer_text(), "Unsent request");
    assert!(chat.clipboard_lease.is_some());
    assert!(ops.try_recv().is_err());
}

#[tokio::test]
async fn diagnostic_terminal_copy_keeps_private_report_out_of_history() {
    let (mut chat, mut rx, mut ops) = make_chatwidget_manual(None).await;
    chat.clipboard_lease = Some(ClipboardLease::test());
    chat.insert_str("Unsent request");
    let status = chat.copy_airs_report_with("PRIVATE-REPORT", |text| {
        assert_eq!(text, "PRIVATE-REPORT");
        Ok(CopyOutcome::Requested)
    });
    assert!(status.contains("not confirmed"));
    assert!(!status.contains("PRIVATE-REPORT"));
    assert!(chat.clipboard_lease.is_some());
    assert_eq!(chat.bottom_pane.composer_text(), "Unsent request");
    assert!(drain_insert_history(&mut rx).is_empty());
    assert!(ops.try_recv().is_err());
}
