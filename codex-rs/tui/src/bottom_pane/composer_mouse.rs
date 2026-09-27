//! Composer mouse and copy entry points yield to private and modal views.
use super::*;
impl BottomPane {
    pub(crate) fn end_composer_drag(&mut self) {
        self.composer.end_mouse_drag();
    }

    pub(crate) fn copy_composer_selection(
        &mut self,
        event: &crate::tui::TuiEvent,
        copy: impl FnOnce(&str) -> Result<crate::clipboard_copy::CopyStatus, String>,
    ) -> Option<(usize, Result<crate::clipboard_copy::CopyStatus, String>)> {
        if self.has_active_view() || self.questions.as_ref().is_some_and(|q| q.expanded) {
            return None;
        }
        self.composer.copy_selection(event, copy)
    }

    pub(crate) fn prepare_composer_mouse(&mut self, event: crossterm::event::MouseEvent) -> bool {
        if self.has_active_view() || self.questions.as_ref().is_some_and(|q| q.expanded) {
            self.composer.end_mouse_drag();
            return false;
        }
        self.composer.prepare_mouse(event)
    }

    pub(crate) fn handle_composer_mouse(&mut self, event: crossterm::event::MouseEvent) -> bool {
        self.composer.handle_mouse(event)
    }
}
