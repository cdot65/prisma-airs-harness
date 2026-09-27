//! Choose the next launch's transcript renderer without changing this session.

use super::ChatWidget;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionDescriptionLayout;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::keymap::ListAction;
use ratatui::text::Line;

impl ChatWidget {
    pub(crate) fn show_tui_mode_picker(&mut self) {
        let items = [
            (false, "Scrollback", "Use your terminal's scrollback"),
            (true, "Fullscreen", "Scroll within AIRS’s fullscreen view"),
        ]
        .into_iter()
        .map(|(enabled, name, description)| SelectionItem {
            name: name.into(),
            description: Some(description.into()),
            is_current: enabled == self.local_settings.tui.fullscreen_transcript,
            actions: vec![Box::new(move |tx| {
                tx.send(AppEvent::FullscreenTranscriptSelected { enabled });
            })],
            dismiss_on_select: true,
            require_explicit_confirmation: true,
            ..Default::default()
        })
        .collect();
        let keymap = self.bottom_pane.list_keymap();
        let mut hints = Vec::new();
        for (action, label) in [(ListAction::Accept, "select"), (ListAction::Cancel, "back")] {
            if let Some(key) = keymap.primary_hint(action) {
                if !hints.is_empty() {
                    hints.push(" · ".into());
                }
                hints.push(key.into());
                hints.push(format!(" {label}").into());
            }
        }
        self.show_selection_view(SelectionViewParams {
            title: Some("TUI mode for next launch".into()),
            description_layout: SelectionDescriptionLayout::StackBelowWhenNarrow {
                min_description_width: 25,
            },
            footer_note: Some("Restart to apply. Launch overrides still apply.".into()),
            footer_hint: Some(Line::from(hints)),
            items,
            ..SelectionViewParams::default()
        });
    }
}
