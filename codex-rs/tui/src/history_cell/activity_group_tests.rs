use super::*;
use crate::history_cell::HistoryRenderMode;
use crate::history_cell::PlainHistoryCell;
use crate::history_cell::ReasoningSummaryCell;
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
    older.push_detail(Arc::<ReasoningSummaryCell>::from(
        new_reasoning_summary_block(vec!["older reasoning".into()], Path::new("/tmp")),
    ));
    let mut current = ActivityGroup::new(vec!["current command"]);
    current.push_detail(Arc::new(PlainHistoryCell::new(vec![Line::from(
        "terminal input",
    )])));
    current.push_detail(Arc::<ReasoningSummaryCell>::from(
        new_reasoning_summary_block(vec!["current reasoning".into()], Path::new("/tmp")),
    ));
    current.prepend(older);

    assert_eq!(current.calls, vec!["older command", "current command"]);
    let render = |position, mode| {
        current
            .details
            .lines_after(position, 80, mode)
            .iter()
            .map(|line| line.line.to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        render(1, HistoryRenderMode::Rich),
        vec!["", "• older reasoning"]
    );
    assert_eq!(
        render(2, HistoryRenderMode::Rich),
        vec!["", "terminal input", "", "• current reasoning"]
    );
    assert_eq!(render(1, HistoryRenderMode::Raw), Vec::<String>::new());
    assert_eq!(render(2, HistoryRenderMode::Raw), vec!["terminal input"]);
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
