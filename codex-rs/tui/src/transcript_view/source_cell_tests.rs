//! Logical copy and display wrapping for reasoning, plans, and questions.

use super::*;
use crate::history_cell::HistoryCell;
use crate::history_cell::ReasoningSummaryCell;
use crate::history_cell::RequestUserInputResultCell;
use codex_app_server_protocol::ToolRequestUserInputAnswer;
use codex_app_server_protocol::ToolRequestUserInputQuestion;
use codex_protocol::plan_tool::PlanItemArg;
use codex_protocol::plan_tool::StepStatus;
use codex_protocol::plan_tool::UpdatePlanArgs;
use pretty_assertions::assert_eq;
use ratatui::style::Stylize;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn reasoning_copy_joins_soft_wraps_and_retains_the_existing_rendering() {
    let content = "Investigate the failing condition and preserve the current user experience across terminal resize.";
    let cell = ReasoningSummaryCell::new(
        "Thinking".to_owned(),
        content.to_owned(),
        Path::new("/project"),
        /*transcript_only*/ true,
    );
    let width = 36;
    let lines = cell.transcript_hyperlink_lines(width);
    let layout = TextLayout::new(lines.clone(), width);
    assert_eq!(layout.text(), content);
    assert_eq!(layout.rewrap(/*width*/ 19).text(), content);
    assert_eq!(cell.display_hyperlink_lines(width), Vec::new());

    assert_layout_paints_like_lines(lines, width);
}

#[test]
fn plan_copy_retains_checkboxes_but_omits_wrapping_gutters() {
    let note = "Preserve the existing behavior while simplifying the transcript renderer.";
    let step = "Replace the terminal scrollback path with an owned transcript viewport.";
    let cell = crate::history_cell::new_plan_update(UpdatePlanArgs {
        explanation: Some(note.to_owned()),
        plan: vec![PlanItemArg {
            step: step.to_owned(),
            status: StepStatus::Completed,
        }],
    });
    let width = 38;
    let lines = cell.display_hyperlink_lines(width);
    let layout = TextLayout::new(lines.clone(), width);
    let expected_text = format!("• Updated Plan\n{note}\n✔ {step}");
    assert_eq!(layout.text(), expected_text);
    assert_eq!(layout.rewrap(/*width*/ 20).text(), expected_text);

    assert_layout_paints_like_lines(lines, width);
}

#[test]
fn question_copy_retains_answer_labels_masks_secrets_and_keeps_unanswered_suffixes() {
    let question = "Which behavior should remain available when this long conversation is resumed?";
    let answer = "Preserve all of the existing selection and navigation controls across resize.";
    let secret_question = ToolRequestUserInputQuestion {
        id: "secret".to_owned(),
        header: "Token".to_owned(),
        question: "Which token?".to_owned(),
        is_other: false,
        is_secret: true,
        options: None,
    };
    let cell = RequestUserInputResultCell {
        questions: vec![
            ToolRequestUserInputQuestion {
                id: "behavior".to_owned(),
                header: "Behavior".to_owned(),
                question: question.to_owned(),
                is_other: false,
                is_secret: false,
                options: None,
            },
            secret_question.clone(),
            ToolRequestUserInputQuestion {
                id: "pending".to_owned(),
                question: "Where next?".to_owned(),
                is_secret: false,
                ..secret_question
            },
        ],
        answers: HashMap::from([
            (
                "behavior".to_owned(),
                ToolRequestUserInputAnswer {
                    answers: vec![answer.to_owned()],
                },
            ),
            (
                "secret".to_owned(),
                ToolRequestUserInputAnswer {
                    answers: vec!["never expose this credential".to_owned()],
                },
            ),
        ]),
        interrupted: true,
    };
    let width = 48;
    let lines = cell.display_hyperlink_lines(width);
    let layout = TextLayout::new(lines.clone(), width);
    let expected = format!(
        "• Questions 2/3 answered (interrupted)\n• {question}\nanswer: {answer}\n• Which token?\nanswer: ••••••\n• Where next? (unanswered)\n↳ interrupted with 1 unanswered"
    );
    assert_eq!(layout.text(), expected);
    assert_eq!(layout.rewrap(/*width*/ 24).text(), expected);

    assert_layout_paints_like_lines(lines, width);
}

#[test]
fn resize_restores_the_exact_styles_of_whitespace_omitted_by_wrapping() {
    let line = HyperlinkLine::new(Line::from(vec![
        "alpha".bold(),
        "  ".on_red(),
        "beta".italic(),
        " ".dim(),
        "gamma".underlined(),
    ]));
    let wrapped = crate::terminal_hyperlinks::adaptive_wrap_hyperlink_lines(
        std::slice::from_ref(&line),
        RtOptions::new(/*width*/ 6),
    );
    let layout = TextLayout::new(wrapped, /*width*/ 6).rewrap(/*width*/ 24);
    assert_eq!(layout.text(), "alpha  beta gamma");
    let area = Rect::new(
        /*x*/ 0, /*y*/ 0, /*width*/ 24, /*height*/ 2,
    );
    let mut expected = Buffer::empty(area);
    HyperlinkParagraph::new(&[line], Style::default()).render(area, &mut expected);
    let mut actual = Buffer::empty(area);
    layout.render(area, &mut actual, /*start_row*/ 0);
    assert_eq!(actual, expected);
}

fn assert_layout_paints_like_lines(lines: Vec<HyperlinkLine>, width: u16) {
    let area = Rect::new(/*x*/ 0, /*y*/ 0, width, /*height*/ 30);
    let mut expected = Buffer::empty(area);
    HyperlinkParagraph::new(&lines, Style::default()).render(area, &mut expected);
    let mut actual = Buffer::empty(area);
    TextLayout::new(lines, width).render(area, &mut actual, /*start_row*/ 0);
    assert_eq!(actual, expected);
}

#[test]
fn long_reasoning_source_height_matches_complete_guttered_rendering() {
    let summary = "example.test/api/v1/projects/alpha-team/releases/2026-02-17/builds/1234567890/artifacts/reports/performance/summary/detail/with/a/very/long/path/that/keeps/going";
    let cell = ReasoningSummaryCell::new(
        "Thinking".to_owned(),
        summary.to_owned(),
        Path::new("/project"),
        /*transcript_only*/ true,
    );
    let layout = TextLayout::new(cell.transcript_hyperlink_lines(24), /*width*/ 24);
    assert_eq!(layout.text(), summary);
    let area = Rect::new(0, 0, 24, layout.row_count() as u16);
    let mut buffer = Buffer::empty(area);
    layout.render(area, &mut buffer, /*start_row*/ 0);
    insta::assert_snapshot!("reasoning_source_height", format!("{buffer:?}"));
}
