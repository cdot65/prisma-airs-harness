//! Read local clipboard text without blocking input or replacing a changed draft.
//! Keep one outstanding read even after invalidation or timeout to bound worker growth.

use super::*;
use codex_config::types::RightClickPaste;
use crossterm::event::MouseButton;
use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

pub(super) struct PendingPaste {
    target: Option<(Option<ThreadId>, (String, usize))>,
    deadline: Instant,
    result: mpsc::Receiver<Result<String, String>>,
}

pub(super) struct PasteEnvironment {
    pub(super) platform_default: bool,
    pub(super) ssh: bool,
    pub(super) terminal_owns_paste: bool,
    pub(super) wsl_unknown_terminal: bool,
}

impl PasteEnvironment {
    pub(super) fn detect() -> Self {
        let terminal = std::env::var("TERM_PROGRAM").ok();
        Self {
            platform_default: cfg!(any(target_os = "windows", target_os = "linux")),
            ssh: crate::clipboard_copy::is_ssh_session(),
            terminal_owns_paste: terminal.as_deref() == Some("vscode"),
            wsl_unknown_terminal: crate::clipboard_copy::is_wsl_session() && terminal.is_none(),
        }
    }

    fn allows(&self, mode: RightClickPaste) -> bool {
        if self.ssh || self.terminal_owns_paste {
            return false;
        }
        match mode {
            RightClickPaste::Off => false,
            RightClickPaste::On => !cfg!(target_os = "android"),
            RightClickPaste::Auto => self.platform_default && !self.wsl_unknown_terminal,
        }
    }
}

impl App {
    fn right_click_paste_target(
        &self,
        tui: &tui::Tui,
    ) -> Option<(Option<ThreadId>, (String, usize))> {
        if !tui.is_owned_screen()
            || self.overlay.is_some()
            || self.transcript_view.has_selection_range()
            || self.transcript_view.is_search_active()
            || !self
                .right_click_paste_environment
                .allows(self.local_settings.tui.right_click_paste)
        {
            return None;
        }
        Some((
            self.current_displayed_thread_id(),
            self.chat_widget.right_click_paste_target()?,
        ))
    }

    pub(super) fn start_right_click_paste(&mut self, tui: &mut tui::Tui, mouse: MouseEvent) {
        if mouse.kind != MouseEventKind::Down(MouseButton::Right)
            || !mouse.modifiers.is_empty()
            || self.pending_right_click_paste.is_some()
        {
            return;
        }
        let Some(target) = self.right_click_paste_target(tui) else {
            return;
        };
        let deadline = Instant::now() + Duration::from_secs(2);
        let (sender, result) = mpsc::channel();
        let requester = tui.frame_requester();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send(crate::clipboard_paste::text::read(deadline));
            requester.schedule_frame();
        });
        self.pending_right_click_paste = Some(PendingPaste {
            target: Some(target),
            deadline,
            result,
        });
    }

    pub(super) fn invalidate_right_click_paste(&mut self, event: &TuiEvent) {
        let keep = match event {
            TuiEvent::Draw | TuiEvent::Resize(_) | TuiEvent::FocusGained => true,
            TuiEvent::Mouse(mouse) => {
                matches!(
                    mouse.kind,
                    MouseEventKind::Moved
                        | MouseEventKind::Up(MouseButton::Right)
                        | MouseEventKind::Down(MouseButton::Right)
                ) && mouse.modifiers.is_empty()
            }
            TuiEvent::Key(_) | TuiEvent::Paste(_) | TuiEvent::FocusLost | TuiEvent::Resume => false,
        };
        if !keep && let Some(pending) = &mut self.pending_right_click_paste {
            pending.target = None;
        }
    }

    pub(super) fn finish_right_click_paste(
        &mut self,
        tui: &mut tui::Tui,
        event: TuiEvent,
    ) -> TuiEvent {
        if !matches!(event, TuiEvent::Draw) {
            return event;
        }
        let Some(pending) = &mut self.pending_right_click_paste else {
            return event;
        };
        if Instant::now() >= pending.deadline {
            pending.target = None;
        }
        let result = match pending.result.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return event,
            Err(mpsc::TryRecvError::Disconnected) => Err("clipboard reader stopped".into()),
        };
        let Some(pending) = self.pending_right_click_paste.take() else {
            return event;
        };
        if pending.target.is_none() || pending.target != self.right_click_paste_target(tui) {
            return event;
        }
        match result {
            Ok(text) if !text.is_empty() => {
                tui.frame_requester().schedule_frame();
                TuiEvent::Paste(text)
            }
            Ok(_) => event,
            Err(error) => {
                self.chat_widget.add_error_message(error);
                event
            }
        }
    }
}

#[cfg(test)]
#[path = "right_click_paste_tests.rs"]
mod tests;
