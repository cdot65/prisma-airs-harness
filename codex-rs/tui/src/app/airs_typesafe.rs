use super::App;
use super::airs_recovery::RecoveryOperation;
use crate::airs_typesafe::Event;
use crate::airs_typesafe::Operation;
use crate::airs_typesafe::process;
use crate::airs_typesafe::views;
use crate::app_event::AppEvent;
use crate::app_server_session::AppServerSession;
use tokio::sync::oneshot;

impl App {
    pub(super) fn handle_airs_typesafe(&mut self, app_server: &AppServerSession, event: Event) {
        if !codex_utils_home_dir::is_airs_harness() {
            return;
        }
        if !app_server.uses_embedded_app_server() {
            self.chat_widget.add_error_message(
                "TypeSafe settings must be configured on the host running this remote session."
                    .into(),
            );
            return;
        }
        if let Event::Done {
            attempt,
            thread,
            home,
            result,
        } = event
        {
            if self.airs_recovery.finish(attempt)
                && self.current_displayed_thread_id() == thread
                && self.config.codex_home.as_path() == home
            {
                let environment = crate::airs_branding::environment_name(&home)
                    .unwrap_or_else(|| "Current environment".into());
                let status = result.unwrap_or_else(|error| error);
                self.chat_widget
                    .show_airs_typesafe(views::overview(&environment, &status));
            }
            return;
        }
        if !self.chat_widget.airs_typesafe_ready() {
            return;
        }
        if matches!(event, Event::ConfirmClear) {
            self.chat_widget.show_airs_typesafe(views::confirm_clear());
            return;
        }
        let Some((attempt, cancellation)) = self.airs_recovery.begin(RecoveryOperation::TypeSafe)
        else {
            self.chat_widget.add_error_message("Another diagnostic, sign-in or settings operation is running. Finish it before retrying.".into());
            return;
        };
        let operation = match event {
            Event::Set => Operation::Set,
            Event::Clear => Operation::Clear,
            _ => Operation::Status,
        };
        let receiver = if operation == Operation::Set {
            let (sender, receiver) = oneshot::channel();
            self.chat_widget.enter_airs_typesafe_key(sender);
            Some(receiver)
        } else {
            None
        };
        let home = self.config.codex_home.to_path_buf();
        let thread = self.current_displayed_thread_id();
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let work = async {
                let key = match receiver {
                    Some(receiver) => Some(
                        tokio::time::timeout(std::time::Duration::from_secs(600), receiver)
                            .await
                            .map_err(|_| {
                                "Key entry expired. Existing credentials are unchanged.".to_owned()
                            })?
                            .map_err(|_| {
                                "Cancelled. Existing credentials are unchanged.".to_owned()
                            })?,
                    ),
                    None => None,
                };
                process::run(&home, operation, key).await
            };
            let result =
                tokio::select! { _ = cancellation.cancelled() => return, result = work => result };
            tx.send(AppEvent::AirsTypeSafe(Event::Done {
                attempt,
                thread,
                home,
                result,
            }));
        });
    }
}
