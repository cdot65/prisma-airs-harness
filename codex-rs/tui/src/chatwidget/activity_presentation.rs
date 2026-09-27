//! Render live AIRS activity in compact or detailed form without changing source cells.
use super::*;
impl ChatWidget {
    pub(crate) fn active_cell_hyperlink_lines_with(
        &self,
        width: u16,
        render: impl Fn(&dyn HistoryCell, u16) -> Vec<HyperlinkLine>,
    ) -> Option<Vec<HyperlinkLine>> {
        let mut lines = Vec::new();
        if let Some(cell) = self.transcript.active_cell.as_ref() {
            lines.extend(render(cell.as_ref(), width));
        }
        if let Some(token_activity_cell) = self.pending_token_activity_output() {
            let token_activity_lines = render(token_activity_cell, width);
            if !token_activity_lines.is_empty() && !lines.is_empty() {
                lines.push(HyperlinkLine::from(""));
            }
            lines.extend(token_activity_lines);
        }
        if let Some(rate_limit_reset_hint) = self.pending_rate_limit_reset_hint() {
            let hint_lines = render(rate_limit_reset_hint, width);
            if !hint_lines.is_empty() && !lines.is_empty() {
                lines.push(HyperlinkLine::from(""));
            }
            lines.extend(hint_lines);
        }
        (!lines.is_empty()).then_some(lines)
    }
}
