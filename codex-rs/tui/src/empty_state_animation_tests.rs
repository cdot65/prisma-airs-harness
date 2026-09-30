use std::time::Duration;

use crossterm::event::KeyModifiers;
use crossterm::event::MouseButton;
use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::ComposerState;
use super::EmptyStateAnimation;
use super::FRAME_INTERVAL;
use super::geometry::EXTENT;
use super::geometry::FIELDS;
use super::geometry::GRID;
use super::geometry::STEP;
use super::lighting::Lighting;
use super::renderer::Renderer;
use crate::motion::MotionMode;

fn braille_cells(buffer: &Buffer) -> usize {
    buffer
        .content
        .iter()
        .filter(|cell| {
            cell.symbol()
                .chars()
                .next()
                .is_some_and(|c| ('\u{2801}'..='\u{28ff}').contains(&c))
        })
        .count()
}

fn field_at(field: &[f32], x: f64, y: f64) -> f32 {
    let column = ((x + EXTENT) / STEP).round() as usize;
    let row = ((y + EXTENT) / STEP).round() as usize;
    field[row * GRID + column]
}

#[test]
fn airs_field_keeps_the_triangular_cutout_open() {
    // The resting field is the AIRS mark. SVG (108, 131) is the centroid of the open
    // triangle; (40, 128) is inside the left arm.
    let cutout = field_at(&FIELDS[1], (108.0 - 95.5) / 128.0, (131.0 - 128.0) / 128.0);
    let arm = field_at(&FIELDS[1], (40.0 - 95.5) / 128.0, 0.0);
    let outside = field_at(&FIELDS[1], -1.0, -1.0);
    assert!(cutout < 0.0, "cutout should be outside the mark: {cutout}");
    assert!(arm > 0.0, "left arm should be inside the mark: {arm}");
    assert!(
        outside < 0.0,
        "corner should be outside the mark: {outside}"
    );
}

#[test]
fn palo_alto_field_has_three_bars_with_open_gaps() {
    // Raster (175, 246) is the center of the left bar; (330, 150) is the white notch above
    // where the left and middle bars meet. Bounds center is (299.5, 299.5), scale 249.5.
    let bar = field_at(&FIELDS[0], (175.0 - 299.5) / 249.5, (246.0 - 299.5) / 249.5);
    let gap = field_at(&FIELDS[0], (330.0 - 299.5) / 249.5, (150.0 - 299.5) / 249.5);
    let right_bar = field_at(&FIELDS[0], (450.0 - 299.5) / 249.5, (350.0 - 299.5) / 249.5);
    assert!(bar > 0.0, "left bar should be inside the mark: {bar}");
    assert!(
        right_bar > 0.0,
        "right bar should be inside the mark: {right_bar}"
    );
    assert!(gap < 0.0, "the notch between bars stays open: {gap}");
}

fn average_rgb(cells: &[super::renderer::Cell]) -> (f64, f64, f64) {
    let lit: Vec<_> = cells.iter().filter(|cell| cell.dots != 0).collect();
    let sum = lit.iter().fold((0.0, 0.0, 0.0), |acc, cell| {
        let [_, r, g, b] = cell.rgb.to_be_bytes();
        (
            acc.0 + f64::from(r),
            acc.1 + f64::from(g),
            acc.2 + f64::from(b),
        )
    });
    let n = lit.len().max(1) as f64;
    (sum.0 / n, sum.1 / n, sum.2 / n)
}

#[test]
fn renderer_rests_on_cyan_airs_and_morphs_into_orange_palo_alto() {
    let background = (15, 20, 37);
    let lights = [Lighting::palo_alto(background), Lighting::airs(background)];
    let lights = [&lights[0], &lights[1]];
    let mut renderer = Renderer::default();
    let settled: Vec<_> = renderer.frame(60, 21, 0.5, lights).to_vec();
    let lit = settled.iter().filter(|cell| cell.dots != 0).count();
    assert!(lit > 100, "settled pose should light many cells: {lit}");
    let (r, g, _) = average_rgb(&settled);
    assert!(g > r * 1.5, "resting pose is cyan: r={r:.0} g={g:.0}");

    // Late in a half-turn the spin has finished and the morph is complete.
    let morphed: Vec<_> = renderer.frame(60, 21, 0.95, lights).to_vec();
    assert_ne!(settled, morphed, "the morph renders a different shape");
    let (r, g, _) = average_rgb(&morphed);
    assert!(r > g * 1.5, "morphed pose is orange: r={r:.0} g={g:.0}");
}

#[test]
fn fresh_conversation_shows_the_settled_mark_only_with_an_empty_composer() {
    let mut animation = EmptyStateAnimation::default();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 40));
    let available = Rect::new(0, 2, 80, 30);
    let delay = animation.render_first_screen(
        available,
        &mut buffer,
        Some(ComposerState::Empty),
        MotionMode::Animated,
    );
    assert_eq!(delay, None, "the settled mark schedules no redraws");
    assert!(braille_cells(&buffer) > 100);

    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 40));
    let delay = animation.render_first_screen(
        available,
        &mut buffer,
        Some(ComposerState::Draft),
        MotionMode::Animated,
    );
    assert_eq!(delay, None);
    assert_eq!(braille_cells(&buffer), 0, "a draft hides the mark");

    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 40));
    animation.render_first_screen(
        available,
        &mut buffer,
        Some(ComposerState::Empty),
        MotionMode::Reduced,
    );
    assert_eq!(braille_cells(&buffer), 0, "reduced motion hides the mark");
}

#[test]
fn dismissed_animation_never_paints() {
    let mut animation = EmptyStateAnimation::default();
    animation.dismiss();
    assert!(!animation.is_eligible());
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 40));
    let delay = animation.render_in(
        Rect::new(10, 5, 60, 21),
        &mut buffer,
        super::Presentation::Animated,
    );
    assert_eq!(delay, None);
    assert_eq!(braille_cells(&buffer), 0);
}

#[test]
fn clicking_the_mark_replays_the_spin() {
    let mut animation = EmptyStateAnimation::default();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 40));
    let available = Rect::new(0, 2, 80, 30);
    animation.render_first_screen(
        available,
        &mut buffer,
        Some(ComposerState::Empty),
        MotionMode::Animated,
    );
    let stage = animation.stage.expect("stage is recorded after painting");
    let mouse = |column, row, kind| MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    };
    assert!(!animation.handle_mouse(mouse(
        stage.x,
        stage.y.saturating_sub(1),
        MouseEventKind::Down(MouseButton::Left)
    )));
    assert!(!animation.handle_mouse(mouse(
        stage.x + 1,
        stage.y + 1,
        MouseEventKind::Down(MouseButton::Right)
    )));
    assert!(animation.handle_mouse(mouse(
        stage.x + stage.width / 2,
        stage.y + stage.height / 2,
        MouseEventKind::Down(MouseButton::Left)
    )));
    let delay = animation.render_first_screen(
        available,
        &mut buffer,
        Some(ComposerState::Empty),
        MotionMode::Animated,
    );
    assert_eq!(
        delay,
        Some(FRAME_INTERVAL),
        "a replay keeps scheduling frames"
    );
    assert!(animation.replaying);

    // Finishing the spin and its fade returns to the idle pose without further redraws.
    animation.spin_elapsed = super::sequence::SPIN_DURATION
        + super::sequence::STATIC_FADE
        + Duration::from_millis(/*millis*/ 1);
    let delay = animation.render_first_screen(
        available,
        &mut buffer,
        Some(ComposerState::Empty),
        MotionMode::Animated,
    );
    assert_eq!(delay, None);
    assert!(!animation.replaying);
    assert!(braille_cells(&buffer) > 100);
}
