use super::Event;
use super::Kind;
use super::Proposal;
use super::Selection;
use super::VIEW_ID;
use crate::airs_doctor::display;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionDescriptionLayout;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::popup_consts::standard_popup_hint_line;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

fn action(
    name: &str,
    description: &str,
    event: impl Fn() -> Event + Send + Sync + 'static,
) -> SelectionItem {
    SelectionItem {
        name: name.into(),
        description: Some(description.into()),
        actions: vec![Box::new(move |tx| tx.send(AppEvent::AirsRouting(event())))],
        dismiss_on_select: true,
        ..Default::default()
    }
}
fn view(title: &str, detail: String, items: Vec<SelectionItem>) -> SelectionViewParams {
    SelectionViewParams {
        header: Box::new(
            Paragraph::new(
                std::iter::once(Line::from(title.to_owned()).bold())
                    .chain(detail.lines().map(|line| Line::from(line.to_owned()).dim()))
                    .collect::<Vec<_>>(),
            )
            .wrap(Wrap { trim: false }),
        ),
        items,
        view_id: Some(VIEW_ID),
        description_layout: SelectionDescriptionLayout::StackBelowWhenNarrow {
            min_description_width: 40,
        },
        footer_hint: Some(standard_popup_hint_line()),
        ..Default::default()
    }
}
pub(crate) fn menu(kind: Kind, selection: &Selection, models: &[String]) -> SelectionViewParams {
    let (title, enter, placeholder, reset) = match kind {
        Kind::Config => (
            "AI Gateway configuration",
            "Enter saved config ID",
            "Use a saved gateway config ID, not inline JSON.",
            "Use gateway default",
        ),
        Kind::Model => (
            "AI Gateway model",
            "Enter model override",
            "Use @integration/model; the gateway must permit the request.",
            "Follow config routing",
        ),
    };
    let mut items = vec![
        action(enter, placeholder, move || Event::Input(kind)),
        action(
            reset,
            "Verify the proposed routing before using it.",
            move || Event::Propose(kind, None),
        ),
    ];
    if kind == Kind::Model {
        for model in models {
            let model = model.clone();
            items.push(action(
                &display(&model),
                "Configured route; verify gateway access before use.",
                move || Event::Propose(Kind::Model, Some(model.clone())),
            ));
        }
    }
    view(
        title,
        format!(
            "{}\nRequested routing for this conversation. The gateway controls the effective model. Opening this menu sends no request.",
            display(&selection.describe())
        ),
        items,
    )
}

pub(crate) fn confirm(proposal: Proposal) -> SelectionViewParams {
    view(
        "Verify and use this routing?",
        format!(
            "{}\nOne fixed connectivity message, up to 16 output tokens. No conversation, files or tools. May incur a charge; store=false is requested and gateway logging still applies. A saved config may override the requested model.",
            display(&proposal.target.describe())
        ),
        vec![action(
            "Verify and use",
            "Apply only after the gateway accepts this request.",
            move || Event::Verify(proposal.clone()),
        )],
    )
}
pub(crate) fn waiting(attempt: u64) -> SelectionViewParams {
    let mut result = view(
        "Verifying gateway routing…",
        "Your current routing and draft remain. Esc cancels before application.".into(),
        vec![action("Cancel", "Keep the current routing.", move || {
            Event::Cancel(attempt)
        })],
    );
    result.on_cancel = Some(Box::new(move |tx| {
        tx.send(AppEvent::AirsRouting(Event::Cancel(attempt)))
    }));
    result
}
pub(crate) fn applying() -> SelectionViewParams {
    let mut result = view(
        "Applying gateway routing…",
        "Waiting for the conversation to confirm the change. Queued input remains paused.".into(),
        vec![SelectionItem {
            name: "Waiting for confirmation".into(),
            is_disabled: true,
            ..Default::default()
        }],
    );
    result.footer_hint = Some(Line::from("Waiting for session confirmation").dim());
    result
}
