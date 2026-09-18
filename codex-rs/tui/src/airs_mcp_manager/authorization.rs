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
use tokio::sync::oneshot;

pub(crate) struct AuthorizationView {
    url: String,
    input: String,
    sender: Option<oneshot::Sender<String>>,
    completion: Option<ViewCompletion>,
    notice: String,
    clipboard: Option<ClipboardLease>,
}

impl AuthorizationView {
    pub(crate) fn new(url: String, sender: oneshot::Sender<String>) -> Self {
        Self {
            url,
            sender: Some(sender),
            input: String::new(),
            completion: None,
            notice: String::new(),
            clipboard: None,
        }
    }

    fn append(&mut self, input: &str) {
        if self.input.len() + input.len() > 65536 || input.chars().any(char::is_control) {
            self.notice = "Callback must be one line, at most 64 KiB. Ctrl+U clears input.".into();
            return;
        }
        self.input.push_str(input);
        self.notice.clear();
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
                self.notice = if webbrowser::open(&self.url).is_ok() {
                    "Browser opened. Complete company SSO."
                } else {
                    "Could not open a browser here. Copy the link to another device."
                }
                .into();
            }
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
            textwrap::wrap("Open here or in a browser on another device.", width)
                .into_iter()
                .map(|line| Line::from(line.into_owned())),
        );
        lines.push(Line::default());
        let url = textwrap::wrap(&self.url, width);
        lines.extend(
            url.iter()
                .take(4)
                .map(|line| Line::from(line.to_string().cyan())),
        );
        if url.len() > 4 {
            lines.extend(
                textwrap::wrap("Link preview · Ctrl+Y copies the full link", width)
                    .into_iter()
                    .map(|line| Line::from(line.into_owned().dim())),
            );
        }
        lines.push(Line::default());
        lines.extend(
            textwrap::wrap(
                "After consent, paste the full callback URL, even if its page cannot load.",
                width,
            )
            .into_iter()
            .map(|line| Line::from(line.into_owned())),
        );
        let input = if self.input.is_empty() {
            "Callback URL: input hidden"
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
        lines.extend(
            textwrap::wrap(
                "Ctrl+O open · Ctrl+Y copy link · Ctrl+U clear · Esc cancel",
                width,
            )
            .into_iter()
            .map(|line| Line::from(line.into_owned().dim())),
        );
        lines
    }
}

impl Renderable for AuthorizationView {
    fn desired_height(&self, width: u16) -> u16 {
        self.lines(width).len() as u16
    }
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(self.lines(area.width)).render(area, buffer);
    }
}

#[cfg(test)]
#[path = "authorization_tests.rs"]
mod tests;
