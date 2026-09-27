use super::*;
use pretty_assertions::assert_eq;

#[test]
fn compact_patches_retain_full_changes_and_failure_details() {
    let patch = new_patch_event(
        HashMap::from([
            (
                PathBuf::from("new.txt"),
                FileChange::Add {
                    content: "one\ntwo\nthree\nfour\nfive\n".into(),
                },
            ),
            (
                PathBuf::from("existing.txt"),
                FileChange::Update {
                    unified_diff: "@@ -1,3 +1,3 @@\n context\n-before\n+after\n tail\n".into(),
                    move_path: None,
                },
            ),
        ]),
        Path::new("."),
    );
    let failure = new_patch_apply_failure("first\nsecond\nthird\nfourth diagnostic".into());
    let missing = new_patch_apply_failure(String::new());
    let mut snapshots = Vec::new();
    for (label, cell) in [
        ("patch", &patch as &dyn HistoryCell),
        ("failure", &failure),
        ("missing diagnostics", &missing),
    ] {
        let compact = visible_lines(cell.compact_hyperlink_lines(/*width*/ 40));
        let full = visible_lines(cell.transcript_hyperlink_lines(/*width*/ 40));
        snapshots.push(format!(
            "{label}\nCompact\n{}\nFull\n{}",
            Text::from(compact),
            Text::from(full)
        ));
    }
    insta::assert_snapshot!(snapshots.join("\n\n"));
}

#[test]
fn compact_patch_failure_copy_retains_visible_diagnostics_only() {
    let cell = new_patch_apply_failure("one\n  └ real output\nthird\nfourth hidden".into());
    let lines = cell.compact_hyperlink_lines(/*width*/ 40);
    let copied = lines
        .iter()
        .map(|line| {
            let source = line.source.as_ref().unwrap();
            source.text[source.range.clone()].to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        copied,
        vec!["Failed to apply patch", "one", "  └ real output", "third"]
    );
    assert!(
        cell.raw_lines()
            .iter()
            .any(|line| line.to_string() == "fourth hidden")
    );
}

#[test]
fn compact_patch_copy_keeps_change_signs_and_clips_long_headings() {
    let cell = new_patch_event(
        HashMap::from([(
            PathBuf::from("very-long-file-name-for-preview.txt"),
            FileChange::Add {
                content: "first\nsecond\nthird\nfourth hidden".into(),
            },
        )]),
        Path::new("."),
    );
    let lines = cell.compact_hyperlink_lines(/*width*/ 24);
    assert!(lines.iter().all(|line| line.width() <= 24));
    let copied = lines
        .iter()
        .map(|line| {
            let source = line.source.as_ref().unwrap();
            source.text[source.range.clone()].to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        copied,
        vec!["Added very-long-file-…", "+first", "+second", "+third"]
    );
    assert!(
        cell.raw_lines()
            .iter()
            .any(|line| line.to_string().contains("fourth hidden"))
    );
}

#[test]
fn compact_patch_failure_bounds_dense_combining_diagnostics() {
    let cell = new_patch_apply_failure(format!("e{}\nlast diagnostic", "\u{301}".repeat(100_000)));
    let lines = cell.compact_hyperlink_lines(/*width*/ 24);
    assert!(lines.len() <= 4);
    assert!(lines.iter().all(|line| line.width() <= 24));
    assert!(
        lines
            .iter()
            .all(|line| line.source.as_ref().unwrap().text.len() <= 16 * 1024)
    );
    assert!(
        cell.raw_lines()
            .iter()
            .any(|line| line.to_string() == "last diagnostic")
    );
}

#[test]
fn failed_patch_keeps_diagnostics_beyond_the_legacy_preview() {
    let diagnostics = (1..=12)
        .map(|line| format!("diagnostic line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let cell = new_patch_apply_failure(diagnostics.clone());
    assert_eq!(
        cell.raw_lines(),
        std::iter::once(Line::from("Failed to apply patch"))
            .chain(diagnostics.lines().map(|line| Line::from(line.to_owned())))
            .collect::<Vec<_>>()
    );
}
