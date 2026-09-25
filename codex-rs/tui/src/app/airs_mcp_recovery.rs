//! Run explicit MCP browser consent without sending the model a recovery task.
use super::App;
use crate::app_event::AppEvent;
use crate::app_server_session::AppServerSession;
use crate::tui;
use codex_app_server_protocol::McpServerConnectionStatus;
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

/// Tool discovery is evidence of reachability, not permission to execute every tool.
pub(super) fn discovered_tools(
    server: &str,
    statuses: &[McpServerStatus],
) -> Result<usize, String> {
    let Some(status) = statuses.iter().find(|status| status.name == server) else {
        return Err(
            "The saved connection was not found during discovery. Refresh /mcp before retrying."
                .into(),
        );
    };
    match status.runtime_status {
        Some(McpServerConnectionStatus::AuthenticationRequired) => {
            Err("The saved connection requires sign-in. Select it in /mcp and choose Sign in; inference sign-in stays separate.".into())
        }
        Some(McpServerConnectionStatus::Connected) | None
            if status.server_info.is_some() && status.tools_error.is_none() => Ok(status.tools.len()),
        Some(McpServerConnectionStatus::Connected) | None if status.server_info.is_some() => {
            Err("The gateway responded, but tool discovery failed. Check this identity's MCP tool-list permission, then retry Reconnect and verify from /mcp.".into())
        }
        _ => Err("The gateway did not confirm the MCP connection. Check that it is enabled, then sign in or retry from /mcp.".into()),
    }
}

#[cfg(test)]
#[path = "airs_mcp_recovery_tests.rs"]
mod tests;
