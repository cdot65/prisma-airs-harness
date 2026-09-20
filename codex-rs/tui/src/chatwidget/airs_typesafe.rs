use super::ChatWidget;
use crate::airs_typesafe::VIEW_ID;
use crate::airs_typesafe::input::KeyView;
use crate::bottom_pane::SelectionViewParams;
use tokio::sync::oneshot;

impl ChatWidget {
    pub(crate) fn airs_typesafe_ready(&mut self) -> bool {
        if self.bottom_pane.is_task_running() || self.input_queue.user_turn_pending_start {
            self.add_info_message(
                "Finish or interrupt the current request before changing TypeSafe settings.".into(),
                /*hint*/ None,
            );
            false
        } else {
            true
        }
    }

    pub(crate) fn show_airs_typesafe(&mut self, view: SelectionViewParams) {
        self.bottom_pane.dismiss_view_by_id(VIEW_ID);
        self.bottom_pane.show_selection_view(view);
        self.request_redraw();
    }

    pub(crate) fn enter_airs_typesafe_key(&mut self, sender: oneshot::Sender<String>) {
        self.bottom_pane.dismiss_view_by_id(VIEW_ID);
        self.bottom_pane.show_view(Box::new(KeyView::new(sender)));
        self.request_redraw();
    }
}
