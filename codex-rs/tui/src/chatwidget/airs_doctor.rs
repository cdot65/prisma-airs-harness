use super::ChatWidget;
use crate::airs_doctor::VIEW_ID;
use crate::bottom_pane::SelectionViewParams;

impl ChatWidget {
    pub(crate) fn show_airs_doctor(&mut self, view: SelectionViewParams) {
        self.bottom_pane.dismiss_view_by_id(VIEW_ID);
        self.bottom_pane.show_selection_view(view);
        self.request_redraw();
    }

    pub(crate) fn dismiss_airs_doctor(&mut self) {
        self.bottom_pane.dismiss_view_by_id(VIEW_ID);
        self.request_redraw();
    }

    pub(crate) fn airs_doctor_ready(&mut self) -> bool {
        if self.bottom_pane.is_task_running() || self.input_queue.user_turn_pending_start {
            self.add_info_message(
                "Finish or interrupt the current request before running /doctor.".into(),
                /*hint*/ None,
            );
            false
        } else {
            true
        }
    }
}
