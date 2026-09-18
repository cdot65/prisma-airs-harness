//! Run explicit MCP browser consent without sending the model a recovery task.
use super::App;
use crate::app_event::AppEvent;
use crate::app_server_session::AppServerSession;
use crate::tui;
use codex_app_server_protocol::McpAuthStatus;
use codex_app_server_protocol::McpServerStatus;
use codex_protocol::ThreadId;

impl App {
    pub(super) fn start_airs_mcp_sign_in(
        &mut self,
        app_server: &AppServerSession,
        server: String,
        thread_id: ThreadId,
    ) {
        if !codex_utils_home_dir::is_airs_harness()
            || self.current_displayed_thread_id() != Some(thread_id)
            || !self.chat_widget.supports_airs_mcp_sign_in(&server)
        {
            return;
        }
        if !app_server.uses_embedded_app_server() {
            self.chat_widget.add_error_message(
                "Use MCP sign-in on the host running this remote session.".into(),
            );
            return;
        }
        self.app_event_tx.send(AppEvent::AirsMcpManager(
            crate::airs_mcp_manager::Event::Run(crate::airs_mcp_manager::Operation::Login(server)),
        ));
    }

    pub(super) async fn start_airs_mcp_conversation(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        thread_id: ThreadId,
    ) {
        if !codex_utils_home_dir::is_airs_harness()
            || self.current_displayed_thread_id() != Some(thread_id)
        {
            return;
        }
        let draft = self.chat_widget.airs_mcp_draft();
        self.start_fresh_session_with_summary_hint(
            tui, app_server, /*session_start_source*/ None,
            /*initial_user_message*/ None, /*new_thread_name*/ None,
        )
        .await;
        if self
            .current_displayed_thread_id()
            .is_some_and(|current| current != thread_id)
        {
            self.chat_widget.restore_startup_draft(draft);
            self.chat_widget.add_info_message("Your previous conversation is saved. Review your draft and send it when ready; no completed tools were replayed.".into(), None);
        }
    }
}

pub(super) fn connected_tools(server: &str, statuses: &[McpServerStatus]) -> Result<usize, String> {
    let status = statuses.iter().find(|status| status.name == server);
    match status {
        Some(status) if status.auth_status == McpAuthStatus::OAuth
            && status.server_info.is_some() && status.tools_error.is_none() => Ok(status.tools.len()),
        _ => Err("Sign-in was saved, but the gateway did not confirm the MCP connection. Use /signin to retry or /new to reconnect.".into()),
    }
}

#[cfg(test)]
#[path = "airs_mcp_recovery_tests.rs"]
mod tests;
