//! AIRS-only orchestration, using existing CLI mutation/OAuth and app-server MCP APIs.
use super::App;
use super::background_requests::fetch_all_mcp_server_statuses;
use crate::airs_mcp_manager::Event;
use crate::airs_mcp_manager::Operation;
use crate::airs_mcp_manager::process;
use crate::airs_mcp_manager::views;
use crate::app_event::AppEvent;
use crate::app_server_session::AppServerSession;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::McpServerRefreshResponse;
use codex_app_server_protocol::McpServerStatusDetail;
use codex_app_server_protocol::RequestId;
use std::time::Duration;

impl App {
    pub(super) async fn handle_airs_mcp_manager(
        &mut self,
        app_server: &AppServerSession,
        event: Event,
    ) {
        if !codex_utils_home_dir::is_airs_harness() {
            return;
        }
        if !app_server.uses_embedded_app_server() {
            self.chat_widget.add_error_message(
                "Manage MCP on the host running this remote app-server session.".into(),
            );
            return;
        }
        match event {
            Event::Cancel => {
                self.airs_recovery.cancel();
                self.chat_widget.dismiss_airs_mcp_manager();
                self.chat_widget.add_info_message("MCP operation cancelled. Your conversation and draft are preserved. Use /mcp to retry or /new to continue.".into(), /*hint*/ None);
            }
            Event::Authorize {
                attempt,
                thread,
                url,
                callback,
            } => {
                if self.airs_recovery.is_current(attempt)
                    && self.current_displayed_thread_id() == Some(thread)
                {
                    self.chat_widget.show_airs_mcp_authorization(url, callback);
                }
            }
            Event::Overview {
                attempt,
                thread,
                connections,
            } => {
                if self.airs_recovery.finish(attempt)
                    && self.current_displayed_thread_id() == Some(thread)
                {
                    self.chat_widget
                        .show_airs_mcp_menu(views::overview(connections));
                }
            }
            Event::Done {
                attempt,
                thread,
                operation,
                result,
            } => {
                if !self.airs_recovery.finish(attempt)
                    || self.current_displayed_thread_id() != Some(thread)
                {
                    return;
                }
                self.chat_widget.dismiss_airs_mcp_manager();
                // Add can save configuration before consent is cancelled. Always reread it.
                if let Ok(config) = self
                    .rebuild_config_for_cwd(self.config.cwd.to_path_buf())
                    .await
                {
                    self.config.mcp_servers = config.mcp_servers.clone();
                    self.chat_widget.update_airs_mcp_connections(&config);
                }
                let result = result.and_then(|statuses| match &operation {
                    Operation::Logout(name) => Ok(format!(
                        "{name}: signed out. Inference sign-in is unchanged."
                    )),
                    Operation::Remove(name) if self.config.mcp_servers.get().contains_key(name) => {
                        Err("This connection is still provided by another configuration layer. Update that configuration to remove it.".into())
                    },
                    Operation::Remove(name) => Ok(format!(
                        "{name}: removed from this environment. Other connections are unchanged."
                    )),
                    Operation::Verify(name) => {
                        statuses.iter().find(|status| {
                            status.name == *name && status.server_info.is_some() && status.tools_error.is_none()
                        }).map(|status| format!("{name}: connected · {} tools. Nothing has been replayed.", status.tools.len()))
                        .ok_or_else(|| "The gateway did not confirm this connection. Check that it is enabled, then sign in or retry from /mcp.".into())
                    },
                    Operation::Add { name, .. }
                    | Operation::Login(name) => {
                        super::airs_mcp_recovery::connected_tools(name, &statuses).map(|count| {
                            format!("{name}: connected · {count} tools. Nothing has been replayed.")
                        })
                    }
                });
                match result {
                    Ok(message) => self.chat_widget.airs_mcp_manager_changed(message, thread),
                    Err(message) => self.chat_widget.add_error_message(format!("{message} Your conversation and draft remain. Use /mcp to retry or /new to continue.")),
                }
            }
            Event::Select(connection) => self
                .chat_widget
                .show_airs_mcp_menu(views::connection(connection)),
            Event::Confirm(operation) => self
                .chat_widget
                .show_airs_mcp_menu(views::confirm(operation)),
            Event::AddName => self.chat_widget.prompt_airs_mcp_connection(/*name*/ None),
            Event::AddUrl(name) => {
                if self.chat_widget.check_airs_mcp_name(&name) {
                    self.chat_widget.prompt_airs_mcp_connection(Some(name));
                }
            }
            Event::Open => {
                if !self.chat_widget.airs_mcp_manager_ready() {
                    return;
                }
                let Some(thread) = self.current_displayed_thread_id() else {
                    return;
                };
                let Some((attempt, cancellation)) = self.airs_recovery.begin() else {
                    self.chat_widget.add_info_message("An authentication or MCP operation is already running. Cancel it before opening another.".into(), /*hint*/ None);
                    return;
                };
                let config = match self
                    .rebuild_config_for_cwd(self.config.cwd.to_path_buf())
                    .await
                {
                    Ok(config) => config,
                    Err(_) => {
                        self.airs_recovery.finish(attempt);
                        self.chat_widget.add_error_message(
                            "Could not load this environment's MCP configuration.".into(),
                        );
                        return;
                    }
                };
                self.config.mcp_servers = config.mcp_servers.clone();
                self.chat_widget.update_airs_mcp_connections(&config);
                self.chat_widget.show_airs_mcp_menu(views::waiting());
                let request = app_server.request_handle();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let statuses = tokio::select! {
                        _ = cancellation.cancelled() => return,
                        result = tokio::time::timeout(Duration::from_secs(15), fetch_all_mcp_server_statuses(request, McpServerStatusDetail::ToolsAndAuthOnly, /*thread_id*/ None)) => result.ok().and_then(Result::ok).unwrap_or_default(),
                    };
                    let connections = crate::airs_mcp_manager::connections(&config, &statuses);
                    tx.send(AppEvent::AirsMcpManager(Event::Overview {
                        attempt,
                        thread,
                        connections,
                    }));
                });
            }
            Event::Run(operation) => {
                if let Operation::Add { name, url } = &operation {
                    if let Err(message) = process::validate_new_connection(name, url) {
                        self.chat_widget.add_error_message(message);
                        return;
                    }
                    if !self.chat_widget.check_airs_mcp_name(name) {
                        return;
                    }
                }
                let Some(thread) = self.current_displayed_thread_id() else {
                    return;
                };
                if !self.chat_widget.airs_mcp_manager_ready() {
                    return;
                }
                let Some((attempt, cancellation)) = self.airs_recovery.begin() else {
                    self.chat_widget.add_error_message(
                        "Another sign-in or MCP operation is already running.".into(),
                    );
                    return;
                };
                if !self.chat_widget.prepare_airs_mcp_sign_in(operation.name()) {
                    self.airs_recovery.cancel();
                    return;
                }
                self.chat_widget.show_airs_mcp_menu(views::waiting());
                let home = self.config.codex_home.to_path_buf();
                let cwd = self.config.cwd.to_path_buf();
                let tx = self.app_event_tx.clone();
                let request = app_server.request_handle();
                tokio::spawn(async move {
                    let work = async {
                        process::run(&home, &cwd, &operation, attempt, thread, &tx).await?;
                        request.request_typed::<McpServerRefreshResponse>(ClientRequest::McpServerRefresh { request_id: RequestId::String(format!("airs-mcp-manager-{attempt}")), params: None }).await.map_err(|_| "Connection saved, but MCP reload failed. Start a new conversation and retry /mcp.".to_string())?;
                        fetch_all_mcp_server_statuses(request, McpServerStatusDetail::ToolsAndAuthOnly, /*thread_id*/ None).await.map_err(|_| "Connection saved, but tool discovery could not be verified. Retry from /mcp.".to_string())
                    };
                    let result = tokio::select! {
                        _ = cancellation.cancelled() => return,
                        result = tokio::time::timeout(Duration::from_secs(330), work) => result.unwrap_or_else(|_| Err("MCP operation timed out. Your saved connection and draft remain; retry from /mcp.".into())),
                    };
                    tx.send(AppEvent::AirsMcpManager(Event::Done {
                        attempt,
                        thread,
                        operation,
                        result,
                    }));
                });
            }
        }
    }
}
