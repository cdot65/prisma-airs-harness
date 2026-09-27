//! Shared bounds for owned-transcript action previews; expansion uses retained source content.

use crate::line_truncation::truncate_line_with_ellipsis_if_overflow;
use crate::terminal_hyperlinks::HyperlinkLine;
use crate::terminal_hyperlinks::LogicalLineSource;
use ratatui::text::Line;

pub(crate) const DETAIL_PREVIEW_LINES: usize = 3;

/// Clip a preview row without teaching selection to copy text that is currently hidden.
pub(crate) fn clipped_line(line: Line<'static>, width: u16) -> HyperlinkLine {
    truncate_line_with_ellipsis_if_overflow(line, usize::from(width)).into()
}

/// Clip a row while retaining only visible content as copy source. The caller supplies
/// the decorative prefix explicitly; real output that resembles a gutter stays intact.
pub(crate) fn clipped_prefixed_line(
    prefix: Line<'static>,
    content: Line<'static>,
    width: u16,
) -> HyperlinkLine {
    let prefix_width = prefix.width();
    let prefix_spans = prefix.spans.len();
    let content_style = content.style;
    let mut combined = content;
    combined.spans.splice(0..0, prefix.spans);
    let mut clipped = clipped_line(combined, width);
    let body = if usize::from(width) <= prefix_width {
        Line::default()
    } else {
        Line::from(
            clipped
                .line
                .spans
                .iter()
                .skip(prefix_spans)
                .cloned()
                .collect::<Vec<_>>(),
        )
        .style(content_style)
    };
    let mut source = LogicalLineSource::from_line(&body);
    source.prefix_bytes = clipped
        .line
        .to_string()
        .len()
        .saturating_sub(body.to_string().len());
    clipped.source = Some(source);
    clipped
}

#[cfg(test)]
#[path = "activity_preview_tests.rs"]
mod tests;
