//! Composer mouse and copy entry points yield to private and modal views.
use super::*;
impl BottomPane {
    pub(crate) fn end_composer_drag(&mut self) {
        self.composer.end_mouse_drag();
    }

    pub(crate) fn composer_selection_for_copy(&mut self, key: KeyEvent) -> Option<String> {
        if self.has_active_view() || self.questions.as_ref().is_some_and(|q| q.expanded) {
            return None;
        }
        self.composer.selection_for_copy(key)
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
