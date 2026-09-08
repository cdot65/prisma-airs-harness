//! Interactive secret entry owned by the CLI, never by the model or a shell.
use anyhow::Context;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use crossterm::terminal;
use std::io::IsTerminal;
use std::io::Write;
use std::time::Duration;

#[path = "airs_secret_terminal.rs"]
mod terminal_guard;

const MAX_KEY_BYTES: usize = 16_384;
const PROMPT: &str = "Workspace API key (input hidden; Esc to cancel): ";
const WINDOWS_PROMPT: &str =
    "Workspace API key (input hidden; Ctrl+Enter to submit; Esc to cancel): ";

#[derive(Clone, Copy)]
enum Submission {
    Enter,
    ControlEnter,
}

enum InputResult {
    Continue,
    Submit,
    Cancel,
}

fn apply_event(
    value: &mut String,
    event: Event,
    submission: Submission,
) -> anyhow::Result<InputResult> {
    match event {
        Event::Key(key) if key.kind != KeyEventKind::Release => {
            if key.code == KeyCode::Enter {
                anyhow::ensure!(
                    matches!(submission, Submission::Enter)
                        || key.modifiers == KeyModifiers::CONTROL,
                    "Use Ctrl+Enter to submit on Windows; a pasted newline cannot submit a key"
                );
                return Ok(InputResult::Submit);
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(match key.code {
                    KeyCode::Char('c' | 'd' | 'z') => InputResult::Cancel,
                    KeyCode::Char('u') => {
                        value.clear();
                        InputResult::Continue
                    }
                    _ => anyhow::bail!("Unsupported control character in workspace API key"),
                });
            }
            match key.code {
                KeyCode::Esc => return Ok(InputResult::Cancel),
                KeyCode::Backspace => {
                    value.pop();
                }
                KeyCode::Char(c) => {
                    anyhow::ensure!(
                        c.is_ascii_graphic(),
                        "Workspace API keys must contain only printable ASCII without whitespace"
                    );
                    anyhow::ensure!(value.len() < MAX_KEY_BYTES, "Workspace API key is too long");
                    value.push(c);
                }
                KeyCode::Tab | KeyCode::BackTab => {
                    anyhow::bail!("Workspace API keys cannot contain whitespace");
                }
                _ => {}
            }
        }
        Event::Paste(paste) => {
            let paste = paste.trim_ascii();
            anyhow::ensure!(
                paste.bytes().all(|c| c.is_ascii_graphic())
                    && value.len().saturating_add(paste.len()) <= MAX_KEY_BYTES,
                "Paste a single workspace API key without embedded whitespace"
            );
            value.push_str(paste);
        }
        _ => {}
    }
    Ok(InputResult::Continue)
}

pub(super) fn workspace_key() -> anyhow::Result<String> {
    anyhow::ensure!(
        std::io::stdin().is_terminal() && std::io::stderr().is_terminal(),
        "Interactive login requires a terminal. Automation can use login --with-api-key with stdin."
    );
    anyhow::ensure!(
        !terminal::is_raw_mode_enabled()?,
        "Finish the current interactive session before signing in"
    );
    // Disable echo before displaying the prompt, including before a paste can arrive.
    let restore =
        terminal_guard::SecretTerminal::open().context("Unable to open secure terminal input")?;
    let (prompt, submission) = if cfg!(windows) {
        (WINDOWS_PROMPT, Submission::ControlEnter)
    } else {
        (PROMPT, Submission::Enter)
    };
    write!(std::io::stderr(), "{prompt}")?;
    std::io::stderr().flush()?;
    let mut value = String::new();
    loop {
        anyhow::ensure!(
            !restore.cancelled(),
            "Sign-in cancelled; credentials were not changed"
        );
        // A finite poll lets termination signals take the ordinary cleanup path.
        if !crossterm::event::poll(Duration::from_millis(50))? {
            continue;
        }
        match apply_event(&mut value, crossterm::event::read()?, submission)? {
            InputResult::Continue => {}
            InputResult::Submit => {
                anyhow::ensure!(
                    !restore.cancelled(),
                    "Sign-in cancelled; credentials were not changed"
                );
                anyhow::ensure!(!value.is_empty(), "No workspace API key was entered");
                return Ok(value);
            }
            InputResult::Cancel => anyhow::bail!("Sign-in cancelled; credentials were not changed"),
        }
    }
}

#[cfg(test)]
#[path = "airs_secret_prompt_tests.rs"]
mod tests;
