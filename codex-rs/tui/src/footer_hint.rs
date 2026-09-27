//! Choose an intact hint rather than clipping a shortcut in a narrow footer.
use ratatui::text::Line;
pub(crate) fn first_fitting_line(
    candidates: impl IntoIterator<Item = Line<'static>>,
    width: u16,
) -> Line<'static> {
    candidates
        .into_iter()
        .find(|line| line.width() <= usize::from(width))
        .unwrap_or_default()
}
