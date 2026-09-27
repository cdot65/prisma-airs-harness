//! Shift-click keeps the original anchor and text unit without changing AIRS link activation.

use super::*;
use crate::history_cell::AgentMarkdownCell;
use pretty_assertions::assert_eq;

fn transcript(markdown: &str, width: u16) -> (TranscriptView, Vec<Arc<dyn HistoryCell>>) {
    let cells: Vec<Arc<dyn HistoryCell>> = vec![Arc::new(AgentMarkdownCell::new(
        markdown.into(),
        std::path::Path::new("/"),
    ))];
    let mut view = TranscriptView::default();
    let area = Rect::new(/*x*/ 2, /*y*/ 1, width, /*height*/ 8);
    view.render(area, &mut Buffer::empty(area), &cells);
    (view, cells)
}

fn mouse(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

#[test]
fn shift_click_extends_a_double_clicked_word_in_both_directions() {
    let (mut view, cells) = transcript("alpha beta gamma", /*width*/ 24);
    let area = view.area;
    let mut buffer = Buffer::empty(area);
    for _ in 0..2 {
        if let Some((at, ..)) = &mut view.last_click {
            *at = std::time::Instant::now();
        }
        for kind in [
            MouseEventKind::Down(MouseButton::Left),
            MouseEventKind::Up(MouseButton::Left),
        ] {
            view.handle_mouse(mouse(kind, /*column*/ 11, /*row*/ 1), &cells);
            view.render(area, &mut buffer, &cells);
        }
    }
    assert_eq!(view.selected_text(&cells).as_deref(), Some("beta"));

    let mut frames = Vec::new();
    for (column, expected) in [(17, "beta gamma"), (5, "alpha beta"), (17, "beta gamma")] {
        for kind in [
            MouseEventKind::Down(MouseButton::Left),
            MouseEventKind::Up(MouseButton::Left),
        ] {
            let action = view.handle_mouse(
                MouseEvent {
                    modifiers: KeyModifiers::SHIFT,
                    ..mouse(kind, column, /*row*/ 1)
                },
                &cells,
            );
            assert!(matches!(action, Some(ViewAction::Changed)));
            view.render(area, &mut buffer, &cells);
            assert_eq!(view.selected_text(&cells).as_deref(), Some(expected));
        }
        frames.push(format!("{buffer:?}"));
    }
    // An ordinary click after extending still starts a fresh, empty selection.
    for kind in [
        MouseEventKind::Down(MouseButton::Left),
        MouseEventKind::Up(MouseButton::Left),
    ] {
        view.handle_mouse(mouse(kind, /*column*/ 17, /*row*/ 1), &cells);
    }
    assert_eq!(view.selected_text(&cells), None);
    insta::assert_snapshot!(frames.join("\n\n"));
}

#[test]
fn shift_click_preserves_selection_units_and_allows_dragging() {
    for (clicks, expected) in [(1, "beta gam"), (3, "alpha beta gamma")] {
        let (mut view, cells) = transcript("alpha beta gamma", /*width*/ 24);
        view.begin_selection(&cells, /*column*/ 10, /*row*/ 1, clicks);
        view.handle_mouse(
            mouse(
                MouseEventKind::Drag(MouseButton::Left),
                /*column*/ 14,
                /*row*/ 1,
            ),
            &cells,
        );
        view.handle_mouse(
            mouse(
                MouseEventKind::Up(MouseButton::Left),
                /*column*/ 14,
                /*row*/ 1,
            ),
            &cells,
        );
        for kind in [
            MouseEventKind::Down(MouseButton::Left),
            MouseEventKind::Drag(MouseButton::Left),
            MouseEventKind::Up(MouseButton::Left),
        ] {
            let column = if matches!(kind, MouseEventKind::Down(_)) {
                16
            } else {
                18
            };
            view.handle_mouse(
                MouseEvent {
                    modifiers: KeyModifiers::SHIFT,
                    ..mouse(kind, column, /*row*/ 1)
                },
                &cells,
            );
        }
        assert_eq!(view.selected_text(&cells).as_deref(), Some(expected));
    }
}
