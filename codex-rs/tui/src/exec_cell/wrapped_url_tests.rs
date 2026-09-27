use super::*;

#[test]
fn transcript_view_renders_wrapped_url_like_rows_without_clipping() {
    let url = "https://example.test/api/v1/projects/alpha-team/releases/2026-02-17/builds/1234567890/artifacts/reports/performance/summary/detail/with/a/very/long/path/that/keeps/going/for/testing/purposes";
    let call = ExecCall {
        call_id: "call-id".to_string(),
        command: vec!["bash".into(), "-lc".into(), "echo done".into()],
        parsed: Vec::new(),
        output: Some(CommandOutput::new(/*exit_code*/ 0, url.to_string())),
        source: ExecCommandSource::Agent,
        start_time: None,
        duration: None,
        interaction_input: None,
    };

    let cell = ExecCell::new(call, /*animations_enabled*/ false);
    let width: u16 = 36;
    let logical_height = cell.transcript_lines(width).len();
    let cells: Vec<std::sync::Arc<dyn HistoryCell>> = vec![std::sync::Arc::new(cell)];
    let mut view = crate::transcript_view::TranscriptView::default();
    view.set_presentation(
        /*detailed*/ true,
        crate::history_cell::HistoryRenderMode::Rich,
    );
    let area = Rect::new(/*x*/ 0, /*y*/ 0, width, /*height*/ 16);
    let mut buffer = Buffer::empty(area);
    view.render(area, &mut buffer, &cells);
    let rendered = buffer
        .content()
        .chunks(usize::from(width))
        .map(|row| {
            row.iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>()
                .trim()
                .to_string()
        })
        .filter(|row| !row.is_empty())
        .collect::<Vec<_>>();

    assert!(
        rendered.len() > logical_height,
        "expected the transcript viewport to wrap URL-like rows, got: {rendered:?}"
    );
    assert!(
        rendered.concat().contains(url),
        "expected the complete URL, got: {rendered:?}"
    );
}
