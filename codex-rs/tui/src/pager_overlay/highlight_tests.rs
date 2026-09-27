//! Backtrack selection preserves unrelated layout caches and the complete rendered viewport.

use super::*;
use crate::keymap::RuntimeKeymap;
use pretty_assertions::assert_eq;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[derive(Debug)]
struct MeasuredCell {
    measurements: AtomicUsize,
}

impl HistoryCell for MeasuredCell {
    fn display_lines(&self, _width: u16) -> Vec<Line<'static>> {
        self.measurements.fetch_add(1, Ordering::Relaxed);
        vec!["history".into()]
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        vec!["history".into()]
    }
}

#[test]
fn moving_highlight_preserves_unaffected_height_caches() {
    let cells: Vec<_> = (0..32)
        .map(|_| {
            Arc::new(MeasuredCell {
                measurements: AtomicUsize::new(0),
            })
        })
        .collect();
    let mut overlay = TranscriptOverlay::new(
        cells
            .iter()
            .map(|cell| cell.clone() as Arc<dyn HistoryCell>)
            .collect(),
        RuntimeKeymap::defaults().pager,
    );
    let mut area = Rect::new(
        /*x*/ 0, /*y*/ 0, /*width*/ 40, /*height*/ 12,
    );
    overlay.render(area, &mut Buffer::empty(area));

    for selection in [Some(30), Some(28), Some(28), None] {
        overlay.set_highlight_cell(selection);
        overlay.render(area, &mut Buffer::empty(area));
    }

    let measurements = || {
        cells
            .iter()
            .map(|cell| cell.measurements.load(Ordering::Relaxed))
            .collect::<Vec<_>>()
    };
    let expected = measurements();
    assert!(expected.iter().all(|count| *count <= 1));
    assert!(expected.contains(&0));
    overlay.render(area, &mut Buffer::empty(area));
    assert_eq!(measurements(), expected);

    // Width changes invalidate visible layouts without formatting offscreen history.
    area.width = 24;
    overlay.render(area, &mut Buffer::empty(area));
    let resized = measurements();
    assert!(
        resized
            .iter()
            .zip(&expected)
            .any(|(now, before)| now > before)
    );
    assert!(resized.iter().all(|count| *count <= 2));
    assert!(resized.contains(&0));
}
