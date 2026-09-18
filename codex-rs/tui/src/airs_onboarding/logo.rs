//! Prisma AIRS mark, transcribed from the owner's supplied cyan reference.
//! Half blocks preserve its transparent cutout without terminal image protocols.
use super::OnboardingOptions;
use crate::terminal_palette::rgb_color;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::time::Duration;

pub(super) const CYAN: (u8, u8, u8) = (0, 192, 232);

// Same coordinates as prisma-airs-mark.svg. The two filled regions leave the
// triangular center open to the terminal's own background, including light themes.
const LEFT: &[(f64, f64)] = &[
    (128.0, 0.0),
    (0.0, 128.0),
    (128.0, 256.0),
    (128.0, 171.0),
    (68.0, 171.0),
    (128.0, 52.0),
];
const RIGHT: &[(f64, f64)] = &[(128.0, 52.0), (191.0, 171.0), (128.0, 171.0)];

pub(super) fn draw(
    frame: &mut Frame<'_>,
    area: Rect,
    options: OnboardingOptions,
    elapsed: Duration,
) {
    // A half-block pixel is approximately square in a standard terminal font.
    let scale = (f64::from(area.width) / 191.0).min(f64::from(area.height) * 2.0 / 256.0);
    if scale == 0.0 {
        return;
    }
    let offset_x = (f64::from(area.width) - 191.0 * scale) / 2.0;
    let offset_y = (f64::from(area.height) * 2.0 - 256.0 * scale) / 2.0;
    for row in 0..area.height {
        for column in 0..area.width {
            let x = (f64::from(column) + 0.5 - offset_x) / scale;
            let y = (f64::from(row) * 2.0 + 0.5 - offset_y) / scale;
            let filled = |y| contains(LEFT, x, y) || contains(RIGHT, x, y);
            let symbol = match (filled(y), filled(y + 1.0 / scale)) {
                (true, true) => "█",
                (true, false) => "▀",
                (false, true) => "▄",
                (false, false) => continue,
            };
            let color = if !options.color {
                Color::Reset
            } else if options.animations {
                // Sweep light across the fixed silhouette; never rotate or
                // distort the supplied brand mark. Static mode uses exact cyan.
                let phase = elapsed.as_secs_f64() / 2.4 - (x + y) / 447.0;
                let light = (phase * std::f64::consts::TAU).cos().max(0.0).powi(4);
                let intensity = 0.8 + 0.2 * light;
                rgb_color((0, (192.0 * intensity) as u8, (232.0 * intensity) as u8))
            } else {
                rgb_color(CYAN)
            };
            if let Some(cell) = frame.buffer_mut().cell_mut((area.x + column, area.y + row)) {
                cell.set_symbol(symbol).set_fg(color);
            }
        }
    }
}

fn contains(polygon: &[(f64, f64)], x: f64, y: f64) -> bool {
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];
    for &(next_x, next_y) in polygon {
        if (next_y > y) != (previous.1 > y)
            && x < (previous.0 - next_x) * (y - next_y) / (previous.1 - next_y) + next_x
        {
            inside = !inside;
        }
        previous = (next_x, next_y);
    }
    inside
}
