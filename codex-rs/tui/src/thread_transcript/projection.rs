//! Reconstruct exploration groups without crossing visible items or turn boundaries.
//! Gateway MCP calls are ordinary independent calls; no upstream computer mode is inferred.

use super::*;
use crate::exec_cell::ExecCell;

pub(super) fn project_items(
    cwd: &AbsolutePathBuf,
    items: impl IntoIterator<Item = ThreadItem>,
    visibility: RawReasoningVisibility,
    context: Option<InlineVisualizationContext>,
) -> TranscriptCells {
    let mut cells = Vec::new();
    let mut pending: Option<ExecCell> = None;
    for item in items {
        if matches!(item, ThreadItem::Reasoning { .. })
            && let Some(group) = &mut pending
        {
            for cell in item_to_cells(item, cwd, visibility, context.clone()) {
                group.group.push_detail(cell);
            }
            continue;
        }
        if let Some(cell) = tools::historical_tool_fallback(&item) {
            flush(&mut pending, &mut cells);
            cells.push(Arc::new(cell));
            continue;
        }
        match item {
            item @ ThreadItem::CommandExecution { .. } => {
                if let Some(command) = tools::CommandHistory::from_item(item) {
                    let newer = command.into_cell();
                    if let Some(group) = &mut pending {
                        if let Err(newer) = group.append_completed(newer) {
                            flush(&mut pending, &mut cells);
                            pending = Some(newer);
                        }
                    } else {
                        pending = Some(newer);
                    }
                    if pending.as_ref().is_some_and(ExecCell::should_flush) {
                        flush(&mut pending, &mut cells);
                    }
                }
            }
            item => {
                let projected = item_to_cells(item, cwd, visibility, context.clone());
                if !projected.is_empty() {
                    flush(&mut pending, &mut cells);
                    cells.extend(projected);
                }
            }
        }
    }
    flush(&mut pending, &mut cells);
    cells
}

fn flush(pending: &mut Option<ExecCell>, cells: &mut TranscriptCells) {
    if let Some(group) = pending.take() {
        cells.push(Arc::new(group));
    }
}
