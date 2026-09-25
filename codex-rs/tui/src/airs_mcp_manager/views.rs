//! AIRS connection menus; these actions are UI events, never model instructions.
use super::Connection;
use super::Event;
use super::Operation;
use super::Progress;
use super::VIEW_ID;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::popup_consts::standard_popup_hint_line;

fn action(
    name: &str,
    description: &str,
    event: impl Fn() -> Event + Send + Sync + 'static,
) -> SelectionItem {
    SelectionItem {
        name: name.into(),
        description: Some(description.into()),
        actions: vec![Box::new(move |tx| {
            tx.send(AppEvent::AirsMcpManager(event()))
        })],
        dismiss_on_select: true,
        ..Default::default()
    }
}

pub(crate) fn overview(environment: &str, connections: Vec<Connection>) -> SelectionViewParams {
    let mut items: Vec<_> = connections
        .into_iter()
        .map(|connection| {
            action(
                &connection.name.clone(),
                &connection.status.clone(),
                move || Event::Select(connection.clone()),
            )
        })
        .collect();
    items.push(action(
        "Add gateway MCP server",
        "Use the gateway-facing MCP URL, not the upstream server or inference URL.",
        || Event::AddName,
    ));
    items.push(action(
        "Refresh connections",
        "Check the gateway connection and available tools.",
        || Event::Open,
    ));
    SelectionViewParams {
        title: Some(format!("MCP connections · {environment}")),
        subtitle: Some("Saved in this environment. MCP sign-in is separate from inference.".into()),
        items,
        view_id: Some(VIEW_ID),
        footer_hint: Some(standard_popup_hint_line()),
        ..Default::default()
    }
}

pub(crate) fn connection(connection: Connection) -> SelectionViewParams {
    let mut items = Vec::new();
    if connection.can_login {
        let name = connection.name.clone();
        items.push(action(
            "Sign in",
            "Company SSO in your browser; callback entry works over SSH.",
            move || Event::Run(Operation::Login(name.clone())),
        ));
        let name = connection.name.clone();
        items.push(action(
            "Sign out",
            "Clear saved MCP OAuth credentials; keep inference sign-in.",
            move || Event::Confirm(Operation::Logout(name.clone())),
        ));
    }
    let name = connection.name.clone();
    items.push(action(
        "Reconnect and verify",
        "List gateway tools; a real tool call verifies its authorization.",
        move || Event::Run(Operation::Verify(name.clone())),
    ));
    let name = connection.name.clone();
    items.push(action(
        "Remove connection",
        "Remove this environment's configuration. Sign out first to clear credentials.",
        move || Event::Confirm(Operation::Remove(name.clone())),
    ));
    items.push(action("Back", "Return to MCP connections.", || Event::Open));
    SelectionViewParams {
        title: Some(connection.name),
        subtitle: Some(format!("{} · {}", connection.status, connection.url)),
        items,
        view_id: Some(VIEW_ID),
        footer_hint: Some(standard_popup_hint_line()),
        ..Default::default()
    }
}

pub(crate) fn confirm(operation: Operation) -> SelectionViewParams {
    let title = match &operation {
        Operation::Remove(name) => format!("Remove {name}?"),
        Operation::Logout(name) => format!("Sign out of {name}?"),
        _ => "Confirm MCP change".into(),
    };
    SelectionViewParams {
        title: Some(title),
        subtitle: Some(
            "Inference sign-in and history stay saved. Continue in a new conversation.".into(),
        ),
        items: vec![
            action("Cancel", "Keep this connection.", || Event::Open),
            action(
                "Confirm",
                "Apply this change to the selected environment.",
                move || Event::Run(operation.clone()),
            ),
        ],
        view_id: Some(VIEW_ID),
        footer_hint: Some(standard_popup_hint_line()),
        ..Default::default()
    }
}

pub(crate) fn waiting(attempt: u64) -> SelectionViewParams {
    SelectionViewParams {
        title: Some("Checking gateway MCP…".into()),
        subtitle: Some("Your conversation and draft stay here.".into()),
        items: vec![action("Cancel", "Stop this operation.", move || {
            Event::Cancel(attempt)
        })],
        on_cancel: Some(Box::new(move |tx| {
            tx.send(AppEvent::AirsMcpManager(Event::Cancel(attempt)))
        })),
        view_id: Some(VIEW_ID),
        footer_hint: Some(standard_popup_hint_line()),
        ..Default::default()
    }
}

pub(crate) fn continue_after_change(thread: codex_protocol::ThreadId) -> SelectionViewParams {
    SelectionViewParams {
        title: Some("MCP connection updated".into()),
        subtitle: Some(
            "Start a new conversation with your draft. History is saved; nothing replays.".into(),
        ),
        items: vec![
            SelectionItem {
                name: "Start new conversation".into(),
                description: Some("Review your preserved draft before sending.".into()),
                actions: vec![Box::new(move |tx| {
                    tx.send(AppEvent::AirsMcpNewConversation { thread_id: thread })
                })],
                dismiss_on_select: true,
                ..Default::default()
            },
            action("Manage connections", "Return to /mcp.", || Event::Open),
        ],
        view_id: Some(VIEW_ID),
        footer_hint: Some(standard_popup_hint_line()),
        ..Default::default()
    }
}

pub(crate) fn progress(attempt: u64, progress: Progress) -> SelectionViewParams {
    let (title, subtitle) = match progress {
        Progress::ExchangingCode => (
            "Completing MCP sign-in…",
            "Browser response received. Verifying authorization with the gateway.",
        ),
        Progress::SavingCredential => (
            "Saving MCP credential…",
            "Gateway authorization completed. Waiting for credential storage.",
        ),
        Progress::DiscoveringTools => (
            "Connecting and discovering MCP tools…",
            "Checking the connection. Tool discovery does not execute a tool.",
        ),
    };
    SelectionViewParams {
        title: Some(title.into()),
        subtitle: Some(subtitle.into()),
        ..waiting(attempt)
    }
}
