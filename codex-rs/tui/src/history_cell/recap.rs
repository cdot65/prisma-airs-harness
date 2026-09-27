//! AIRS checkpoint recaps retain their established appearance and exact body provenance.

use super::*;

const RECAP_HEADING: &str = "Conversation recap";

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug)]
pub(crate) struct ThreadRecapHistoryCell {
    recap: String,
}

#[cfg_attr(not(test), allow(dead_code))]
impl ThreadRecapHistoryCell {
    pub(crate) fn new(recap: String) -> Self {
        Self { recap }
    }
}

impl HistoryCell for ThreadRecapHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        visible_lines(self.display_hyperlink_lines(width))
    }

    fn display_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let width = usize::from(width);
        let mut remaining_width = width;
        let mut heading = Vec::new();

        if remaining_width > 0 {
            heading.push("─".dim());
            remaining_width -= 1;
        }
        if remaining_width > 0 {
            heading.push(" ".dim());
            remaining_width -= 1;
        }

        let (visible_heading, _suffix, heading_width) =
            take_prefix_by_width(RECAP_HEADING, remaining_width);
        if !visible_heading.is_empty() {
            heading.push(visible_heading.bold());
            remaining_width -= heading_width;
        }
        if remaining_width > 0 {
            heading.push(" ".dim());
            remaining_width -= 1;
        }
        if remaining_width > 0 {
            heading.push("─".repeat(remaining_width).dim());
        }

        let wrap_width = width.saturating_sub(2).max(1);
        let mut body =
            crate::terminal_hyperlinks::annotate_web_urls(raw_lines_from_source(&self.recap));
        for line in &mut body {
            let mut source = crate::terminal_hyperlinks::LogicalLineSource::from_line(&line.line);
            source.wrap_policy = crate::terminal_hyperlinks::LineWrapPolicy::UrlAware;
            line.source = Some(source);
        }
        let wrapped = crate::terminal_hyperlinks::adaptive_wrap_hyperlink_lines(
            &body,
            RtOptions::new(wrap_width),
        );
        // The labeled rule stays visible/copyable exactly as painted. Body gutters are
        // presentation only; their provenance joins soft wraps without losing hard breaks.
        let mut lines = vec![HyperlinkLine::new(heading.into()), HyperlinkLine::default()];
        lines.extend(crate::terminal_hyperlinks::prefix_hyperlink_lines(
            wrapped,
            "  ".into(),
            "  ".into(),
        ));

        lines
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        self.display_hyperlink_lines(width)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from(RECAP_HEADING)];
        lines.extend(raw_lines_from_source(&self.recap));
        lines
    }
}
