//! Ordered calls and transcript details for adjacent activity groups.
//!
//! Call positions stay fixed as in-flight calls complete. Prepending older calls shifts the
//! shared detail positions without recreating live clocks or discarding raw terminal input.

use super::ActivityDetails;
use super::HistoryCell;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct ActivityGroup<T> {
    pub(crate) calls: Vec<T>,
    pub(crate) details: ActivityDetails,
}

impl<T> ActivityGroup<T> {
    pub(crate) fn new(calls: Vec<T>) -> Self {
        Self {
            calls,
            details: ActivityDetails::default(),
        }
    }

    pub(crate) fn push_detail(&mut self, cell: Arc<dyn HistoryCell>) {
        self.details.push(self.calls.len(), cell);
    }

    #[allow(
        dead_code,
        reason = "Used by the following compact command integration stage."
    )]
    pub(crate) fn prepend(&mut self, older: Self) {
        self.details.prepend(older.details, older.calls.len());
        self.calls.splice(0..0, older.calls);
    }

    // Compatibility with existing command rendering until its source-aware conversion lands.
    pub(crate) fn push_reasoning(&mut self, cell: Box<dyn HistoryCell>) {
        self.push_detail(cell.into());
    }

    pub(crate) fn transcript_lines(
        &self,
        width: u16,
        mode: super::HistoryRenderMode,
        mut render_call: impl FnMut(usize, &T, &mut Vec<ratatui::text::Line<'static>>),
    ) -> Vec<ratatui::text::Line<'static>> {
        let mut lines = Vec::new();
        for (index, call) in self.calls.iter().enumerate() {
            render_call(index, call, &mut lines);
            lines.extend(crate::terminal_hyperlinks::visible_lines(
                self.details.lines_after(index + 1, width, mode),
            ));
        }
        lines
    }
}

#[cfg(test)]
#[path = "activity_group_tests.rs"]
mod tests;
