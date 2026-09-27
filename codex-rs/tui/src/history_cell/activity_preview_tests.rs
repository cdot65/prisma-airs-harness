use super::*;
use pretty_assertions::assert_eq;
use ratatui::style::Stylize;

#[test]
fn compact_copy_excludes_only_explicit_gutters_and_hidden_text() {
    for body in ["  └ real output", "界 👩‍💻 e\u{301}  tail", "short"] {
        for width in 0..40 {
            let prefix = Line::from("  └ ".dim());
            let content = Line::from(body.cyan());
            let mut original = content.clone();
            original.spans.insert(0, "  └ ".dim());
            let expected = clipped_line(original, width);
            let actual = clipped_prefixed_line(prefix, content, width);
            assert_eq!(actual.line, expected.line);
            assert!(actual.width() <= usize::from(width));
            let source = actual.source.as_ref().unwrap();
            let visible = actual.line.to_string();
            assert_eq!(
                &visible[source.prefix_bytes..],
                &source.text[source.range.clone()]
            );
            assert_eq!(source.range, 0..source.text.len());
            if width <= 4 {
                assert_eq!(&*source.text, "");
            } else {
                assert!(visible.starts_with("  └ "));
                assert_eq!(source.prefix_bytes, "  └ ".len());
            }
        }
    }
}

#[test]
fn compact_copy_retains_meaningful_status_and_styles() {
    let line = clipped_prefixed_line(
        Line::from(vec!["•".green(), " ".into()]),
        Line::from(vec!["Failed (exit 2) ".bold(), "command".cyan()]),
        80,
    );
    let source = line.source.unwrap();
    assert_eq!(&*source.text, "Failed (exit 2) command");
    assert_eq!(
        source.styled_range(source.range.clone()),
        Line::from(vec!["Failed (exit 2) ".bold(), "command".cyan()])
    );
}

#[test]
fn terminal_bookkeeping_is_retained_in_transcript_and_raw_history() {
    use crate::history_cell::HistoryCell;
    use crate::history_cell::new_unified_exec_interaction;
    let cell = new_unified_exec_interaction(Some("cat".into()), "  └ actual input\nsecond".into());
    assert!(cell.display_lines(/*width*/ 40).is_empty());
    let lines = cell.transcript_hyperlink_lines(/*width*/ 40);
    let sources = lines
        .iter()
        .filter_map(|line| line.source.as_ref())
        .map(|source| source.text[source.range.clone()].to_owned())
        .collect::<Vec<_>>();
    assert!(sources.iter().any(|text| text == "  └ actual input"));
    assert!(
        cell.raw_lines()
            .iter()
            .any(|line| line.to_string() == "second")
    );
}
