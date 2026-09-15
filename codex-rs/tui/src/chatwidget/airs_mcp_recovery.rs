//! Gateway MCP consent is explicit and bound to a configured OAuth connection.
use super::ChatWidget;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::popup_consts::standard_popup_hint_line;
use codex_config::types::McpServerAuth;
use codex_config::types::McpServerConfig;
use codex_config::types::McpServerTransportConfig;
use codex_protocol::ThreadId;

pub(super) fn supports_oauth(server: &McpServerConfig) -> bool {
    server.enabled
        && server.auth == McpServerAuth::OAuth
        && matches!(&server.transport, McpServerTransportConfig::StreamableHttp {
            url, bearer_token_env_var: None, http_headers, env_http_headers,
            http_headers_helper: None,
        } if url.starts_with("https://")
            && !http_headers.iter().flat_map(|h| h.keys()).any(|h| h.eq_ignore_ascii_case("authorization"))
            && !env_http_headers.iter().flat_map(|h| h.keys()).any(|h| h.eq_ignore_ascii_case("authorization")))
}

impl ChatWidget {
    pub(crate) fn prepare_airs_mcp_sign_in(&mut self, server: &str) -> bool {
        if self.bottom_pane.is_task_running() || self.input_queue.user_turn_pending_start {
            self.add_info_message(
                "Finish or interrupt the current request before signing in to MCP.".into(),
                None,
            );
            return false;
        }
        self.input_queue.authentication_pending = true;
        self.input_queue.mcp_authentication_pending = Some(server.to_owned());
        true
    }

    pub(crate) fn supports_airs_mcp_sign_in(&self, name: &str) -> bool {
        self.config
            .mcp_servers
            .get()
            .get(name)
            .is_some_and(supports_oauth)
    }

    pub(super) fn open_airs_mcp_recovery(&mut self, message: &str) {
        // Match a configured name, never interpret remote text as a shell command or URL.
        let server = self
            .config
            .mcp_servers
            .get()
            .iter()
            .find_map(|(name, config)| {
                (supports_oauth(config)
                    && message.starts_with(&format!("MCP sign-in required: {name}. ")))
                .then(|| name.clone())
            });
        if let Some(server) = server {
            self.input_queue.mcp_authentication_pending = Some(server.clone());
            self.add_info_message(format!("Sign in to restore {server} tools. Your conversation and draft stay here until you choose to continue."), None);
            if let Some(thread_id) = self.thread_id() {
                self.bottom_pane
                    .show_selection_view(mcp_sign_in_view(server, thread_id));
            }
        } else {
            self.add_error_message(
                "MCP sign-in is required. Use /signin to choose a configured gateway connection."
                    .into(),
            );
        }
        self.request_redraw();
    }

    pub(crate) fn airs_mcp_sign_in_completed(
        &mut self,
        server: String,
        tools: usize,
        thread_id: ThreadId,
    ) {
        self.add_info_message(
            format!("{server}: connected · {tools} tools. Nothing has been replayed."),
            None,
        );
        if !self.bottom_pane.is_task_running() && !self.has_pending_protected_request() {
            self.bottom_pane
                .show_selection_view(mcp_continue_view(server, thread_id));
        }
        self.request_redraw();
    }

    pub(crate) fn airs_mcp_draft(&mut self) -> crate::bottom_pane::ComposerDraftSnapshot {
        if let Some(draft) = self.drain_pending_messages_for_restore() {
            self.restore_composer_state(draft);
            self.refresh_pending_input_preview();
        }
        self.bottom_pane.composer_draft_snapshot()
    }
}

pub(super) fn mcp_sign_in_view(server: String, thread_id: ThreadId) -> SelectionViewParams {
    SelectionViewParams {
        title: Some(format!("Sign in to {server}")),
        subtitle: Some("Sign in in your browser, then choose a new conversation.".into()),
        footer_hint: Some(standard_popup_hint_line()),
        items: vec![
            SelectionItem {
                name: "Sign in".into(),
                description: Some(
                    "Open gateway sign-in and save credentials in the native store.".into(),
                ),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::AirsMcpSignIn {
                        server: server.clone(),
                        thread_id,
                    })
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Cancel".into(),
                description: Some(
                    "Keep your conversation and draft. Use /signin when ready.".into(),
                ),
                actions: vec![Box::new(|tx| tx.send(AppEvent::AirsSignInCancel))],
                dismiss_on_select: true,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

pub(super) fn mcp_continue_view(server: String, thread_id: ThreadId) -> SelectionViewParams {
    SelectionViewParams {
        title: Some(format!("{server} is connected")),
        subtitle: Some("Account continuity is unverified. Continue in a new conversation.".into()),
        footer_hint: Some(standard_popup_hint_line()),
        items: vec![
            SelectionItem {
                name: "Start new conversation".into(),
                description: Some(
                    "Keep your draft for review; nothing sends automatically.".into(),
                ),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::AirsMcpNewConversation { thread_id })
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Stay here".into(),
                description: Some("Keep viewing this conversation. Use /new when ready.".into()),
                dismiss_on_select: true,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
