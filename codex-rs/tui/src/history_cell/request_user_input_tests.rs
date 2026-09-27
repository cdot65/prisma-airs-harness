//! Result rendering preserves answer styles and wrapped unanswered questions.

use super::*;
use codex_app_server_protocol::ToolRequestUserInputOption;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

#[test]
fn completed_and_interrupted_results() {
    let question = ToolRequestUserInputQuestion {
        id: "choice".into(),
        header: "Approach".into(),
        question: "Which approach should we use for the next step?".into(),
        is_other: true,
        is_secret: false,
        options: Some(vec![ToolRequestUserInputOption {
            label: "Keep it small".into(),
            description: "Use the existing helper".into(),
        }]),
    };
    let mut cell = RequestUserInputResultCell {
        questions: vec![question],
        answers: HashMap::from([(
            "choice".into(),
            ToolRequestUserInputAnswer {
                answers: vec![
                    "Keep it small".into(),
                    "user_note: Preserve the behavior".into(),
                ],
            },
        )]),
        interrupted: false,
    };
    crate::terminal_palette::with_test_default_colors(
        crate::terminal_probe::DefaultColors {
            fg: (240, 240, 240),
            bg: (24, 24, 24),
        },
        || {
            let mut snapshots = Vec::new();
            for state in ["completed", "interrupted"] {
                let lines = cell.display_lines(/*width*/ 40);
                let area = Rect::new(
                    /*x*/ 0,
                    /*y*/ 0,
                    /*width*/ 40,
                    lines.len() as u16,
                );
                let mut buffer = Buffer::empty(area);
                Paragraph::new(lines).render(area, &mut buffer);
                snapshots.push(format!("{state}: {buffer:?}"));
                cell.answers.clear();
                cell.interrupted = true;
            }
            insta::assert_snapshot!(snapshots.join("\n"));
        },
    );
}

#[test]
fn wrapped_answer_copy_source_keeps_label_whitespace_and_unicode() {
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    let answer = "first  界 👩‍💻 continued answer";
    let expected = format!("answer: {answer}");
    for width in [24, 40] {
        let lines = wrap_with_prefix(
            answer,
            width,
            "    answer: ".dim(),
            "            ".dim(),
            Style::default().fg(Color::Cyan),
        );
        let first = lines.first().unwrap().source.as_ref().unwrap();
        assert_eq!(first.text.as_ref(), expected);
        assert_eq!((first.range.start, first.prefix_bytes), (0, 4));
        for line in &lines {
            let source = line.source.as_ref().unwrap();
            assert!(Arc::ptr_eq(&first.text, &source.text));
            assert_eq!(source.text.as_ref(), expected);
            assert!(source.text.is_char_boundary(source.range.start));
            assert!(source.text.is_char_boundary(source.range.end));
        }
        assert_eq!(
            lines.last().unwrap().source.as_ref().unwrap().range.end,
            expected.len()
        );
        assert_eq!(
            first.styled_range(0.."answer: ".len()).spans[0].style,
            Style::default().dim(),
        );
        assert_eq!(
            first.styled_range("answer: ".len()..expected.len()).spans[0]
                .style
                .fg,
            Some(Color::Cyan),
        );
    }
}

#[test]
fn unanswered_question_copy_source_includes_status_without_alignment_spaces() {
    use pretty_assertions::assert_eq;

    let question = "Which long question should remain readable across rows?";
    let cell = RequestUserInputResultCell {
        questions: vec![ToolRequestUserInputQuestion {
            id: "pending".into(),
            header: "Choice".into(),
            question: question.into(),
            is_other: true,
            is_secret: false,
            options: None,
        }],
        answers: HashMap::new(),
        interrupted: true,
    };
    let expected = format!("• {question} (unanswered)");
    for width in [24, 40] {
        let lines = cell.display_hyperlink_lines(width);
        let sources = lines
            .iter()
            .filter_map(|line| line.source.as_ref())
            .filter(|source| source.text.starts_with("• "))
            .collect::<Vec<_>>();
        assert!(sources.len() > 1);
        for source in &sources {
            assert_eq!(source.text.as_ref(), expected);
        }
        assert_eq!(sources[0].prefix_bytes, 2);
        assert_eq!(sources.last().unwrap().range.end, expected.len());
    }
}

#[test]
fn secret_answer_is_masked_in_retained_copy_source_and_raw_export() {
    let secret = "fixture-secret-must-not-be-copied";
    let cell = RequestUserInputResultCell {
        questions: vec![ToolRequestUserInputQuestion {
            id: "secret".into(),
            header: "Secret".into(),
            question: "Enter a secret".into(),
            is_other: true,
            is_secret: true,
            options: None,
        }],
        answers: HashMap::from([(
            "secret".into(),
            ToolRequestUserInputAnswer {
                answers: vec![secret.into()],
            },
        )]),
        interrupted: false,
    };
    for width in [24, 40] {
        let lines = cell.transcript_hyperlink_lines(width);
        let sources = lines
            .iter()
            .filter_map(|line| line.source.as_ref())
            .collect::<Vec<_>>();
        assert!(
            sources
                .iter()
                .any(|source| source.text.contains("answer: ••••••"))
        );
        assert!(sources.iter().all(|source| !source.text.contains(secret)));
        assert!(
            lines
                .iter()
                .all(|line| !line.line.to_string().contains(secret))
        );
        assert!(
            cell.raw_lines()
                .iter()
                .all(|line| !line.to_string().contains(secret))
        );
    }
}
