//! Run explicit MCP browser consent without sending the model a recovery task.
use super::App;
use super::background_requests::fetch_all_mcp_server_statuses;
use crate::app_event::AppEvent;
use crate::app_server_session::AppServerSession;
use crate::tui;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::McpAuthStatus;
use codex_app_server_protocol::McpServerRefreshResponse;
use codex_app_server_protocol::McpServerStatus;
use codex_app_server_protocol::McpServerStatusDetail;
use codex_app_server_protocol::RequestId;
use codex_protocol::ThreadId;
use std::process::Stdio;
use std::time::Duration;
use uuid::Uuid;

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
        let Some((attempt, cancellation)) = self.airs_recovery.begin() else {
            self.chat_widget.add_info_message(
                "Sign-in is already open. Use /signin and Cancel to stop it.".into(),
                None,
            );
            return;
        };
        if !self.chat_widget.prepare_airs_mcp_sign_in(&server) {
            self.airs_recovery.cancel();
            return;
        }
        let home = self.config.codex_home.to_path_buf();
        let fallback = format!(
            "AIRS_HARNESS_HOME={} airs mcp login --no-browser -- {}",
            shlex::try_quote(&home.to_string_lossy()).unwrap_or_default(),
            shlex::try_quote(&server).unwrap_or_default()
        );
        self.chat_widget.add_info_message(format!("Opening gateway sign-in for {server}. Complete browser consent within five minutes. Your conversation and draft stay here."), Some(fallback));
        let tx = self.app_event_tx.clone();
        let request_handle = app_server.request_handle();
        tokio::spawn(async move {
            let operation = async {
                #[cfg(target_os = "linux")]
                let executable = std::path::PathBuf::from("/proc/self/exe");
                #[cfg(not(target_os = "linux"))]
                let executable = std::env::current_exe()
                    .map_err(|_| "Cannot locate the running harness".to_string())?;
                let output = tokio::process::Command::new(executable)
                    .args(["mcp", "login", "--", &server])
                    .env("AIRS_HARNESS_HOME", &home)
                    .stdin(Stdio::null())
                    .kill_on_drop(true)
                    .output()
                    .await
                    .map_err(|_| {
                        "Could not open gateway sign-in. Use /signin to retry.".to_string()
                    })?;
                if !output.status.success() {
                    // OAuth output may contain callback codes and provider-supplied URLs.
                    // Do not put captured output in the transcript or treat it as an action.
                    return Err("Gateway sign-in was not completed. Your conversation and draft are preserved. Use /signin to retry.".into());
                }
                request_handle
                    .request_typed::<McpServerRefreshResponse>(ClientRequest::McpServerRefresh {
                        request_id: RequestId::String(format!(
                            "airs-mcp-reload-{}",
                            Uuid::new_v4()
                        )),
                        params: None,
                    })
                    .await
                    .map_err(|_| {
                        "Sign-in was saved, but MCP could not reload. Use /new to reconnect."
                            .to_string()
                    })?;
                let statuses = fetch_all_mcp_server_statuses(request_handle, McpServerStatusDetail::ToolsAndAuthOnly, /*thread_id*/ None)
                    .await.map_err(|_| "Sign-in was saved, but the gateway connection could not be checked. Use /new to reconnect.".to_string())?;
                connected_tools(&server, &statuses)
            };
            let result = tokio::select! {
                _ = cancellation.cancelled() => return,
                result = tokio::time::timeout(Duration::from_secs(330), operation) => result.unwrap_or_else(|_| Err("Gateway sign-in or reconnection timed out. Your conversation and draft are preserved. Use /signin to retry.".into())),
            };
            tx.send(AppEvent::AirsMcpSignInCompleted {
                attempt,
                server,
                thread_id,
                result,
            });
        });
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

fn connected_tools(server: &str, statuses: &[McpServerStatus]) -> Result<usize, String> {
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
