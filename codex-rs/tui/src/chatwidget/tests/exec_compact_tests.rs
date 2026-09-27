use super::*;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn compact_search_reports_nonzero_status_without_false_failure_count() {
    let mut snapshots = Vec::new();
    for exit_code in [0, 1, 2] {
        let (mut chat, _rx, _op_rx) = make_chatwidget_manual(/*model_override*/ None).await;
        let item = begin_exec(&mut chat, "search", "rg absent .");
        end_exec(&mut chat, item, "", "", exit_code);
        let cell = chat.transcript.active_cell.as_ref().unwrap();
        let lines = cell.compact_hyperlink_lines(/*width*/ 80);
        assert_eq!(cell.activity_ids(), vec!["exec:search"]);
        assert_eq!(
            lines[0].line.to_string(),
            if exit_code == 2 {
                "• Explored · 1 failed"
            } else {
                "• Explored"
            }
        );
        snapshots.push(format!(
            "exit {exit_code}:\n{}",
            lines
                .iter()
                .map(|line| line.line.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    insta::assert_snapshot!(snapshots.join("\n\n"));
}
