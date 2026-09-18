use super::App;
use super::background_requests::fetch_all_mcp_server_statuses;
use crate::airs_doctor::Event;
use crate::airs_doctor::process;
use crate::airs_doctor::views;
use crate::app_event::AppEvent;
use crate::app_server_session::AppServerSession;
use codex_app_server_protocol::McpServerStatusDetail;
use std::time::Duration;

impl App {
    pub(super) async fn handle_airs_doctor(&mut self, app_server: &AppServerSession, event: Event) {
        if !codex_utils_home_dir::is_airs_harness() {
            return;
        }
        if !app_server.uses_embedded_app_server() {
            self.chat_widget.add_error_message(
                "Run airs doctor on the host running this remote app-server session.".into(),
            );
            return;
        }
        match event {
            Event::Cancel(attempt) => {
                if self.airs_recovery.is_current(attempt) {
                    self.airs_recovery.cancel();
                    self.chat_widget.dismiss_airs_doctor();
                }
            }
            Event::ConfirmVerify => self.chat_widget.show_airs_doctor(views::confirm_verify()),
            Event::SignIn => self.chat_widget.open_airs_sign_in(),
            Event::Done {
                attempt,
                thread,
                environment,
                report,
                connections,
            } => {
                if self.airs_recovery.finish(attempt) {
                    self.chat_widget.dismiss_airs_doctor();
                    if self.current_displayed_thread_id() != Some(thread) {
                        return;
                    }
                    self.chat_widget.show_airs_doctor(views::overview(
                        &environment,
                        report,
                        connections,
                    ));
                }
            }
            Event::Open(mode) => {
                if !self.chat_widget.airs_doctor_ready() {
                    return;
                }
                let Some(thread) = self.current_displayed_thread_id() else {
                    return;
                };
                let Some((attempt, cancellation)) = self.airs_recovery.begin() else {
                    self.chat_widget.add_error_message("Another diagnostic, sign-in or MCP operation is running. Cancel it before retrying.".into());
                    return;
                };
                self.chat_widget.show_airs_doctor(views::waiting(attempt));
                let home = self.config.codex_home.clone();
                let cwd = self.config.cwd.to_path_buf();
                let environment = crate::airs_branding::environment_name(&home)
                    .unwrap_or_else(|| "Current environment".into());
                let config = match self.rebuild_config_for_cwd(cwd.clone()).await {
                    Ok(config) => config,
                    Err(_) => {
                        self.airs_recovery.finish(attempt);
                        self.chat_widget.show_airs_doctor(views::overview(
                            &environment,
                            Err("Could not load this environment. Check airs env status.".into()),
                            Vec::new(),
                        ));
                        return;
                    }
                };
                self.config.mcp_servers = config.mcp_servers.clone();
                self.chat_widget.update_airs_mcp_connections(&config);
                let request = app_server.request_handle();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let work = async {
                        let (report, statuses) = tokio::join!(
                            process::run(&home, &cwd, mode),
                            tokio::time::timeout(
                                Duration::from_secs(15),
                                fetch_all_mcp_server_statuses(
                                    request,
                                    McpServerStatusDetail::ToolsAndAuthOnly,
                                    Some(thread)
                                )
                            )
                        );
                        let statuses = statuses.ok().and_then(Result::ok).unwrap_or_default();
                        (
                            report,
                            crate::airs_mcp_manager::connections(&config, &statuses),
                        )
                    };
                    let (report, connections) = tokio::select! {
                        _ = cancellation.cancelled() => return,
                        result = tokio::time::timeout(Duration::from_secs(55), work) => result.unwrap_or_else(|_| (Err("Diagnostics timed out. Retry /doctor; your conversation and draft are preserved.".into()), Vec::new())),
                    };
                    tx.send(AppEvent::AirsDoctor(Event::Done {
                        attempt,
                        thread,
                        environment,
                        report,
                        connections,
                    }));
                });
            }
        }
    }
}
