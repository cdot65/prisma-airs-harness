use super::OnboardingContext;
use super::OnboardingOptions;
use super::View;
use super::safe_text;
use crate::terminal_palette::rgb_color;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::symbols::Marker;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::canvas::Line as CanvasLine;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

pub(super) fn draw(
    frame: &mut Frame<'_>,
    context: &OnboardingContext,
    options: OnboardingOptions,
    elapsed: Duration,
    view: &View<'_>,
) {
    let area = frame.area();
    frame.render_widget(Clear, area);
    if area.width == 0 || area.height == 0 {
        return;
    }
    let accent = if options.color {
        rgb_color((241, 119, 54))
    } else {
        Color::Reset
    };
    let bold = Style::default().add_modifier(Modifier::BOLD);
    let muted = Style::default().add_modifier(Modifier::DIM);
    let tiny = area.width < 40 || area.height < 14;
    let margin = u16::from(area.width >= 32);
    let width = area.width.saturating_sub(margin * 2).min(66);
    let left = area.x + (area.width - width) / 2;
    let top = area.y + u16::from(area.height >= 14);
    let content = Rect::new(left, top, width, area.height.saturating_sub(top - area.y));
    let header_height = if tiny {
        1
    } else if area.width >= 60 && area.height >= 30 {
        9
    } else if area.width >= 60 && area.height >= 24 {
        7
    } else if area.width >= 48 && area.height >= 20 {
        5
    } else {
        2
    };
    let header = Rect::new(content.x, content.y, content.width, header_height);
    if header_height >= 5 {
        let prism_width = if header_height >= 9 {
            25
        } else if header_height >= 7 {
            23
        } else {
            18
        };
        let angle = if options.animations {
            elapsed.as_secs_f64() * 0.65
        } else {
            0.55
        };
        prism(
            frame,
            Rect::new(header.x, header.y, prism_width, header.height),
            angle,
            accent,
        );
        let x = header.x + prism_width + 2;
        let y = header.y + header.height / 2 - 1;
        frame.render_widget(
            Paragraph::new("PRISMA AIRS").style(bold),
            Rect::new(
                x,
                y,
                header.width.saturating_sub(prism_width + 2),
                /*height*/ 1,
            ),
        );
        frame.render_widget(
            Paragraph::new("Your secure agent workspace").style(muted),
            Rect::new(
                x,
                y + 2,
                header.width.saturating_sub(prism_width + 2),
                /*height*/ 1,
            ),
        );
    } else {
        let title = if tiny && !context.environment.is_empty() {
            format!("AIRS · {}", safe_text(&context.environment))
        } else {
            "PRISMA AIRS".to_owned()
        };
        frame.render_widget(
            Paragraph::new(title).style(bold),
            Rect::new(header.x, header.y, header.width, /*height*/ 1),
        );
    }

    let mut body_y = header.bottom() + 1;
    if !tiny && !context.environment.is_empty() {
        let environment = Line::from(vec![
            Span::styled("Environment  ", muted),
            Span::styled(safe_text(&context.environment), bold),
        ]);
        frame.render_widget(
            Paragraph::new(environment),
            Rect::new(content.x, body_y, content.width, /*height*/ 1),
        );
        body_y += 1;
        if let Some(gateway) = &context.gateway
            && area.height >= 24
        {
            frame.render_widget(
                Paragraph::new(safe_text(gateway)).style(muted),
                Rect::new(content.x, body_y, content.width, /*height*/ 1),
            );
            body_y += 1;
        }
        body_y += 1;
    }
    let footer_y = area
        .bottom()
        .saturating_sub(1 + u16::from(area.height >= 14));
    let body = Rect::new(
        content.x,
        body_y.min(footer_y),
        content.width,
        footer_y.saturating_sub(body_y + 1),
    );
    let footer = Rect::new(content.x, footer_y, content.width, /*height*/ 1);

    match view {
        View::Menu {
            title,
            items,
            selected,
        } => {
            let title_height = u16::from(body.height >= 3);
            if title_height > 0 {
                frame.render_widget(
                    Paragraph::new(safe_text(title)).style(bold),
                    Rect::new(body.x, body.y, body.width, /*height*/ 1),
                );
            }
            let menu_y = body.y + title_height + u16::from(body.height >= 6);
            let rows = body.bottom().saturating_sub(menu_y);
            let item_height = if rows >= (items.len() * 2) as u16 {
                2
            } else {
                1
            };
            let visible = (rows / item_height).max(1) as usize;
            let first = selected.saturating_sub(visible.saturating_sub(1));
            for (index, item) in items.iter().enumerate().skip(first).take(visible) {
                let y = menu_y + ((index - first) as u16 * item_height);
                if y >= footer_y {
                    break;
                }
                let active = index == *selected;
                let marker = if active { "› " } else { "  " };
                let style = if active { bold } else { Style::default() };
                let line = Line::from(vec![
                    Span::styled(marker, Style::default().fg(accent)),
                    Span::styled(format!("{}. {}", index + 1, safe_text(&item.label)), style),
                ]);
                frame.render_widget(
                    Paragraph::new(line),
                    Rect::new(body.x, y, body.width, /*height*/ 1),
                );
                if item_height == 2 && y + 1 < footer_y {
                    frame.render_widget(
                        Paragraph::new(format!("     {}", safe_text(&item.detail))).style(muted),
                        Rect::new(body.x, y + 1, body.width, /*height*/ 1),
                    );
                }
            }
            let hint = if tiny {
                "↑↓ Move · Enter · Esc Exit"
            } else {
                "↑↓ Navigate   Enter Select   Esc / Ctrl+C Exit"
            };
            frame.render_widget(Paragraph::new(hint).style(muted), footer);
        }
        View::Input { field, editor } => {
            let mut y = body.y;
            if body.height >= 5 {
                frame.render_widget(
                    Paragraph::new(safe_text(&field.title)).style(bold),
                    Rect::new(body.x, y, body.width, /*height*/ 1),
                );
                y += 2;
            }
            frame.render_widget(
                Paragraph::new(safe_text(&field.label)).style(bold),
                Rect::new(body.x, y, body.width, /*height*/ 1),
            );
            y += 1;
            let input_width = body.width.saturating_sub(2) as usize;
            let (visible, cursor) = input_window(&editor.value, editor.cursor, input_width);
            frame.render_widget(
                Paragraph::new(format!("› {visible}")),
                Rect::new(body.x, y, body.width, /*height*/ 1),
            );
            if y < footer_y && body.width >= 3 {
                frame.set_cursor_position((body.x + 2 + cursor as u16, y));
            }
            y += 2;
            let detail = editor
                .error
                .map(str::to_owned)
                .or_else(|| field.error.clone())
                .unwrap_or_else(|| field.help.clone());
            frame.render_widget(
                Paragraph::new(safe_text(&detail))
                    .style(muted)
                    .wrap(Wrap { trim: false }),
                Rect::new(
                    body.x,
                    y.min(footer_y),
                    body.width,
                    footer_y.saturating_sub(y + 1),
                ),
            );
            let hint = if tiny {
                "Enter Continue · Esc Back"
            } else {
                "Enter Continue   Esc Back   Ctrl+U Clear"
            };
            frame.render_widget(Paragraph::new(hint).style(muted), footer);
        }
        View::Progress(progress) => {
            let mut lines = vec![
                Line::styled(safe_text(&progress.title), bold),
                Line::default(),
                Line::from(safe_text(&progress.detail)),
            ];
            if let Some(link) = &progress.link {
                lines.push(Line::default());
                lines.push(Line::styled("Open in your browser:", muted));
                lines.push(Line::from(safe_text(link)));
            }
            frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), body);
            frame.render_widget(Paragraph::new("Esc / Ctrl+C Cancel").style(muted), footer);
        }
    }
}

fn prism(frame: &mut Frame<'_>, area: Rect, angle: f64, accent: Color) {
    let vertices = [
        (0.0, 1.05, 0.0),
        (0.0, -1.05, 0.0),
        (0.9, 0.0, 0.0),
        (0.0, 0.0, 0.9),
        (-0.9, 0.0, 0.0),
        (0.0, 0.0, -0.9),
    ];
    let projected: Vec<_> = vertices
        .iter()
        .map(|&(x, y, z)| {
            let x1 = x * angle.cos() + z * angle.sin();
            let z1 = z * angle.cos() - x * angle.sin();
            let scale = 3.6 / (3.6 + z1);
            (x1 * scale, (y * 0.86 - z1 * 0.5) * scale, z1)
        })
        .collect();
    let edges = [
        (0, 2),
        (0, 3),
        (0, 4),
        (0, 5),
        (1, 2),
        (1, 3),
        (1, 4),
        (1, 5),
        (2, 3),
        (3, 4),
        (4, 5),
        (5, 2),
    ];
    let canvas = Canvas::default()
        .marker(Marker::HalfBlock)
        .x_bounds([-1.45, 1.45])
        .y_bounds([-1.2, 1.2])
        .paint(|context| {
            for (a, b) in edges {
                let a = projected[a];
                let b = projected[b];
                let color = if a.2 + b.2 > 0.5 && accent != Color::Reset {
                    rgb_color((139, 75, 45))
                } else {
                    accent
                };
                context.draw(&CanvasLine {
                    x1: a.0,
                    y1: a.1,
                    x2: b.0,
                    y2: b.1,
                    color,
                });
            }
        });
    frame.render_widget(canvas, area);
}

pub(super) fn input_window(value: &str, cursor: usize, width: usize) -> (&str, usize) {
    if width == 0 {
        return ("", 0);
    }
    let mut start = 0;
    while value[start..cursor].width() >= width {
        start += value[start..].chars().next().map_or(0, char::len_utf8);
    }
    (&value[start..], value[start..cursor].width())
}
