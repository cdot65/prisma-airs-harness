use super::Event;
use super::Mode;
use super::Report;
use super::VIEW_ID;
use super::display;
use super::report;
use crate::airs_mcp_manager::Connection;
use crate::app_event::AppEvent;
use crate::bottom_pane::SelectionDescriptionLayout;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionViewParams;
use crate::bottom_pane::popup_consts::standard_popup_hint_line;
use crate::render::renderable::Renderable;
use codex_protocol::ThreadId;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use std::sync::Arc;

fn header(title: String, detail: String) -> Box<dyn Renderable> {
    Box::new(
        Paragraph::new(vec![Line::from(title).bold(), Line::from(detail).dim()])
            .wrap(Wrap { trim: false }),
    )
}

fn action(
    name: &str,
    description: &str,
    event: impl Fn() -> AppEvent + Send + Sync + 'static,
) -> SelectionItem {
    SelectionItem {
        name: name.into(),
        description: Some(description.into()),
        actions: vec![Box::new(move |tx| tx.send(event()))],
        dismiss_on_select: true,
        ..Default::default()
    }
}

pub(crate) fn overview(
    environment: &str,
    report: Result<Report, String>,
    connections: Vec<Connection>,
    thread: Option<ThreadId>,
) -> SelectionViewParams {
    let session = report::Session::new(report::render(report.as_ref().ok()), thread);
    let mut items = vec![action(
        "Refresh diagnostics",
        "Read configuration and health; no inference request.",
        || AppEvent::AirsDoctor(Event::Open(Mode::Inspect)),
    )];
    items.push(action(
        "Verify gateway access",
        "Confirm a minimal inference request; no conversation, files or tools.",
        || AppEvent::AirsDoctor(Event::ConfirmVerify),
    ));
    let mut authentication = "Unknown".to_string();
    match report {
        Ok(mut report) => {
            authentication = display(&report.authentication);
            if authentication == "Company SSO" {
                items.push(action(
                    "Restore company sign-in",
                    "Sign in as the same person; preserve this conversation and draft.",
                    || AppEvent::AirsDoctor(Event::SignIn),
                ));
            } else {
                items.push(SelectionItem { name: "Credential recovery".into(), description: Some("To replace a workspace key, exit and run airs --environment NAME login for the environment shown above.".into()), ..Default::default() });
            }
            if !report
                .checks
                .iter()
                .any(|check| check.name == "gateway_access")
            {
                items.push(SelectionItem {
                    name: "Gateway access · Not verified".into(),
                    description: Some(
                        "A saved credential and health response do not verify inference access."
                            .into(),
                    ),
                    ..Default::default()
                });
            }
            // Recovery failures and access checks should be visible before
            // installation details, especially in a short terminal viewport.
            report.checks.sort_by_key(|check| {
                (
                    check.passed,
                    match check.name.as_str() {
                        "gateway_access" => 0,
                        "credential_service"
                        | "credential_cleanup"
                        | "credential_configuration" => 1,
                        "configuration" | "gateway_health" => 2,
                        _ => 3,
                    },
                )
            });
            for check in report.checks {
                let status = if check.passed {
                    "OK"
                } else {
                    "Needs attention"
                };
                items.push(SelectionItem {
                    name: format!("{} · {status}", display(&check.name.replace('_', " "))),
                    description: Some(display(&check.detail)),
                    ..Default::default()
                });
            }
        }
        Err(message) => items.push(SelectionItem {
            name: "Diagnostics unavailable".into(),
            description: Some(display(&message)),
            ..Default::default()
        }),
    }
    for connection in connections {
        let name = format!("MCP · {}", display(&connection.name));
        let status = display(&connection.status);
        items.push(action(&name, &status, move || {
            AppEvent::AirsMcpManager(crate::airs_mcp_manager::Event::Select(connection.clone()))
        }));
    }
    items.push(report_action(
        "Diagnostic report",
        "Preview, copy or save a report with private identifiers omitted.",
        &session,
        report::Action::Open,
    ));
    items.push(action(
        "Manage MCP connections",
        "Add a gateway server, sign in, reconnect or check discovered tools.",
        || AppEvent::AirsMcpManager(crate::airs_mcp_manager::Event::Open),
    ));
    SelectionViewParams {
        header: Box::new(ReportHeader {
            content: header(
                format!("Connection health · {}", display(environment)),
                format!("{authentication}. No conversation or tool is replayed."),
            ),
            session,
        }),
        description_layout: SelectionDescriptionLayout::StackBelowWhenNarrow {
            min_description_width: 40,
        },
        items,
        view_id: Some(VIEW_ID),
        footer_hint: Some(standard_popup_hint_line()),
        ..Default::default()
    }
}

pub(crate) fn waiting(attempt: u64) -> SelectionViewParams {
    SelectionViewParams {
        header: header(
            "Checking this environment…".into(),
            "Your conversation and draft stay here. Esc cancels.".into(),
        ),
        items: vec![action(
            "Cancel",
            "Stop this check; keep your sign-in and draft.",
            move || AppEvent::AirsDoctor(Event::Cancel(attempt)),
        )],
        on_cancel: Some(Box::new(move |tx| {
            tx.send(AppEvent::AirsDoctor(Event::Cancel(attempt)))
        })),
        view_id: Some(VIEW_ID),
        footer_hint: Some(standard_popup_hint_line()),
        ..Default::default()
    }
}

pub(crate) fn confirm_verify() -> SelectionViewParams {
    SelectionViewParams {
        header: header("Verify gateway access?".into(), "This sends one fixed connectivity message with up to 16 output tokens. No local files, conversation or tools are sent. The request asks the provider not to store the response; gateway logging policy still applies.".into()),
        description_layout: SelectionDescriptionLayout::StackBelowWhenNarrow { min_description_width: 40 },
        items: vec![action("Send connectivity check", "Verify access with this environment's saved credential.", || AppEvent::AirsDoctor(Event::Open(Mode::Verify)))],
        view_id: Some(VIEW_ID), footer_hint: Some(standard_popup_hint_line()),
        ..Default::default()
    }
}

// The header owns the capability lifetime, so closing, replacing or dropping
// this view invalidates queued report actions without a global report cache.
struct ReportHeader {
    content: Box<dyn Renderable>,
    session: Arc<report::Session>,
}

impl Drop for ReportHeader {
    fn drop(&mut self) {
        self.session.invalidate();
    }
}

impl Renderable for ReportHeader {
    fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        self.content.render(area, buf);
    }
    fn desired_height(&self, width: u16) -> u16 {
        self.content.desired_height(width)
    }
    fn render_scrolled(
        &self,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
        offset: u16,
    ) -> bool {
        self.content.render_scrolled(area, buf, offset)
    }
}

fn report_action(
    name: &str,
    description: &str,
    session: &Arc<report::Session>,
    action: report::Action,
) -> SelectionItem {
    let session = Arc::clone(session);
    SelectionItem {
        name: name.into(),
        description: Some(description.into()),
        actions: vec![Box::new(move |tx| {
            tx.send(AppEvent::AirsDoctor(Event::Report {
                session: Arc::clone(&session),
                action,
            }))
        })],
        dismiss_on_select: false,
        ..Default::default()
    }
}

pub(crate) fn report_actions(
    text: Arc<str>,
    thread: Option<ThreadId>,
    status: &str,
) -> SelectionViewParams {
    let session = report::Session::new(text, thread);
    let items = vec![
        report_action(
            "Preview report",
            "Read the exact report; Esc returns here.",
            &session,
            report::Action::Preview,
        ),
        report_action(
            "Copy report",
            "Send plain text to your clipboard; terminal support may be required.",
            &session,
            report::Action::Copy,
        ),
        report_action(
            "Save local report",
            "Create a private text file in this environment's directory.",
            &session,
            report::Action::Save,
        ),
        report_action(
            "Close",
            "Return to your conversation; preserve your draft.",
            &session,
            report::Action::Close,
        ),
    ];
    SelectionViewParams {
        header: Box::new(ReportHeader {
            content: header(
                "Diagnostic report".into(),
                format!(
                    "Version, platform, authentication method and check results. Names, addresses, paths and raw details are omitted. Nothing is uploaded. {status}"
                ),
            ),
            session,
        }),
        description_layout: SelectionDescriptionLayout::StackBelowWhenNarrow {
            min_description_width: 40,
        },
        items,
        view_id: Some(VIEW_ID),
        footer_hint: Some(standard_popup_hint_line()),
        ..Default::default()
    }
}
