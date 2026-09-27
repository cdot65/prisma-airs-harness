//! Cache reuse, bounded lifetime, and invalidation across frames.

use super::*;
use pretty_assertions::assert_eq;
use ratatui::text::Line;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[derive(Debug, Default)]
struct Cell {
    renders: AtomicUsize,
    tick: AtomicUsize,
    mutable: bool,
}

impl HistoryCell for Cell {
    fn display_lines(&self, _width: u16) -> Vec<Line<'static>> {
        self.renders.fetch_add(/*val*/ 1, Ordering::Relaxed);
        vec![format!("tick {}", self.tick.load(Ordering::Relaxed)).into()]
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        vec![]
    }

    fn has_stable_transcript_height(&self) -> bool {
        !self.mutable
    }

    fn transcript_animation_tick(&self) -> Option<u64> {
        Some(self.tick.load(Ordering::Relaxed) as u64)
    }
}

#[test]
fn layouts_refresh_for_width_animation_and_mutable_frames() {
    for mutable in [false, true] {
        let cell = Arc::new(Cell {
            mutable,
            ..Cell::default()
        });
        let history = cell.clone() as Arc<dyn HistoryCell>;
        let mut cache = LayoutCache::default();
        let get = |cache: &mut LayoutCache, width| {
            cache.get(
                &history,
                width,
                CellPresentation { separated: false },
                || TextLayout::new(history.transcript_hyperlink_lines(width), width),
            )
        };
        cache.begin_frame();
        get(&mut cache, /*width*/ 20);
        get(&mut cache, /*width*/ 20);
        assert_eq!(cell.renders.load(Ordering::Relaxed), 1);
        cache.begin_frame();
        get(&mut cache, /*width*/ 20);
        assert_eq!(
            cell.renders.load(Ordering::Relaxed),
            1 + usize::from(mutable)
        );
        cell.tick.store(/*val*/ 2, Ordering::Relaxed);
        cache.begin_frame();
        let next = get(&mut cache, /*width*/ 20);
        assert_eq!(next.text(), "tick 2");
        get(&mut cache, /*width*/ 10);
        assert_eq!(
            cell.renders.load(Ordering::Relaxed),
            3 + usize::from(mutable)
        );
        crate::terminal_palette::with_test_default_colors(
            crate::terminal_probe::DefaultColors {
                fg: (12, 34, 56),
                bg: (65, 43, 21),
            },
            || {
                cache.begin_frame();
                get(&mut cache, /*width*/ 10);
                assert_eq!(
                    cell.renders.load(Ordering::Relaxed),
                    4 + usize::from(mutable)
                );
            },
        );
    }
}

#[test]
fn recent_entries_are_reused_and_old_entries_are_evicted() {
    let cells: Vec<_> = (0..100).map(|_| Arc::new(Cell::default())).collect();
    let mut cache = LayoutCache::default();
    cache.begin_frame();
    for cell in &cells {
        let history = cell.clone() as Arc<dyn HistoryCell>;
        cache.get(
            &history,
            /*width*/ 20,
            CellPresentation { separated: false },
            || {
                TextLayout::new(
                    history.transcript_hyperlink_lines(/*width*/ 20),
                    /*width*/ 20,
                )
            },
        );
    }
    for index in [99, 98, 0] {
        let history = cells[index].clone() as Arc<dyn HistoryCell>;
        cache.get(
            &history,
            /*width*/ 20,
            CellPresentation { separated: false },
            || {
                TextLayout::new(
                    history.transcript_hyperlink_lines(/*width*/ 20),
                    /*width*/ 20,
                )
            },
        );
    }
    assert_eq!(
        [99, 98, 0].map(|index| cells[index].renders.load(Ordering::Relaxed)),
        [1, 1, 2]
    );
}

#[test]
fn byte_budget_evicts_old_entries_but_retains_one_oversized_visible_entry() {
    use crate::history_cell::PlainHistoryCell;

    let first: Arc<dyn HistoryCell> = Arc::new(PlainHistoryCell::new(vec![
        "x".repeat(5 * 1024 * 1024).into(),
    ]));
    let second: Arc<dyn HistoryCell> = Arc::new(PlainHistoryCell::new(vec![
        "y".repeat(5 * 1024 * 1024).into(),
    ]));
    let oversized: Arc<dyn HistoryCell> = Arc::new(PlainHistoryCell::new(vec![
        "z".repeat(MAX_CACHED_TEXT_BYTES + 1).into(),
    ]));
    let mut cache = LayoutCache::default();
    cache.begin_frame();
    cache.get(
        &first,
        /*width*/ 120,
        CellPresentation { separated: false },
        || TextLayout::new(first.transcript_hyperlink_lines(120), 120),
    );
    let recent = cache.get(
        &second,
        /*width*/ 120,
        CellPresentation { separated: false },
        || TextLayout::new(second.transcript_hyperlink_lines(120), 120),
    );
    assert_eq!(cache.entries.len(), 1);
    assert!(Arc::ptr_eq(&cache.entries[0].layout, &recent));
    let large = cache.get(
        &oversized,
        /*width*/ 120,
        CellPresentation { separated: false },
        || TextLayout::new(oversized.transcript_hyperlink_lines(120), 120),
    );
    assert_eq!(cache.entries.len(), 1);
    assert!(large.text().len() > MAX_CACHED_TEXT_BYTES);
    cache.begin_frame();
    assert!(Arc::ptr_eq(
        &large,
        &cache.get(
            &oversized,
            /*width*/ 120,
            CellPresentation { separated: false },
            || TextLayout::new(oversized.transcript_hyperlink_lines(120), 120)
        )
    ));
    cache.get(
        &second,
        /*width*/ 120,
        CellPresentation { separated: false },
        || TextLayout::new(second.transcript_hyperlink_lines(120), 120),
    );
    assert_eq!(cache.entries.len(), 1);
    assert!(cache.entries[0].layout.text().len() <= MAX_CACHED_TEXT_BYTES);
}

#[test]
fn cached_layout_does_not_keep_source_cell_alive_and_separator_changes_refresh_it() {
    let concrete = Arc::new(Cell::default());
    let cell = concrete.clone() as Arc<dyn HistoryCell>;
    let weak = Arc::downgrade(&cell);
    let mut cache = LayoutCache::default();
    cache.begin_frame();
    let plain = cache.get(
        &cell,
        /*width*/ 20,
        CellPresentation { separated: false },
        || TextLayout::new(cell.transcript_hyperlink_lines(20), 20),
    );
    let separated = cache.get(
        &cell,
        /*width*/ 20,
        CellPresentation { separated: true },
        || TextLayout::new(cell.transcript_hyperlink_lines(20), 20),
    );
    assert_eq!(separated.row_count(), plain.row_count() + 1);
    assert_eq!(concrete.renders.load(Ordering::Relaxed), 2);
    drop(concrete);
    drop(cell);
    assert!(weak.upgrade().is_none());
    assert_eq!(cache.entries.len(), 1);
}
