use super::Event;
use super::VIEW_ID;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionDescriptionLayout;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::popup_consts::standard_popup_hint_line;
use crate::render::renderable::Renderable;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

fn action(name: &str, description: &str, event: fn() -> Event) -> SelectionItem {
    SelectionItem {
        name: name.into(),
        description: Some(description.into()),
        actions: vec![Box::new(move |tx| tx.send(AppEvent::AirsTypeSafe(event())))],
        dismiss_on_select: true,
        ..Default::default()
    }
}

pub(crate) fn overview(environment: &str, status: &str) -> SelectionViewParams {
    SelectionViewParams {
        view_id: Some(VIEW_ID),
        header: header(
            format!(
                "TypeSafe Jev · {}",
                crate::airs_doctor::display(environment)
            ),
            format!(
                "Optional red-team judge. {}",
                crate::airs_doctor::display(status)
            ),
        ),
        description_layout: SelectionDescriptionLayout::StackBelowWhenNarrow {
            min_description_width: 32,
        },
        footer_hint: Some(standard_popup_hint_line()),
        items: vec![
            action(
                "Save or replace API key",
                "Hidden entry; stored only for this environment. Judging sends scan data to TypeSafe and may incur charges.",
                || Event::Set,
            ),
            action(
                "Refresh status",
                "Inspect configuration without sending a paid judgment request.",
                || Event::Open,
            ),
            action(
                "Remove saved API key",
                "Remove this environment's saved key. An inherited TYPESAFE_API_KEY still takes precedence.",
                || Event::ConfirmClear,
            ),
            SelectionItem {
                name: "Close".into(),
                dismiss_on_select: true,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

pub(crate) fn confirm_clear() -> SelectionViewParams {
    SelectionViewParams {
        view_id: Some(VIEW_ID),
        header: header("Remove this environment's TypeSafe key?".into(), "Inference and MCP sign-in are unchanged. An inherited TYPESAFE_API_KEY remains active.".into()),
        footer_hint: Some(standard_popup_hint_line()),
        items: vec![
            action("Keep saved key", "Return to TypeSafe settings.", || Event::Open),
            action("Remove saved key", "Delete the saved key from the native credential store.", || Event::Clear),
        ],
        ..Default::default()
    }
}

fn header(title: String, detail: String) -> Box<dyn Renderable> {
    Box::new(
        Paragraph::new(vec![Line::from(title).bold(), Line::from(detail)])
            .wrap(Wrap { trim: false }),
    )
}
