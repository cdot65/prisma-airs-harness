//! Probe before mutation, then wait for the authoritative settings notification.
use super::App;
use super::airs_recovery::RecoveryOperation;
use crate::airs_routing::Event;
use crate::airs_routing::Kind;
use crate::airs_routing::Proposal;
use crate::airs_routing::views;
use crate::app_event::AppEvent;
use crate::app_server_session::AppServerSession;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::ThreadSettingsUpdateParams;
use codex_app_server_protocol::ThreadSettingsUpdateResponse;
use std::time::Duration;

impl App {
    pub(super) fn handle_airs_routing(&mut self, app_server: &mut AppServerSession, event: Event) {
        if !codex_utils_home_dir::is_airs_harness() {
            return;
        }
        let Some(routing) = self.config.model_provider.gateway.clone() else {
            return;
        };
        if !app_server.uses_embedded_app_server() {
            self.chat_widget.add_error_message(
                "Gateway routing verification requires a local AIRS session.".into(),
            );
            return;
        }
        match event {
            Event::Open(kind) => {
                if self.chat_widget.airs_routing_ready()
                    && let Some(selection) = self.chat_widget.airs_routing_selection()
                {
                    let models = self
                        .model_catalog
                        .try_list_models()
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|preset| {
                            preset.model.starts_with('@') && preset.model.len() <= 2048
                        })
                        .filter(|preset| {
                            self.config
                                .model_provider
                                .gateway
                                .as_ref()
                                .is_some_and(|routing| routing.request_model(&preset.model).is_ok())
                        })
                        .take(32)
                        .map(|preset| preset.model)
                        .collect::<Vec<_>>();
                    self.chat_widget
                        .show_airs_routing(views::menu(kind, &selection, &models));
                }
            }
            Event::Input(kind) => {
                if self.chat_widget.airs_routing_ready() {
                    self.chat_widget.prompt_airs_routing(kind);
                }
            }
            Event::Propose(kind, value) => {
                if !self.chat_widget.airs_routing_ready() {
                    return;
                }
                let Some(thread) = self.current_displayed_thread_id() else {
                    return;
                };
                let Some(baseline) = self.chat_widget.airs_routing_selection() else {
                    return;
                };
                let mut target = baseline.changed(kind, value);
                let validation = target
                    .saved_config
                    .as_deref()
                    .map(codex_model_provider_info::GatewayRouting::validate_saved_config)
                    .transpose()
                    .and_then(|_| match target.model.as_deref() {
                        Some(model) if model.len() <= 2048 => {
                            routing.request_model(model).map(|_| ())
                        }
                        Some(_) => Err("Model route is too long.".into()),
                        None => Ok(()),
                    });
                if let Err(message) = validation {
                    self.chat_widget.add_error_message(message);
                    return;
                }
                target.model = target
                    .model
                    .as_deref()
                    .and_then(|model| routing.request_model(model).ok().flatten());
                self.chat_widget.show_airs_routing(views::confirm(Proposal {
                    thread,
                    baseline,
                    target,
                    kind,
                }));
            }
            Event::Verify(proposal) => {
                if !self.chat_widget.airs_routing_ready()
                    || self.current_displayed_thread_id() != Some(proposal.thread)
                    || self.chat_widget.airs_routing_selection().as_ref()
                        != Some(&proposal.baseline)
                {
                    return;
                }
                let Some((attempt, cancellation)) =
                    self.airs_recovery.begin(RecoveryOperation::Routing)
                else {
                    self.chat_widget.add_error_message("Finish or cancel the current sign-in, MCP or diagnostic operation before verifying routing.".into());
                    return;
                };
                self.chat_widget.show_airs_routing(views::waiting(attempt));
                let home = self.config.codex_home.clone();
                let cwd = self.config.cwd.to_path_buf();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = tokio::select! {
                        _ = cancellation.cancelled() => return,
                        result = tokio::time::timeout(Duration::from_secs(/*secs*/ 55), crate::airs_doctor::process::routing(&home, &cwd, &proposal.target)) => result.unwrap_or_else(|_| Err("Routing verification timed out; current routing is unchanged.".into())),
                    }.and_then(|report| match report.checks.into_iter().find(|check| check.name == "gateway_access") {
                        Some(check) if check.passed => Ok(()),
                        Some(check) => Err(crate::airs_doctor::display(&check.detail)),
                        None => Err("Gateway verification returned no access result.".into()),
                    });
                    tx.send(AppEvent::AirsRouting(Event::Checked {
                        attempt,
                        proposal,
                        result,
                    }));
                });
            }
            Event::Cancel(attempt) => {
                if !self.chat_widget.airs_routing_pending(attempt)
                    && self.airs_recovery.cancel_attempt(attempt)
                {
                    self.chat_widget.dismiss_airs_routing();
                }
            }
            Event::Checked {
                attempt,
                proposal,
                result,
            } => {
                if !self.airs_recovery.is_current(attempt) {
                    return;
                }
                if self.current_displayed_thread_id() != Some(proposal.thread)
                    || self.chat_widget.airs_routing_selection().as_ref()
                        != Some(&proposal.baseline)
                {
                    self.airs_recovery.finish(attempt);
                    self.chat_widget.dismiss_airs_routing();
                    return;
                }
                if let Err(message) = result {
                    self.airs_recovery.finish(attempt);
                    self.chat_widget.dismiss_airs_routing();
                    self.chat_widget
                        .add_error_message(format!("Routing unchanged. {message}"));
                    return;
                }
                if proposal.baseline == proposal.target {
                    self.airs_recovery.finish(attempt);
                    self.chat_widget.dismiss_airs_routing();
                    self.chat_widget.add_info_message("Current routing accepted by the gateway connectivity check. No settings changed.".into(), /*hint*/ None);
                    return;
                }
                let mut params = ThreadSettingsUpdateParams {
                    thread_id: proposal.thread.to_string(),
                    ..Default::default()
                };
                match proposal.kind {
                    Kind::Config => {
                        params.gateway_config = Some(proposal.target.saved_config.clone())
                    }
                    Kind::Model => {
                        params.model = Some(
                            proposal
                                .target
                                .model
                                .clone()
                                .unwrap_or_else(|| routing.default_route.clone()),
                        )
                    }
                }
                self.chat_widget
                    .start_airs_routing_commit(attempt, proposal);
                let request = app_server.request_handle();
                let request_id = app_server.next_request_id();
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    let result = tokio::time::timeout(
                        Duration::from_secs(/*secs*/ 15),
                        request.request_typed::<ThreadSettingsUpdateResponse>(
                            ClientRequest::ThreadSettingsUpdate { request_id, params },
                        ),
                    )
                    .await;
                    if !matches!(result, Ok(Ok(_))) {
                        tx.send(AppEvent::AirsRouting(Event::CommitFailed(attempt)));
                    }
                });
                let tx = self.app_event_tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(/*secs*/ 20)).await;
                    tx.send(AppEvent::AirsRouting(Event::Deadline(attempt)));
                });
            }
            Event::Applied(attempt) => {
                self.airs_recovery.finish(attempt);
                self.chat_widget.finish_airs_routing(attempt);
            }
            Event::CommitFailed(attempt) | Event::Deadline(attempt) => {
                self.airs_recovery.finish(attempt);
                self.chat_widget.fail_airs_routing_commit(attempt);
            }
        }
    }
}
