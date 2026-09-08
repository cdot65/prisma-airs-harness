use super::*;
use crossterm::event::KeyEvent;
use pretty_assertions::assert_eq;

fn apply_enter_event(value: &mut String, event: Event) -> anyhow::Result<InputResult> {
    apply_event(value, event, Submission::Enter)
}

#[test]
fn paste_is_bounded_and_cannot_inject_terminal_controls() {
    let mut value = String::new();
    apply_enter_event(&mut value, Event::Paste(" workspace-test-key\n".into())).unwrap();
    assert_eq!(value, "workspace-test-key");
    for invalid in ["key\nother", "key\u{1b}[2J", "key other"] {
        assert!(apply_enter_event(&mut value, Event::Paste(invalid.into())).is_err());
        assert_eq!(value, "workspace-test-key");
    }
    assert!(apply_enter_event(&mut value, Event::Paste("x".repeat(MAX_KEY_BYTES))).is_err());
    assert_eq!(value, "workspace-test-key");
}

#[test]
fn editing_and_cancellation_do_not_submit_a_secret() {
    let mut value = "test-key".to_owned();
    apply_enter_event(
        &mut value,
        Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
    )
    .unwrap();
    assert_eq!(value, "test-ke");
    for code in [KeyCode::Char('c'), KeyCode::Char('d'), KeyCode::Char('z')] {
        assert!(matches!(
            apply_enter_event(
                &mut value,
                Event::Key(KeyEvent::new(code, KeyModifiers::CONTROL))
            )
            .unwrap(),
            InputResult::Cancel
        ));
    }
    apply_enter_event(
        &mut value,
        Event::Key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL)),
    )
    .unwrap();
    assert_eq!(value, "");
    assert!(matches!(
        apply_enter_event(
            &mut value,
            Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
        )
        .unwrap(),
        InputResult::Cancel
    ));
}

#[test]
fn workspace_key_prompt_snapshot() {
    insta::assert_snapshot!(PROMPT);
}

#[test]
fn typed_invalid_characters_are_rejected_without_silently_changing_the_key() {
    for character in ['é', ' ', '\t', '\0', '\u{1b}', '\u{200b}'] {
        let mut value = "unchanged-key".to_owned();
        let error = apply_enter_event(
            &mut value,
            Event::Key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE)),
        )
        .err()
        .unwrap();
        assert_eq!(value, "unchanged-key");
        assert!(!error.to_string().contains("unchanged-key"));
    }
    let mut value = "x".repeat(MAX_KEY_BYTES);
    assert!(
        apply_enter_event(
            &mut value,
            Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
        )
        .is_err()
    );
    assert_eq!(value.len(), MAX_KEY_BYTES);
}

#[test]
fn windows_pasted_enter_never_submits_a_prefix() {
    let mut value = "prefix".to_owned();
    assert!(
        apply_event(
            &mut value,
            Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Submission::ControlEnter,
        )
        .is_err()
    );
    assert_eq!(value, "prefix");
    assert!(matches!(
        apply_event(
            &mut value,
            Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)),
            Submission::ControlEnter,
        )
        .unwrap(),
        InputResult::Submit
    ));
    assert!(WINDOWS_PROMPT.contains("Ctrl+Enter"));
}

#[test]
fn release_events_cannot_append_or_submit() {
    let mut value = "key".to_owned();
    for code in [KeyCode::Char('x'), KeyCode::Enter] {
        assert!(matches!(
            apply_enter_event(
                &mut value,
                Event::Key(KeyEvent::new_with_kind(
                    code,
                    KeyModifiers::NONE,
                    KeyEventKind::Release
                )),
            )
            .unwrap(),
            InputResult::Continue
        ));
    }
    assert_eq!(value, "key");
}
