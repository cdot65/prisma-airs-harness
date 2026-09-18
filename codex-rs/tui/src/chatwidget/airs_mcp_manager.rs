//! Small chat-widget hooks for the isolated AIRS MCP manager.
use super::ChatWidget;
use crate::airs_mcp_manager::AuthorizationView;
use crate::airs_mcp_manager::Event;
use crate::airs_mcp_manager::Operation;
use crate::airs_mcp_manager::VIEW_ID;
use crate::airs_mcp_manager::process::validate_new_connection;
use crate::airs_mcp_manager::views;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::custom_prompt_view::CustomPromptView;
use crate::legacy_core::config::Config;
use tokio::sync::oneshot;

impl ChatWidget {
    pub(crate) fn airs_mcp_manager_ready(&mut self) -> bool {
        if self.bottom_pane.is_task_running() || self.input_queue.user_turn_pending_start {
            self.add_info_message(
                "Finish or interrupt the current request before managing MCP connections.".into(),
                /*hint*/ None,
            );
            false
        } else {
            true
        }
    }

    pub(crate) fn show_airs_mcp_menu(&mut self, view: SelectionViewParams) {
        self.bottom_pane.dismiss_view_by_id(VIEW_ID);
        self.bottom_pane.show_selection_view(view);
        self.request_redraw();
    }

    pub(crate) fn dismiss_airs_mcp_manager(&mut self) {
        self.bottom_pane.dismiss_view_by_id(VIEW_ID);
        self.request_redraw();
    }

    pub(crate) fn show_airs_mcp_authorization(
        &mut self,
        url: String,
        callback: oneshot::Sender<String>,
    ) {
        self.bottom_pane.dismiss_view_by_id(VIEW_ID);
        self.bottom_pane
            .show_view(Box::new(AuthorizationView::new(url, callback)));
        self.request_redraw();
    }

    pub(crate) fn update_airs_mcp_connections(&mut self, config: &Config) {
        self.config.mcp_servers = config.mcp_servers.clone();
    }

    pub(crate) fn prompt_airs_mcp_connection(&mut self, name: Option<String>) {
        let tx = self.app_event_tx.clone();
        let (title, placeholder) = if name.is_some() {
            (
                "AI Gateway MCP URL",
                "https://gateway-mcp.example.com/service-now/mcp",
            )
        } else {
            ("Name this MCP connection", "service-now")
        };
        let view = CustomPromptView::new(
            title.into(),
            placeholder.into(),
            String::new(),
            Some("Use your administrator's AI Gateway endpoint. Esc cancels.".into()),
            Box::new(move |value| {
                let event = match &name {
                    Some(name) => Event::Run(Operation::Add {
                        name: name.clone(),
                        url: value.trim().into(),
                    }),
                    None => Event::AddUrl(value.trim().into()),
                };
                tx.send(AppEvent::AirsMcpManager(event));
            }),
        );
        self.bottom_pane.show_text_prompt(view);
        self.request_redraw();
    }

    pub(crate) fn check_airs_mcp_name(&mut self, name: &str) -> bool {
        let result = validate_new_connection(name, "https://gateway.example.com/mcp");
        if let Err(message) = result {
            self.add_error_message(message);
            return false;
        }
        if self.config.mcp_servers.get().contains_key(name) {
            self.add_error_message(
                "That MCP name already exists. Choose it from /mcp or add a different name.".into(),
            );
            return false;
        }
        true
    }

    pub(crate) fn airs_mcp_manager_changed(
        &mut self,
        message: String,
        thread: codex_protocol::ThreadId,
    ) {
        self.add_info_message(message, /*hint*/ None);
        self.show_airs_mcp_menu(views::continue_after_change(thread));
    }
}
