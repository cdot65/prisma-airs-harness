//! Standalone AIRS onboarding presentation. Authentication remains with the CLI.

mod input;
mod render;
mod terminal;

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use input::Editor;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::future::Future;
use std::io;
use std::io::IsTerminal;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::watch;

const FRAME_INTERVAL: Duration = Duration::from_millis(/*millis*/ 100);

/// Public environment context; never pass credentials into the presentation layer.
#[derive(Clone, Debug, Default)]
pub struct OnboardingContext {
    pub environment: String,
    pub gateway: Option<String>,
}

/// Explicit display preferences, so fixtures need not mutate process configuration.
#[derive(Clone, Copy, Debug)]
pub struct OnboardingOptions {
    pub animations: bool,
    pub color: bool,
}

/// One keyboard-selectable action and its short explanation.
#[derive(Clone, Debug)]
pub struct OnboardingMenuItem {
    pub label: String,
    pub detail: String,
}

/// A public, single-line connection field. Secret entry uses the CLI's secure prompt.
#[derive(Clone, Debug)]
pub struct OnboardingInput {
    pub title: String,
    pub label: String,
    pub help: String,
    pub value: String,
    pub error: Option<String>,
}

/// Truthful progress supplied by the operation owner, with an optional public URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnboardingProgress {
    pub title: String,
    pub detail: String,
    pub link: Option<String>,
}

/// Cancellation is a user action, distinct from an I/O or authentication failure.
#[derive(Debug, PartialEq, Eq)]
pub enum OnboardingResult<T> {
    Selected(T),
    Cancelled,
}

/// Owns the temporary terminal session used before the ordinary agent UI starts.
pub struct AirsOnboarding {
    terminal: Terminal<CrosstermBackend<io::Stderr>>,
    context: OnboardingContext,
    options: OnboardingOptions,
    started: Instant,
    // Drop after the backend so raw mode and the alternate screen are restored.
    guard: terminal::TerminalGuard,
}

enum View<'a> {
    Menu {
        title: &'a str,
        items: &'a [OnboardingMenuItem],
        selected: usize,
    },
    Input {
        field: &'a OnboardingInput,
        editor: &'a Editor,
    },
    Progress(&'a OnboardingProgress),
}

impl AirsOnboarding {
    /// Check capability without entering raw mode or writing escape sequences.
    pub fn supported() -> bool {
        io::stdin().is_terminal()
            && io::stderr().is_terminal()
            && std::env::var("TERM").is_ok_and(|term| term != "dumb")
    }

    /// Open a branded terminal session. Dropping it restores the previous terminal.
    pub fn open(context: OnboardingContext, options: OnboardingOptions) -> io::Result<Self> {
        let guard = terminal::TerminalGuard::open()?;
        let terminal = Terminal::new(CrosstermBackend::new(io::stderr()))?;
        Ok(Self {
            terminal,
            context,
            options,
            started: Instant::now(),
            guard,
        })
    }

    /// Update public context after the caller has resolved an environment.
    pub fn set_context(&mut self, context: OnboardingContext) {
        self.context = context;
    }

    /// Select an action without imposing a splash delay.
    pub fn menu(
        &mut self,
        title: &str,
        items: &[OnboardingMenuItem],
    ) -> io::Result<OnboardingResult<usize>> {
        if items.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No onboarding actions",
            ));
        }
        let mut selected = 0;
        let mut dirty = true;
        loop {
            if self.guard.cancelled() {
                return Ok(OnboardingResult::Cancelled);
            }
            if dirty || self.options.animations {
                self.draw(&View::Menu {
                    title,
                    items,
                    selected,
                })?;
            }
            dirty = false;
            if crossterm::event::poll(FRAME_INTERVAL)? {
                match crossterm::event::read()? {
                    Event::Key(key) if key.kind != KeyEventKind::Release => {
                        if cancelled(key) {
                            return Ok(OnboardingResult::Cancelled);
                        }
                        match key.code {
                            KeyCode::Up | KeyCode::BackTab => {
                                selected = (selected + items.len() - 1) % items.len()
                            }
                            KeyCode::Down | KeyCode::Tab => selected = (selected + 1) % items.len(),
                            KeyCode::Home => selected = 0,
                            KeyCode::End => selected = items.len() - 1,
                            KeyCode::Enter if key.kind == KeyEventKind::Press => {
                                return Ok(OnboardingResult::Selected(selected));
                            }
                            KeyCode::Char(c)
                                if key.kind == KeyEventKind::Press && key.modifiers.is_empty() =>
                            {
                                if let Some(index) = c
                                    .to_digit(10)
                                    .and_then(|n| n.checked_sub(1))
                                    .map(|n| n as usize)
                                    && index < items.len()
                                {
                                    return Ok(OnboardingResult::Selected(index));
                                }
                            }
                            _ => {}
                        }
                        dirty = true;
                    }
                    Event::Resize(_, _) => dirty = true,
                    _ => {}
                }
            }
        }
    }

    /// Edit one public field. Paste never submits a value, and input is bounded.
    pub fn input(&mut self, field: OnboardingInput) -> io::Result<OnboardingResult<String>> {
        let mut editor = Editor::new(&field.value);
        let mut dirty = true;
        loop {
            if self.guard.cancelled() {
                return Ok(OnboardingResult::Cancelled);
            }
            if dirty || self.options.animations {
                self.draw(&View::Input {
                    field: &field,
                    editor: &editor,
                })?;
            }
            dirty = false;
            if crossterm::event::poll(FRAME_INTERVAL)? {
                let event = crossterm::event::read()?;
                if let Event::Key(key) = event
                    && key.kind != KeyEventKind::Release
                {
                    if cancelled(key) {
                        return Ok(OnboardingResult::Cancelled);
                    }
                    if key.code == KeyCode::Enter && key.kind == KeyEventKind::Press {
                        return Ok(OnboardingResult::Selected(editor.value));
                    }
                }
                dirty = editor.apply(event);
            }
        }
    }

    /// Keep a real async operation responsive to progress updates and cancellation.
    /// Dropping the returned future cancels the owned operation; no success is inferred.
    pub async fn wait<F: Future>(
        &mut self,
        operation: F,
        mut progress: watch::Receiver<OnboardingProgress>,
    ) -> io::Result<OnboardingResult<F::Output>> {
        tokio::pin!(operation);
        let mut tick = tokio::time::interval(FRAME_INTERVAL);
        let mut current = progress.borrow_and_update().clone();
        let mut dirty = true;
        loop {
            if self.guard.cancelled() {
                return Ok(OnboardingResult::Cancelled);
            }
            if dirty || self.options.animations {
                self.draw(&View::Progress(&current))?;
            }
            dirty = false;
            tokio::select! {
                result = &mut operation => return Ok(OnboardingResult::Selected(result)),
                _ = tick.tick() => {
                    if progress.has_changed().unwrap_or(false) {
                        current = progress.borrow_and_update().clone();
                        dirty = true;
                    }
                    if crossterm::event::poll(Duration::ZERO)? {
                        match crossterm::event::read()? {
                            Event::Key(key) if key.kind != KeyEventKind::Release && cancelled(key) => return Ok(OnboardingResult::Cancelled),
                            Event::Resize(_, _) => dirty = true,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn draw(&mut self, view: &View<'_>) -> io::Result<()> {
        let elapsed = self.started.elapsed();
        let context = &self.context;
        let options = self.options;
        self.terminal
            .draw(|frame| render::draw(frame, context, options, elapsed, view))?;
        Ok(())
    }
}

fn cancelled(key: KeyEvent) -> bool {
    key.code == KeyCode::Esc
        || (key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c' | 'd' | 'z')))
}

/// Treat external names and labels as text, never as terminal controls or bidi commands.
fn safe_text(value: &str) -> String {
    value
        .chars()
        .take(2048)
        .map(|c| {
            if c.is_control() || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') {
                ' '
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "onboarding_tests.rs"]
mod tests;
