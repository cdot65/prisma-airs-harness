use super::*;
use crate::terminal_palette::with_test_default_colors;
use crate::terminal_probe::DefaultColors;
use ratatui::style::Modifier;

#[test]
fn selection_fill_covers_wrapping_and_moves_without_coloring_disabled_rows() {
    for bg in [(20, 20, 20), (250, 250, 250)] {
        with_test_default_colors(
            DefaultColors {
                fg: (128, 128, 128),
                bg,
            },
            || {
                for width in [1, 24, 80] {
                    let rows = vec![
                        GenericDisplayRow {
                            name: "Sign in through the AI Gateway".into(),
                            ..Default::default()
                        },
                        GenericDisplayRow {
                            name: "Refresh".into(),
                            ..Default::default()
                        },
                        GenericDisplayRow {
                            name: "Unavailable".into(),
                            is_disabled: true,
                            ..Default::default()
                        },
                    ];
                    let area = Rect::new(0, 0, width, 80);
                    let mut buffer = Buffer::empty(area);
                    let selected = selection_style();
                    let state = ScrollState {
                        selected_idx: Some(0),
                        ..Default::default()
                    };
                    render_rows(area, &mut buffer, &rows, &state, 3, "none");
                    let lines =
                        wrap_row_lines(&rows[0], 0, width, SelectionDescriptionLayout::Columns)
                            .len();
                    for y in 0..lines as u16 {
                        for x in 0..width {
                            let cell = &buffer[(x, y)];
                            assert_eq!(
                                (cell.fg, cell.bg),
                                (selected.fg.unwrap(), selected.bg.unwrap())
                            );
                            assert!(cell.modifier.contains(Modifier::BOLD));
                        }
                    }
                    // Repaint the same surface as the real menu does before moving focus.
                    render_menu_surface(area, &mut buffer);
                    let state = ScrollState {
                        selected_idx: Some(1),
                        ..Default::default()
                    };
                    render_rows(area, &mut buffer, &rows, &state, 3, "none");
                    assert_ne!(buffer[(0, 0)].bg, selected.bg.unwrap());
                    assert_eq!(buffer[(width - 1, lines as u16)].bg, selected.bg.unwrap());
                    let mut disabled = Buffer::empty(area);
                    let state = ScrollState {
                        selected_idx: Some(2),
                        ..Default::default()
                    };
                    render_rows(area, &mut disabled, &rows, &state, 3, "none");
                    assert!(
                        disabled
                            .content
                            .iter()
                            .all(|cell| cell.bg != selected.bg.unwrap())
                    );
                }
            },
        );
    }
}

#[test]
fn truncated_single_line_selection_includes_ellipsis_and_trailing_fill() {
    with_test_default_colors(
        DefaultColors {
            fg: (220, 220, 220),
            bg: (20, 20, 20),
        },
        || {
            for width in [1, 8, 80] {
                let rows = vec![GenericDisplayRow {
                    name: "A long gateway connection label".into(),
                    ..Default::default()
                }];
                let state = ScrollState {
                    selected_idx: Some(0),
                    ..Default::default()
                };
                let area = Rect::new(0, 0, width, 1);
                let mut buffer = Buffer::empty(area);
                render_rows_single_line(area, &mut buffer, &rows, &state, 1, "none");
                let expected = selection_style();
                for cell in buffer.content {
                    assert_eq!(
                        (cell.fg, cell.bg),
                        (expected.fg.unwrap(), expected.bg.unwrap())
                    );
                    assert!(cell.modifier.contains(Modifier::BOLD));
                }
            }
        },
    );
}
