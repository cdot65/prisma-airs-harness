use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;

const MAX_INPUT_BYTES: usize = 2048;

pub(super) struct Editor {
    pub value: String,
    pub cursor: usize,
    pub error: Option<&'static str>,
}

impl Editor {
    pub fn new(value: &str) -> Self {
        let value = super::safe_text(value);
        let mut result = Self {
            value: String::new(),
            cursor: 0,
            error: None,
        };
        result.insert(&value);
        result
    }

    pub fn apply(&mut self, event: Event) -> bool {
        match event {
            Event::Paste(value) => {
                if value.chars().any(char::is_control) {
                    self.error =
                        Some("Paste one line. Newlines and control characters are not accepted.");
                } else {
                    self.insert(&value);
                }
                true
            }
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                self.error = None;
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('u') => {
                            self.value.clear();
                            self.cursor = 0;
                        }
                        KeyCode::Char('a') => self.cursor = 0,
                        KeyCode::Char('e') => self.cursor = self.value.len(),
                        _ => {}
                    }
                    return true;
                }
                match key.code {
                    KeyCode::Left => self.cursor = self.previous(),
                    KeyCode::Right => self.cursor = self.next(),
                    KeyCode::Home => self.cursor = 0,
                    KeyCode::End => self.cursor = self.value.len(),
                    KeyCode::Backspace if self.cursor > 0 => {
                        let previous = self.previous();
                        self.value.replace_range(previous..self.cursor, "");
                        self.cursor = previous;
                    }
                    KeyCode::Delete if self.cursor < self.value.len() => {
                        self.value.replace_range(self.cursor..self.next(), "");
                    }
                    KeyCode::Char(c)
                        if !c.is_control() && !key.modifiers.contains(KeyModifiers::ALT) =>
                    {
                        self.insert(&c.to_string())
                    }
                    _ => {}
                }
                true
            }
            Event::Resize(_, _) => true,
            _ => false,
        }
    }

    fn insert(&mut self, value: &str) {
        if self.value.len().saturating_add(value.len()) > MAX_INPUT_BYTES {
            self.error = Some("This field is limited to 2048 bytes.");
            return;
        }
        let value = super::safe_text(value);
        self.value.insert_str(self.cursor, &value);
        self.cursor += value.len();
        self.error = None;
    }

    fn previous(&self) -> usize {
        self.value[..self.cursor]
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index)
    }

    fn next(&self) -> usize {
        self.value[self.cursor..]
            .chars()
            .next()
            .map_or(self.cursor, |c| self.cursor + c.len_utf8())
    }
}
