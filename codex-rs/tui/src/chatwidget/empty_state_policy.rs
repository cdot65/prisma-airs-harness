//! Gate decoration on startup-only live content and an ordinary, locally editable composer.
//! Content eligibility is latched off independently of temporary view ownership.

use super::*;
use crate::empty_state_animation::ComposerState;
use crate::empty_state_animation::is_startup_cell;

impl ChatWidget {
    pub(crate) fn empty_state_composer(&self) -> Option<ComposerState> {
        let startup_only = self
            .transcript
            .active_cell
            .as_deref()
            .is_none_or(is_startup_cell);
        if !startup_only
            || self.is_user_turn_pending_or_running()
            || self.initial_user_message.is_some()
        {
            self.empty_state_animation.borrow_mut().dismiss();
        }
        match self.external_editor_state {
            ExternalEditorState::Requested | ExternalEditorState::Active => None,
            ExternalEditorState::Closed if !self.external_writer_view => {
                if !self.bottom_pane.no_modal_or_popup_active() {
                    None
                } else if self.composer_is_empty() {
                    Some(ComposerState::Empty)
                } else {
                    Some(ComposerState::Draft)
                }
            }
            ExternalEditorState::Closed => None,
        }
    }
}
