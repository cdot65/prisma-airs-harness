//! Transient OAuth dialog. Callback bytes never enter the composer or event log.
use super::VIEW_ID;
use crate::bottom_pane::BottomPaneView;
use crate::bottom_pane::CancellationEvent;
use crate::bottom_pane::ViewCompletion;
use crate::clipboard_copy::ClipboardLease;
use crate::clipboard_copy::CopyFormat;
use crate::render::renderable::Renderable;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use std::cell::Cell;
use tokio::sync::oneshot;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum BrowserMode {
    Desktop,
    OtherDevice,
}

impl BrowserMode {
    pub(crate) fn current() -> Self {
        Self::detect(|name| std::env::var_os(name).is_some_and(|value| !value.is_empty()))
    }

    fn detect(present: impl Fn(&str) -> bool) -> Self {
        if ["SSH_CONNECTION", "SSH_CLIENT", "SSH_TTY"]
            .into_iter()
            .any(&present)
            || (cfg!(target_os = "linux") && !present("DISPLAY") && !present("WAYLAND_DISPLAY"))
        {
            Self::OtherDevice
        } else {
            Self::Desktop
        }
    }
}

pub(crate) struct AuthorizationView {
    url: String,
    input: String,
    sender: Option<oneshot::Sender<String>>,
    completion: Option<ViewCompletion>,
    notice: String,
    clipboard: Option<ClipboardLease>,
    mode: BrowserMode,
    scroll: Cell<u16>,
}

impl AuthorizationView {
    pub(crate) fn new(url: String, sender: oneshot::Sender<String>, mode: BrowserMode) -> Self {
        Self {
            url,
            sender: Some(sender),
            input: String::new(),
            completion: None,
            notice: String::new(),
            clipboard: None,
            mode,
            scroll: Cell::new(0),
        }
    }

    pub(crate) fn open_browser(&mut self, open: impl FnOnce(&str) -> Result<(), ()>) {
        self.notice = if open(&self.url).is_ok() {
            "Browser opened. Complete SSO and consent; AIRS is waiting for the callback."
        } else {
            self.mode = BrowserMode::OtherDevice;
            "Browser could not open. Use the link below on a device with a browser."
        }
        .into();
    }

    fn append(&mut self, input: &str) {
        if self.input.len() + input.len() > 65536 || input.chars().any(char::is_control) {
            self.notice = "Callback must be one line, at most 64 KiB. Ctrl+U clears input.".into();
            return;
        }
        self.input.push_str(input);
        self.notice.clear();
        self.scroll.set(0);
    }

    fn cancel(&mut self) {
        self.input.clear();
        self.sender.take();
        self.completion = Some(ViewCompletion::Cancelled);
    }
}

impl BottomPaneView for AuthorizationView {
    fn handle_key_event(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        match key.code {
            KeyCode::Esc => self.cancel(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.cancel(),
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input.clear();
                self.notice.clear();
            }
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_browser(|url| webbrowser::open(url).map_err(|_| ()));
            }
            KeyCode::PageDown => self.scroll.set(self.scroll.get().saturating_add(6)),
            KeyCode::PageUp => self.scroll.set(self.scroll.get().saturating_sub(6)),
            KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                match crate::clipboard_copy::copy_to_clipboard(&self.url, CopyFormat::PlainText) {
                    Ok(lease) => {
                        self.clipboard = lease;
                        self.notice = "Authorization link copied.".into();
                    }
                    Err(_) => {
                        self.notice =
                            "Clipboard unavailable. Select and copy the displayed link.".into()
                    }
                }
            }
            KeyCode::Enter if !self.input.trim().is_empty() => {
                if let Some(sender) = self.sender.take() {
                    let _ = sender.send(std::mem::take(&mut self.input));
                }
                self.completion = Some(ViewCompletion::Accepted);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                self.append(&c.to_string())
            }
            _ => {}
        }
    }
    fn handle_paste(&mut self, text: String) -> bool {
        self.append(text.trim());
        true
    }
    fn on_ctrl_c(&mut self) -> CancellationEvent {
        self.cancel();
        CancellationEvent::Handled
    }
    fn is_complete(&self) -> bool {
        self.completion.is_some()
    }
    fn completion(&self) -> Option<ViewCompletion> {
        self.completion
    }
    fn view_id(&self) -> Option<&'static str> {
        Some(VIEW_ID)
    }
}

impl AuthorizationView {
    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        let width = usize::from(width.max(1));
        let mut lines = Vec::new();
        lines.extend(
            textwrap::wrap("Sign in to gateway MCP", width)
                .into_iter()
                .map(|line| Line::from(line.into_owned().bold())),
        );
        lines.extend(
            textwrap::wrap("Waiting for browser approval", width)
                .into_iter()
                .map(|line| Line::from(line.into_owned())),
        );
        lines.push(Line::default());
        let instructions = match self.mode {
            BrowserMode::Desktop => {
                "Complete sign-in in this machine's browser. AIRS continues automatically. If you use another device, paste its full callback URL below."
            }
            BrowserMode::OtherDevice => {
                "Open the link on your laptop or phone. After SSO and consent, a localhost page may fail to load. Copy its entire address, paste it below, then press Enter. AIRS continues here."
            }
        };
        lines.extend(
            textwrap::wrap(instructions, width)
                .into_iter()
                .map(|line| Line::from(line.into_owned())),
        );
        let input = if self.input.is_empty() {
            "Paste callback URL here (hidden)"
        } else {
            "Callback received (hidden) · Enter to submit"
        };
        lines.extend(
            textwrap::wrap(input, width)
                .into_iter()
                .map(|line| Line::from(line.into_owned().bold())),
        );
        lines.extend(
            textwrap::wrap(&self.notice, width)
                .into_iter()
                .map(|line| Line::from(line.into_owned())),
        );
        lines.push(Line::default());
        lines.extend(
            textwrap::wrap("Authorization link · Ctrl+Y copies the full link", width)
                .into_iter()
                .map(|line| Line::from(line.into_owned().dim())),
        );
        lines.extend(
            textwrap::wrap(&self.url, width)
                .into_iter()
                .map(|line| Line::from(line.into_owned().cyan())),
        );
        lines
    }
}

impl Renderable for AuthorizationView {
    fn desired_height(&self, width: u16) -> u16 {
        (self.lines(width).len() as u16 + 3).min(24)
    }
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        let hints = textwrap::wrap(
            "Ctrl+O open here · Ctrl+Y copy · Ctrl+U clear · PgUp/PgDn scroll · Esc cancel",
            usize::from(area.width.max(1)),
        )
        .into_iter()
        .map(|line| Line::from(line.into_owned().dim()))
        .collect::<Vec<_>>();
        let footer_height = (hints.len() as u16).min(area.height);
        let content = Rect {
            height: area.height.saturating_sub(footer_height),
            ..area
        };
        let lines = self.lines(area.width);
        let scroll = self
            .scroll
            .get()
            .min((lines.len() as u16).saturating_sub(content.height));
        self.scroll.set(scroll);
        Paragraph::new(lines)
            .scroll((scroll, 0))
            .render(content, buffer);
        Paragraph::new(hints).render(
            Rect {
                y: area.y + content.height,
                height: footer_height,
                ..area
            },
            buffer,
        );
    }
}

#[cfg(test)]
#[path = "authorization_tests.rs"]
mod tests;
