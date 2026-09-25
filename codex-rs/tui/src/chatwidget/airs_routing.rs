use super::ChatWidget;
use crate::airs_routing::Event;
use crate::airs_routing::Kind;
use crate::airs_routing::Selection;
use crate::airs_routing::VIEW_ID;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::custom_prompt_view::CustomPromptView;

impl ChatWidget {
    pub(crate) fn airs_routing_blocked(&self) -> bool {
        self.airs_routing.pending.is_some() || self.airs_routing.uncertain
    }
    pub(crate) fn airs_routing_selection(&self) -> Option<Selection> {
        let routing = self.config.model_provider.gateway.as_ref()?;
        Some(Selection {
            saved_config: self.airs_routing.saved_config.clone(),
            model: routing.request_model(self.current_model()).ok()?,
        })
    }
    pub(crate) fn airs_routing_ready(&mut self) -> bool {
        if self.thread_id.is_none()
            || self.airs_routing_blocked()
            || self.bottom_pane.is_task_running()
            || self.input_queue.user_turn_pending_start
        {
            self.add_error_message("Routing changes require an idle, available conversation. Finish the request or resume this conversation before retrying.".into());
            return false;
        }
        self.config.model_provider.gateway.is_some()
    }
    pub(crate) fn show_airs_routing(&mut self, view: SelectionViewParams) {
        self.bottom_pane.dismiss_view_by_id(VIEW_ID);
        self.bottom_pane.show_selection_view(view);
        self.set_queue_autosend_suppressed(/*suppressed*/ true);
        self.request_redraw();
    }
    pub(crate) fn dismiss_airs_routing(&mut self) {
        self.bottom_pane.dismiss_view_by_id(VIEW_ID);
        self.app_event_tx.send(AppEvent::SettingsSelectionSettled);
        self.request_redraw();
    }
    pub(crate) fn prompt_airs_routing(&mut self, kind: Kind) {
        let (title, placeholder) = match kind {
            Kind::Config => ("Saved AI Gateway config ID", "pc-example-123456"),
            Kind::Model => ("Requested model override", "@integration/model"),
        };
        let tx = self.app_event_tx.clone();
        self.bottom_pane.show_text_prompt(CustomPromptView::new(
            title.into(),
            placeholder.into(),
            String::new(),
            Some("Conversation only. The gateway must accept the proposed routing.".into()),
            Box::new(move |value| {
                tx.send(AppEvent::AirsRouting(Event::Propose(
                    kind,
                    Some(value.trim().to_owned()),
                )));
            }),
        ));
        self.set_queue_autosend_suppressed(/*suppressed*/ true);
        self.request_redraw();
    }
    pub(crate) fn start_airs_routing_commit(
        &mut self,
        attempt: u64,
        proposal: crate::airs_routing::Proposal,
    ) {
        self.airs_routing.pending = Some((attempt, proposal));
        self.show_airs_routing(crate::airs_routing::views::applying());
    }
    pub(crate) fn airs_routing_pending(&self, attempt: u64) -> bool {
        self.airs_routing
            .pending
            .as_ref()
            .is_some_and(|(id, _)| *id == attempt)
    }
    pub(crate) fn finish_airs_routing(&mut self, attempt: u64) {
        if self.airs_routing_pending(attempt) {
            self.airs_routing.pending = None;
            self.airs_routing.uncertain = false;
            self.dismiss_airs_routing();
            self.add_info_message("Gateway routing applied to this conversation. The gateway controls the effective model.".into(), /*hint*/ None);
        }
    }
    pub(crate) fn fail_airs_routing_commit(&mut self, attempt: u64) {
        if self.airs_routing_pending(attempt) {
            // The request may have reached the server: never assert rollback or send queued input.
            self.airs_routing.uncertain = true;
            self.dismiss_airs_routing();
            self.pause_unavailable_thread();
            self.add_error_message("Routing confirmation did not arrive. Input is paused because the server may have applied the change. Resume this conversation to refresh its routing, or use /new. Your draft remains here.".into());
        }
    }
    pub(super) fn airs_routing_settings_applied(&mut self) {
        if let Some((attempt, proposal)) = &self.airs_routing.pending
            && self.thread_id == Some(proposal.thread)
            && self.airs_routing_selection().as_ref() == Some(&proposal.target)
        {
            self.app_event_tx
                .send(AppEvent::AirsRouting(Event::Applied(*attempt)));
        }
    }
}
