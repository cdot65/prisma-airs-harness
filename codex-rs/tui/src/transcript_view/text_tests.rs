use super::*;
use pretty_assertions::assert_eq;
use ratatui::style::Stylize;
use ratatui::text::Line;

fn scrolled_hyperlink_lines() -> Vec<HyperlinkLine> {
    let mut linked = HyperlinkLine::new(Line::from(vec![
        "prefix ".green(),
        "漢字 ".bold(),
        "ｶﾞ ".italic(),
    ]));
    linked.push_span(
        "clickable 漢字 ｶﾞ".cyan().underlined(),
        Some("https://example.com/a/wrapped-private-destination"),
    );
    linked.push_span(
        " suffix words that continue wrapping".blue(),
        /*destination*/ None,
    );

    vec![
        HyperlinkLine::new(Line::from("first styled row".magenta())),
        linked,
        HyperlinkLine::new(Line::default()),
        HyperlinkLine::new(Line::from("last 漢字 ｶﾞ row".red())),
    ]
}

#[test]
fn scrolled_text_layout_matches_full_paragraph_for_links_styles_and_wide_glyphs() {
    let lines = scrolled_hyperlink_lines();
    for width in [5, 7, 13, 28] {
        let layout = TextLayout::new(lines.clone(), width);
        for offset in [
            0,
            1,
            2,
            3,
            layout.row_count().saturating_sub(1),
            layout.row_count(),
        ] {
            for (x, y, visible_height) in [(0, 0, 4), (3, 2, 3)] {
                let area = Rect::new(x, y, width, visible_height);
                let canvas = Rect::new(/*x*/ 0, /*y*/ 0, area.right(), area.bottom());
                let mut actual = Buffer::empty(canvas);
                let mut expected = Buffer::empty(canvas);
                let full_area =
                    Rect::new(/*x*/ 0, /*y*/ 0, width, layout.row_count() as u16);
                let mut full = Buffer::empty(full_area);
                HyperlinkParagraph::new(&lines, Style::default()).render(full_area, &mut full);
                for row in
                    0..usize::from(visible_height).min(layout.row_count().saturating_sub(offset))
                {
                    for column in 0..width {
                        expected[(x + column, y + row as u16)] =
                            full[(column, (offset + row) as u16)].clone();
                    }
                }
                layout.render(area, &mut actual, offset);
                assert_eq!(
                    actual, expected,
                    "width={width}, offset={offset}, area={area:?}"
                );
            }
        }
    }
}
