//! Own the explicit browser attempt without blocking terminal input or losing drafts.
use super::App;
use crate::app_event::AppEvent;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RecoveryOperation {
    CompanySignIn,
    Doctor,
    Mcp,
    TypeSafe,
}

struct RecoveryAttempt {
    operation: RecoveryOperation,
    cancellation: CancellationToken,
}

#[derive(Default)]
pub(super) struct AirsRecoveryState {
    attempt: u64,
    active: Option<RecoveryAttempt>,
}

impl AirsRecoveryState {
    pub(super) fn begin(
        &mut self,
        operation: RecoveryOperation,
    ) -> Option<(u64, CancellationToken)> {
        if self.active.is_some() {
            return None;
        }
        self.attempt = self.attempt.wrapping_add(1);
        let cancellation = CancellationToken::new();
        self.active = Some(RecoveryAttempt {
            operation,
            cancellation: cancellation.clone(),
        });
        Some((self.attempt, cancellation))
    }

    pub(super) fn cancel(&mut self) {
        if let Some(active) = self.active.take() {
            active.cancellation.cancel();
        }
        self.attempt = self.attempt.wrapping_add(1);
    }

    pub(super) fn company_sign_in_attempt(&self) -> Option<u64> {
        self.active
            .as_ref()
            .filter(|active| active.operation == RecoveryOperation::CompanySignIn)
            .map(|_| self.attempt)
    }

    pub(super) fn cancel_attempt(&mut self, attempt: u64) -> bool {
        if !self.is_current(attempt) {
            return false;
        }
        self.cancel();
        true
    }

    pub(super) fn is_current(&self, attempt: u64) -> bool {
        self.attempt == attempt && self.active.is_some()
    }

    pub(super) fn finish(&mut self, attempt: u64) -> bool {
        if attempt != self.attempt {
            return false;
        }
        self.active.take().is_some()
    }
}

impl Drop for AirsRecoveryState {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
#[path = "airs_recovery_tests.rs"]
mod tests;

impl App {
    pub(super) fn start_airs_sign_in(&mut self) {
        if !codex_utils_home_dir::is_airs_harness() {
            return;
        }
        let Some((attempt, cancellation)) =
            self.airs_recovery.begin(RecoveryOperation::CompanySignIn)
        else {
            self.chat_widget.add_info_message(
                "Sign-in is already open in your browser. Use /signin and Cancel to stop it."
                    .into(),
                None,
            );
            return;
        };
        let home = self.config.codex_home.to_path_buf();
        let fallback = format!(
            "AIRS_HARNESS_HOME={} airs login --restore-session --no-browser",
            shlex::try_quote(&home.to_string_lossy()).unwrap_or_default()
        );
        self.chat_widget.add_info_message("Opening company sign-in. Complete the browser flow and any native-store prompt within five minutes. Your draft remains here.".into(), Some(fallback));
        let tx = self.app_event_tx.clone();
        tokio::spawn(async move {
            let operation = async {
                #[cfg(target_os = "linux")]
                let executable = std::path::PathBuf::from("/proc/self/exe");
                #[cfg(not(target_os = "linux"))]
                let executable = std::env::current_exe()
                    .map_err(|_| "Cannot locate the running harness".to_string())?;
                let output = tokio::process::Command::new(executable)
                    .args(["login", "--restore-session"])
                    .env("AIRS_HARNESS_HOME", &home)
                    .stdin(std::process::Stdio::null())
                    .kill_on_drop(true)
                    .output()
                    .await
                    .map_err(|_| {
                        "Could not start sign-in; run the displayed command in another terminal"
                            .to_string()
                    })?;
                if output.status.success() {
                    Ok(())
                } else {
                    // The child is the same native executable, with bounded, sanitized identity errors.
                    let message = String::from_utf8_lossy(&output.stderr);
                    Err(format!(
                        "Sign-in was not completed. {}",
                        message.chars().take(4096).collect::<String>()
                    ))
                }
            };
            let result = tokio::select! {
                _ = cancellation.cancelled() => return,
                result = tokio::time::timeout(std::time::Duration::from_secs(330), operation) => result.unwrap_or_else(|_| Err("Sign-in expired. Use /signin when ready; your conversation is preserved.".into())),
            };
            tx.send(AppEvent::AirsSignInCompleted { attempt, result });
        });
    }
}
