//! Bounded URL wrapping for viewports with source-aware selection.

use super::*;

/// Preserve fitting URL tokens while splitting oversized tokens within the requested width.
/// Source ranges and hanging indents survive the fallback, without terminal autowrap.
pub(crate) fn adaptive_wrap_line_to_width<'a>(
    line: &'a Line<'a>,
    options: RtOptions<'a>,
) -> Vec<WrappedLine<'a>> {
    let wrapped = adaptive_wrap_line_with_source(line, options.clone());
    if wrapped
        .iter()
        .any(|row| line_width(&row.line) > options.width)
    {
        word_wrap_line_with_source(
            line,
            url_preserving_wrap_options(options).break_words(/*break_words*/ true),
        )
    } else {
        wrapped
    }
}
