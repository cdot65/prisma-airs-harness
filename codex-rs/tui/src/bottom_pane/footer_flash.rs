//! Transient browsing feedback reuses the composer's timed footer.
use super::BottomPane;
use ratatui::text::Line;
use std::time::Duration;

impl BottomPane {
    pub(crate) fn show_footer_flash(&mut self, line: Line<'static>, duration: Duration) {
        self.composer.show_footer_flash(line, duration);
        self.request_redraw();
    }
}
