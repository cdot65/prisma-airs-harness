use super::input::Editor;
use super::*;
use crossterm::event::KeyEventState;
use pretty_assertions::assert_eq;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Color;

fn items() -> Vec<OnboardingMenuItem> {
    vec![
        OnboardingMenuItem {
            label: "Sign in with company SSO".into(),
            detail: "Continue securely in your browser".into(),
        },
        OnboardingMenuItem {
            label: "Use a workspace API key".into(),
            detail: "Enter a key from your administrator".into(),
        },
        OnboardingMenuItem {
            label: "Choose another environment".into(),
            detail: "Keep each workspace and identity separate".into(),
        },
    ]
}

fn render_buffer(width: u16, height: u16, options: OnboardingOptions, view: &View<'_>) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| {
            render::draw(
                frame,
                &OnboardingContext {
                    environment: "work".into(),
                    gateway: Some("https://gateway.example.com/v1".into()),
                },
                options,
                Duration::from_millis(/*millis*/ 800),
                view,
                &mut 0,
            )
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

fn text(buffer: &Buffer) -> String {
    buffer
        .content()
        .chunks(buffer.area.width.max(1) as usize)
        .map(|row| {
            row.iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>()
                .trim_end()
                .to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn welcome_snapshots_cover_desktop_compact_and_tiny_layouts() {
    let items = items();
    let view = View::Menu {
        title: "Sign in to continue",
        items: &items,
        selected: 0,
    };
    let options = OnboardingOptions {
        animations: false,
        color: false,
    };
    insta::assert_snapshot!(
        "airs_welcome_80x24",
        text(&render_buffer(
            /*width*/ 80, /*height*/ 24, options, &view
        ))
    );
    insta::assert_snapshot!(
        "airs_welcome_120x40",
        text(&render_buffer(
            /*width*/ 120, /*height*/ 40, options, &view
        ))
    );
    insta::assert_snapshot!(
        "airs_welcome_48x18",
        text(&render_buffer(
            /*width*/ 48, /*height*/ 18, options, &view
        ))
    );
    insta::assert_snapshot!(
        "airs_welcome_30x10",
        text(&render_buffer(
            /*width*/ 30, /*height*/ 10, options, &view
        ))
    );
}

#[test]
fn selected_action_remains_visible_when_the_menu_scrolls_in_a_small_terminal() {
    let items = items();
    let view = View::Menu {
        title: "Sign in",
        items: &items,
        selected: 2,
    };
    let output = text(&render_buffer(
        /*width*/ 48,
        /*height*/ 8,
        OnboardingOptions {
            animations: false,
            color: true,
        },
        &view,
    ));
    assert!(output.contains("› 3. Choose another environment"));
    assert!(output.contains("Esc Exit"));
}

#[test]
fn static_colorless_rendering_uses_terminal_foreground_for_every_cell() {
    let items = items();
    let view = View::Menu {
        title: "Sign in",
        items: &items,
        selected: 0,
    };
    let buffer = render_buffer(
        /*width*/ 80,
        /*height*/ 24,
        OnboardingOptions {
            animations: false,
            color: false,
        },
        &view,
    );
    assert!(
        buffer
            .content()
            .iter()
            .all(|cell| cell.fg == Color::Reset && cell.bg == Color::Reset)
    );
}

#[test]
fn prisma_mark_preserves_its_silhouette_and_transparency_during_animation() {
    let render = |animations, elapsed| {
        let mut terminal = Terminal::new(TestBackend::new(/*width*/ 20, /*height*/ 12)).unwrap();
        terminal
            .draw(|frame| {
                logo::draw(
                    frame,
                    frame.area(),
                    OnboardingOptions {
                        animations,
                        color: true,
                    },
                    elapsed,
                );
            })
            .unwrap();
        terminal.backend().buffer().clone()
    };
    let static_mark = render(/*animations*/ false, Duration::ZERO);
    let first = render(/*animations*/ true, Duration::ZERO);
    let later = render(
        /*animations*/ true,
        Duration::from_millis(/*millis*/ 800),
    );
    insta::assert_snapshot!("prisma_airs_mark", text(&static_mark));
    assert_eq!(text(&first), text(&static_mark));
    assert_eq!(text(&later), text(&static_mark));
    assert_ne!(first, later, "The light sweep should animate the mark");
    assert_eq!(
        static_mark,
        render(/*animations*/ false, Duration::from_secs(/*secs*/ 8))
    );
    for buffer in [&static_mark, &first, &later] {
        assert!(buffer.content().iter().all(|cell| cell.bg == Color::Reset));
    }
    assert!(
        static_mark
            .content()
            .iter()
            .any(|cell| cell.symbol() == "█")
    );
    assert!(
        static_mark
            .content()
            .iter()
            .filter(|cell| cell.symbol() != " ")
            .all(|cell| cell.fg == crate::terminal_palette::rgb_color(logo::CYAN))
    );
}

#[test]
fn input_and_progress_snapshots_keep_context_and_recovery_visible() {
    let field = OnboardingInput {
        title: "Connect your environment".into(),
        label: "AI Gateway URL".into(),
        help: "Use the inference URL supplied by your administrator.".into(),
        value: "https://gateway.example.com/v1".into(),
        error: Some("Enter an HTTPS URL without a username or password.".into()),
    };
    let editor = Editor::new(&field.value);
    let options = OnboardingOptions {
        animations: false,
        color: false,
    };
    insta::assert_snapshot!(
        "airs_input_80x24",
        text(&render_buffer(
            /*width*/ 80,
            /*height*/ 24,
            options,
            &View::Input {
                field: &field,
                editor: &editor
            }
        ))
    );
    let progress = OnboardingProgress {
        title: "Waiting for company sign-in".into(),
        detail: "Complete sign-in in your browser, then return here.".into(),
        link: Some("https://sso.example.com/authorize?client_id=harness-native".into()),
    };
    insta::assert_snapshot!(
        "airs_progress_80x24",
        text(&render_buffer(
            /*width*/ 80,
            /*height*/ 24,
            options,
            &View::Progress(&progress)
        ))
    );
}

#[test]
fn resize_extremes_never_panic_or_lose_the_buffers_dimensions() {
    let items = items();
    let field = OnboardingInput {
        title: "Connection".into(),
        label: "Gateway".into(),
        help: "Help".into(),
        value: "long value".into(),
        error: None,
    };
    let editor = Editor::new(&field.value);
    let progress = OnboardingProgress {
        title: "Waiting".into(),
        detail: "Continue in your browser".into(),
        link: Some("https://example.com".repeat(100)),
    };
    let options = OnboardingOptions {
        animations: true,
        color: true,
    };
    for width in [0, 1, 2, 10, 29, 30, 40, 48, 60, 80, 120] {
        for height in [0, 1, 2, 3, 8, 10, 14, 18, 24, 40] {
            for view in [
                View::Menu {
                    title: "Sign in",
                    items: &items,
                    selected: 2,
                },
                View::Input {
                    field: &field,
                    editor: &editor,
                },
                View::Progress(&progress),
            ] {
                let buffer = render_buffer(width, height, options, &view);
                assert_eq!((buffer.area.width, buffer.area.height), (width, height));
            }
        }
    }
}

#[test]
fn public_text_cannot_inject_terminal_or_direction_controls() {
    assert_eq!(
        safe_text("work\x1b]0;bad\x07\n\u{202e}name"),
        "work ]0;bad   name"
    );
    let items = vec![OnboardingMenuItem {
        label: "\x1b[2JCompany".into(),
        detail: "\x07bell".into(),
    }];
    let output = text(&render_buffer(
        /*width*/ 80,
        /*height*/ 24,
        OnboardingOptions {
            animations: false,
            color: false,
        },
        &View::Menu {
            title: "Welcome",
            items: &items,
            selected: 0,
        },
    ));
    assert!(!output.contains('\x1b'));
    assert!(!output.contains('\x07'));
    assert!(output.contains("Company"));
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

#[test]
fn unicode_editing_and_middle_paste_preserve_character_boundaries() {
    let mut editor = Editor::new("a界c");
    editor.apply(key(KeyCode::Left));
    editor.apply(Event::Paste("é".into()));
    editor.apply(key(KeyCode::Backspace));
    editor.apply(key(KeyCode::Left));
    editor.apply(key(KeyCode::Delete));
    assert_eq!((editor.value.as_str(), editor.cursor), ("ac", 1));
    editor.apply(key(KeyCode::Home));
    editor.apply(Event::Paste("https://".into()));
    assert_eq!((editor.value.as_str(), editor.cursor), ("https://ac", 8));
}

#[test]
fn multiline_control_and_oversized_paste_cannot_submit_or_replace_a_field() {
    for pasted in [
        "first\nsecond".into(),
        "first\rsecond".into(),
        "\x1b[2J".into(),
        "x".repeat(2049),
    ] {
        let mut editor = Editor::new("preserved");
        editor.apply(Event::Paste(pasted));
        assert_eq!(editor.value, "preserved");
        assert!(editor.error.is_some());
    }
}

#[test]
fn release_events_do_not_duplicate_typed_characters_and_clear_is_explicit() {
    let mut editor = Editor::new("a");
    editor.apply(Event::Key(KeyEvent {
        code: KeyCode::Char('b'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Release,
        state: KeyEventState::NONE,
    }));
    assert_eq!(editor.value, "a");
    editor.apply(Event::Key(KeyEvent::new(
        KeyCode::Char('u'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!((editor.value.as_str(), editor.cursor), ("", 0));
}

#[test]
fn cancel_keys_are_distinct_from_ordinary_text_input() {
    for key in [
        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
        KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL),
    ] {
        assert!(cancelled(key));
    }
    assert!(!cancelled(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::NONE
    )));
    assert!(!cancelled(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE
    )));
}

#[test]
fn long_unicode_input_keeps_the_cursor_visible_on_a_character_boundary() {
    assert_eq!(
        super::render::input_window("ab界cdef", /*cursor*/ 7, /*width*/ 4),
        ("cdef", 2)
    );
    assert_eq!(
        super::render::input_window("abcdef", /*cursor*/ 0, /*width*/ 4),
        ("abcdef", 0)
    );
    assert_eq!(
        super::render::input_window("abcdef", /*cursor*/ 6, /*width*/ 0),
        ("", 0)
    );
}

#[test]
fn long_authorization_links_remain_complete_and_scroll_to_the_end() {
    let progress = OnboardingProgress {
        title: "Waiting for company sign-in".into(),
        detail: "Complete sign-in in your browser.".into(),
        link: Some(format!(
            "https://sso.example/authorize?client_id={}&final=VISIBLE",
            "a".repeat(3000)
        )),
    };
    let mut terminal = Terminal::new(TestBackend::new(/*width*/ 80, /*height*/ 24)).unwrap();
    let mut scroll = u16::MAX;
    terminal
        .draw(|frame| {
            render::draw(
                frame,
                &OnboardingContext::default(),
                OnboardingOptions {
                    animations: false,
                    color: false,
                },
                Duration::ZERO,
                &View::Progress(&progress),
                &mut scroll,
            )
        })
        .unwrap();
    assert!(text(terminal.backend().buffer()).contains("final=VISIBLE"));
    assert!(scroll > 0 && scroll < u16::MAX);
}

#[test]
fn outcome_message_snapshot_shows_saved_credentials_and_denied_access() {
    let progress = OnboardingProgress {
        title: "Signed in · gateway access needs attention".into(),
        detail: "Credential saved; the gateway denied access. Check your workspace permissions, then retry with airs doctor --verify-access.".into(), link: None,
    };
    insta::assert_snapshot!(
        "airs_access_denied_80x24",
        text(&render_buffer(
            /*width*/ 80,
            /*height*/ 24,
            OnboardingOptions {
                animations: false,
                color: false
            },
            &View::Message(&progress)
        ))
    );
}
