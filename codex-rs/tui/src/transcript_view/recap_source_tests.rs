//! The actual AIRS recap cell keeps its labeled appearance and copyable logical body.
use super::*;
use crate::history_cell::HistoryCell;
use crate::history_cell::ThreadRecapHistoryCell;
use pretty_assertions::assert_eq;

#[test]
fn recap_copy_joins_soft_wraps_without_changing_checkpoint_rendering() {
    let body = "Preserve the existing checkpoint layout and source spacing.\n\nNext: visit https://a.io and run the focused tests.";
    let cell = ThreadRecapHistoryCell::new(body.into());
    let mut frames = Vec::new();
    for width in [24, 40, 80] {
        let lines = cell.transcript_hyperlink_lines(width);
        let heading = lines[0].line.to_string();
        let layout = TextLayout::new(lines.clone(), width);
        let expected_source = format!("{heading}\n\n{body}");
        assert_eq!(layout.text(), expected_source);
        let start = layout.position_at(/*row*/ 2, /*column*/ 2);
        let end = layout.position_at(layout.row_count() - 1, u16::MAX);
        assert_eq!(&layout.text()[start..end], body);
        for resized in [17, 36, 90] {
            assert_eq!(layout.rewrap(resized).text(), expected_source);
        }
        let area = Rect::new(/*x*/ 0, /*y*/ 0, width, layout.row_count() as u16);
        let mut expected = Buffer::empty(area);
        HyperlinkParagraph::new(&lines, Style::default()).render(area, &mut expected);
        let mut actual = Buffer::empty(area);
        layout.render(area, &mut actual, /*start_row*/ 0);
        assert_eq!(actual, expected);
        frames.push(format!("width={width}\n{actual:?}"));
    }
    assert_eq!(
        cell.raw_lines()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n"),
        format!("Conversation recap\n{body}")
    );
    insta::assert_snapshot!(frames.join("\n\n"));
}

#[test]
fn recap_source_preserves_unicode_whitespace_and_long_url_across_resize() {
    let url = "https://example.invalid/projects/harness/releases/preview/checkpoint/source";
    let body =
        format!("AIRS  e\u{301} 👩‍💻 日本語\n\n  preserved indentation\n\tcolumn\tvalue\n{url}");
    let cell = ThreadRecapHistoryCell::new(body.clone());
    for width in [24, 48, 80] {
        let lines = cell.transcript_hyperlink_lines(width);
        let expected = format!("{}\n\n{body}", lines[0].line);
        let layout = TextLayout::new(lines, width);
        assert_eq!(layout.text(), expected);
        for resized in [12, 32, 100] {
            let resized = layout.rewrap(resized);
            assert_eq!(resized.text(), expected);
            let destinations: std::collections::HashSet<_> = resized
                .rows
                .iter()
                .flat_map(|row| {
                    row.line
                        .hyperlinks
                        .iter()
                        .map(|link| link.destination.as_str())
                })
                .collect();
            assert_eq!(destinations, std::collections::HashSet::from([url]));
            assert!(
                resized
                    .rows
                    .iter()
                    .all(|row| row.line.width() <= usize::from(resized.width))
            );
        }
    }
}
