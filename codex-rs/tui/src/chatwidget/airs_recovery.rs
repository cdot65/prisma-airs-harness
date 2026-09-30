//! Explicit authentication recovery preserves the conversation and composer. It never
//! replays an inference stream or a completed tool; the user chooses what to retry.
use super::ChatWidget;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::popup_consts::standard_popup_hint_line;

impl ChatWidget {
    pub(crate) fn open_airs_sign_in(&mut self) {
        if !codex_utils_home_dir::is_airs_harness() {
            return;
        }
        self.app_event_tx.send(AppEvent::AirsSignInMenu);
    }

    pub(crate) fn show_airs_sign_in(&mut self, attempt: Option<u64>) {
        let command = match crate::airs_branding::environment_name(&self.config.codex_home) {
            Some(name) if name.starts_with('-') => format!("airs --environment={name}"),
            Some(name) => format!("airs --environment {name}"),
            None => "airs".into(),
        };
        let mut view = sign_in_view(attempt, &command);
        if let Some(thread_id) = self.thread_id() {
            for (name, server) in self.config.mcp_servers.get() {
                if super::airs_mcp_recovery::supports_oauth(server) {
                    let name = name.clone();
                    view.items.insert(view.items.len() - 1, SelectionItem {
                        name: format!("Sign in to {name} tools"),
                        description: Some("Gateway consent, followed by a new conversation. Your current conversation stays saved.".into()),
                        actions: vec![Box::new(move |tx| tx.send(AppEvent::AirsMcpSignIn { server: name.clone(), thread_id }))],
                        dismiss_on_select: true,
                        ..Default::default()
                    });
                }
            }
        }
        self.bottom_pane.show_selection_view(view);
        self.request_redraw();
    }

    pub(crate) fn airs_sign_in_completed(&mut self, result: Result<(), String>) {
        match result {
            Ok(()) => {
                self.input_queue.authentication_pending =
                    self.input_queue.mcp_authentication_pending.is_some();
                self.add_info_message("Sign-in restored for the same verified identity. Your conversation and draft are preserved. Retry the request when ready.".into(), None);
            }
            Err(message) => self.add_error_message(message),
        }
        self.request_redraw();
    }
}

pub(super) fn sign_in_view(attempt: Option<u64>, command: &str) -> SelectionViewParams {
    // A different credential ends this session's gateway access, so the change runs
    // from the terminal where its guided flow can confirm, save and test it.
    let change = format!(
        "To replace this environment's workspace API key or switch between company SSO and a key, exit AIRS and run: {command} env auth\nIt confirms the change, saves it and sends one test request. Then continue this conversation with: {command} resume"
    );
    SelectionViewParams {
        title: Some("Restore company sign-in".into()),
        subtitle: Some("Your conversation and draft stay here. Sign in as the same person.".into()),
        footer_hint: Some(standard_popup_hint_line()),
        items: vec![
            SelectionItem {
                name: "Sign in".into(),
                description: Some(
                    "Open your browser and save the verified credentials in the native store."
                        .into(),
                ),
                actions: vec![Box::new(|tx| tx.send(AppEvent::AirsSignIn))],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Change key or sign-in method".into(),
                description: Some(
                    "Use a different workspace API key, company user, or sign-in method.".into(),
                ),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::InsertHistoryCell(Box::new(
                        crate::history_cell::new_info_event(change.clone(), /*hint*/ None),
                    )))
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: "Cancel".into(),
                description: Some(
                    "Keep this conversation and draft; sign in later with /signin.".into(),
                ),
                actions: vec![Box::new(move |tx| {
                    if let Some(attempt) = attempt {
                        tx.send(AppEvent::AirsSignInCancel(attempt));
                    }
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// The public error event includes CodexErr's fatal prefix; match the local recovery
/// marker after that wrapper without interpreting arbitrary remote sign-in URLs.
pub(super) fn mcp_sign_in_message(message: &str) -> Option<&str> {
    let message = message.strip_prefix("Fatal error: ").unwrap_or(message);
    message
        .starts_with("MCP sign-in required:")
        .then_some(message)
}
