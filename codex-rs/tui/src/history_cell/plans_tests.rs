use super::*;
use pretty_assertions::assert_eq;

#[test]
fn finalized_plan_reuses_lines_primed_by_transcript_height() {
    let cell = new_proposed_plan("1. Inspect **markdown**".to_string(), Path::new("/tmp"));
    let width = 48;

    assert_eq!(
        crate::transcript_view::TextLayout::new(cell.transcript_hyperlink_lines(width), width)
            .row_count(),
        5
    );
    cell.rendered_lines
        .cached
        .lock()
        .expect("render cache lock")
        .as_mut()
        .expect("render cache should be populated")
        .1 = vec![HyperlinkLine::from("cached")];

    assert_eq!(
        visible_lines(cell.transcript_hyperlink_lines(width)),
        vec![Line::from("cached")]
    );
}

#[test]
fn finalized_plan_file_citation_renders_as_local_path_snapshot() {
    let cwd = std::env::temp_dir();
    let output = cwd.join("Quarterly Report.xlsx").display().to_string();
    let plan = new_proposed_plan(
        format!(
            "- :codex-file-citation{{path=\"{output}\" purpose=\"output\" artifact_kind=\"workbook\"}}\n"
        ),
        &cwd,
    );

    let rendered = ratatui::text::Text::from(plan.display_lines(/*width*/ 80));

    insta::assert_snapshot!(rendered, @"• Proposed Plan\n \n \n  • Quarterly Report.xlsx");
}

#[test]
fn compact_plan_prioritizes_work_and_preserves_copy_labels_and_details() {
    let cell = new_plan_update(UpdatePlanArgs {
        explanation: Some("Retain full explanation".into()),
        plan: vec![
            PlanItemArg {
                step: "Completed first".into(),
                status: StepStatus::Completed,
            },
            PlanItemArg {
                step: "Pending work".into(),
                status: StepStatus::Pending,
            },
            PlanItemArg {
                step: "Current work".into(),
                status: StepStatus::InProgress,
            },
            PlanItemArg {
                step: "Extra pending work".into(),
                status: StepStatus::Pending,
            },
        ],
    });
    let lines = cell.compact_hyperlink_lines(/*width*/ 32);
    let copied = lines
        .iter()
        .map(|line| {
            let source = line.source.as_ref().unwrap();
            source.text[source.range.clone()].to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        copied,
        vec![
            "Updated Plan · 1/4 complete",
            "□ Current work",
            "□ Pending work",
            "□ Extra pending work"
        ]
    );
    assert!(cell.has_hidden_activity_details(/*width*/ 32));
    let raw = cell
        .raw_lines()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(raw.contains("Retain full explanation"));
    assert!(raw.find("Completed first").unwrap() < raw.find("Pending work").unwrap());
    insta::assert_snapshot!(ratatui::text::Text::from(visible_lines(lines)), @"
    • Updated Plan · 1/4 complete
      └ □ Current work
        □ Pending work
        □ Extra pending work
    ");
    for width in [0, 1, 3, 16] {
        assert!(
            cell.compact_hyperlink_lines(width)
                .iter()
                .all(|line| line.width() <= usize::from(width))
        );
    }
}
