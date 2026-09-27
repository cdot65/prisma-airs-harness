use super::*;
use crate::history_cell::HistoryRenderMode;
use crate::history_cell::PlainHistoryCell;
use crate::history_cell::WebHyperlinkHistoryCell;
use crate::history_cell::new_reasoning_summary_block;
use crate::terminal_hyperlinks::HyperlinkLine;
use crate::terminal_hyperlinks::LogicalLineSource;
use crate::terminal_hyperlinks::TerminalHyperlink;
use crate::terminal_hyperlinks::lines_with_sources_eq;
use pretty_assertions::assert_eq;
use ratatui::text::Line;
use std::path::Path;

#[test]
fn prepending_history_preserves_detail_order_and_raw_contract() {
    let mut older = ActivityGroup::new(vec!["older command"]);
    older.push_reasoning(new_reasoning_summary_block(
        vec!["older reasoning".into()],
        Path::new("/tmp"),
    ));
    let mut current = ActivityGroup::new(vec!["current command"]);
    current.push_detail(Arc::new(PlainHistoryCell::new(vec![Line::from(
        "terminal input",
    )])));
    current.push_reasoning(new_reasoning_summary_block(
        vec!["current reasoning".into()],
        Path::new("/tmp"),
    ));
    current.prepend(older);

    let render = |mode| {
        current
            .transcript_lines(/*width*/ 80, mode, |_, call, lines| {
                lines.push(Line::from((*call).to_owned()));
            })
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        render(HistoryRenderMode::Rich),
        vec![
            "older command",
            "",
            "• older reasoning",
            "current command",
            "",
            "terminal input",
            "",
            "• current reasoning",
        ]
    );
    assert_eq!(
        render(HistoryRenderMode::Raw),
        vec!["older command", "current command", "terminal input"]
    );
}

#[test]
fn prepending_history_retains_shared_source_and_link_destinations() {
    let line = Line::from("linked detail");
    let source = LogicalLineSource::from_line(&line);
    let detail = HyperlinkLine {
        line,
        hyperlinks: vec![TerminalHyperlink::web(
            0..6,
            "https://example.test/details".into(),
        )],
        source: Some(source.clone()),
    };
    let mut current = ActivityGroup::new(vec!["first", "second"]);
    current.push_detail(Arc::new(WebHyperlinkHistoryCell::new_hyperlink_lines(
        vec![detail],
    )));
    let before = current.details.lines_after(
        /*after_calls*/ 2,
        /*width*/ 80,
        HistoryRenderMode::Rich,
    );
    current.prepend(ActivityGroup::new(vec!["older"]));
    let after = current.details.lines_after(
        /*after_calls*/ 3,
        /*width*/ 80,
        HistoryRenderMode::Rich,
    );
    assert!(lines_with_sources_eq(&before, &after));
    assert!(Arc::ptr_eq(
        &source.text,
        &after[1].source.as_ref().unwrap().text
    ));
    assert!(
        current
            .details
            .lines_after(
                /*after_calls*/ 2,
                /*width*/ 80,
                HistoryRenderMode::Rich
            )
            .is_empty()
    );
}
