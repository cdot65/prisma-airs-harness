//! Keys travel through a private channel, never the composer or AppEvent payload.
use super::VIEW_ID;
use crate::bottom_pane::BottomPaneView;
use crate::bottom_pane::CancellationEvent;
use crate::bottom_pane::ViewCompletion;
use crate::render::renderable::Renderable;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use tokio::sync::oneshot;

pub(crate) struct KeyView {
    input: String,
    sender: Option<oneshot::Sender<String>>,
    completion: Option<ViewCompletion>,
    notice: &'static str,
}

impl KeyView {
    pub(crate) fn new(sender: oneshot::Sender<String>) -> Self {
        Self {
            input: String::new(),
            sender: Some(sender),
            completion: None,
            notice: "",
        }
    }

    fn append(&mut self, value: &str) {
        if self.input.len().saturating_add(value.len()) > 16384
            || !value.bytes().all(|b| b.is_ascii_graphic())
        {
            self.notice = "Use one API key without whitespace (maximum 16 KiB).";
            return;
        }
        self.input.push_str(value);
        self.notice = "";
    }

    fn cancel(&mut self) {
        self.input.clear();
        self.sender.take();
        self.completion = Some(ViewCompletion::Cancelled);
    }

    fn lines(&self, width: u16) -> Vec<Line<'static>> {
        [
            "TypeSafe Jev API key",
            "Stored in this environment's native credential store. Never sent to the conversation.",
            if self.input.is_empty() {
                "Paste or type API key (hidden)"
            } else {
                "API key entered (hidden)"
            },
            self.notice,
            "Enter save · Ctrl+U clear · Esc cancel",
        ]
        .into_iter()
        .flat_map(|line| textwrap::wrap(line, usize::from(width.max(1))))
        .map(|line| Line::from(line.into_owned()))
        .collect()
    }
}

impl BottomPaneView for KeyView {
    fn handle_key_event(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        match key.code {
            KeyCode::Esc => self.cancel(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.cancel(),
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input.clear();
                self.notice = "";
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter if !self.input.is_empty() => {
                if let Some(sender) = self.sender.take() {
                    let _ = sender.send(std::mem::take(&mut self.input));
                }
                self.completion = Some(ViewCompletion::Accepted);
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

impl Renderable for KeyView {
    fn desired_height(&self, width: u16) -> u16 {
        self.lines(width).len() as u16
    }
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(self.lines(area.width)).render(area, buffer);
    }
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
