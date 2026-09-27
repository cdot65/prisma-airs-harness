//! Patch summaries and image-tool transcript helpers.

use super::*;
use codex_ansi_escape::ansi_escape;

#[cfg(test)]
#[path = "patches_tests.rs"]
mod tests;
use codex_utils_path_uri::LegacyAppPathString;

#[derive(Debug)]
pub(crate) struct PatchHistoryCell {
    activity_id: String,
    changes: HashMap<PathBuf, FileChange>,
    cwd: PathBuf,
}

impl HistoryCell for PatchHistoryCell {
    fn activity_ids(&self) -> Vec<String> {
        vec![self.activity_id.clone()]
    }

    fn compact_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        crate::diff_render::create_diff_preview_with_links(
            &self.changes,
            &self.cwd,
            usize::from(width),
            super::activity_preview::DETAIL_PREVIEW_LINES,
        )
    }

    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        visible_lines(self.display_hyperlink_lines(width))
    }

    fn display_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        crate::diff_render::create_diff_summary_with_links(
            &self.changes,
            &self.cwd,
            usize::from(width),
        )
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        self.display_hyperlink_lines(width)
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        plain_lines(create_diff_summary(
            &self.changes,
            &self.cwd,
            RAW_DIFF_SUMMARY_WIDTH,
        ))
    }
}
/// Create a new `PendingPatch` cell that lists the file‑level summary of
/// a proposed patch. The summary lines should already be formatted (e.g.
/// "A path/to/file.rs").
pub(crate) fn new_patch_event(
    changes: HashMap<PathBuf, FileChange>,
    cwd: &Path,
) -> PatchHistoryCell {
    PatchHistoryCell {
        activity_id: format!("patch:{}", uuid::Uuid::new_v4()),
        changes,
        cwd: cwd.to_path_buf(),
    }
}

pub(crate) fn new_patch_apply_failure(stderr: String) -> PatchFailureCell {
    PatchFailureCell {
        activity_id: format!("patch-failure:{}", uuid::Uuid::new_v4()),
        stderr,
    }
}

/// Failed patch attempts retain available diagnostics for local disclosure and full transcript.
#[derive(Debug)]
pub(crate) struct PatchFailureCell {
    activity_id: String,
    stderr: String,
}

impl HistoryCell for PatchFailureCell {
    fn activity_ids(&self) -> Vec<String> {
        vec![self.activity_id.clone()]
    }

    fn compact_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        use super::activity_preview::DETAIL_PREVIEW_LINES;
        use super::activity_preview::clipped_prefixed_line;
        let mut lines = vec![clipped_prefixed_line(
            Line::from("✘ ".magenta().bold()),
            Line::from("Failed to apply patch".magenta().bold()),
            width,
        )];
        let error = if self.stderr.trim().is_empty() {
            "(error details unavailable)"
        } else {
            &self.stderr
        };
        let mut preview = crate::tool_output::ToolOutputPreview::new(
            usize::from(width).saturating_sub(4),
            /*omitted*/ 0,
        );
        for raw in error.lines().take(DETAIL_PREVIEW_LINES) {
            let bounded = &raw[..raw.floor_char_boundary(16 * 1024)];
            let line = codex_ansi_escape::ansi_escape_line(&bounded.replace('\t', "    ")).dim();
            preview.push_hyperlink_line(line.into());
        }
        for (index, line) in preview
            .finish_hyperlink_lines()
            .into_iter()
            .take(DETAIL_PREVIEW_LINES)
            .enumerate()
        {
            lines.push(clipped_prefixed_line(
                Line::from(if index == 0 { "  └ " } else { "    " }.dim()),
                line.line,
                width,
            ));
        }
        lines
    }

    fn transcript_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let mut lines = prefix_hyperlink_lines(
            vec![Line::from("Failed to apply patch".magenta().bold()).into()],
            "✘ ".magenta().bold(),
            "  ".into(),
        );
        let error = if self.stderr.trim().is_empty() {
            "(error details unavailable)"
        } else {
            &self.stderr
        };
        let mut diagnostics = ansi_escape(&error.replace('\t', "    ")).lines;
        for line in &mut diagnostics {
            for span in &mut line.spans {
                span.style = span.style.add_modifier(Modifier::DIM);
            }
        }
        lines.extend(crate::terminal_hyperlinks::adaptive_wrap_hyperlink_lines(
            &plain_hyperlink_lines(diagnostics),
            RtOptions::new(usize::from(width).max(/*other*/ 1))
                .initial_indent("  └ ".dim().into())
                .subsequent_indent("    ".into()),
        ));
        lines
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from("Failed to apply patch")];
        lines.extend(plain_lines(ansi_escape(&self.stderr).lines));
        lines
    }

    fn display_lines(&self, _width: u16) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        // Failure title
        lines.push(Line::from("✘ Failed to apply patch".magenta().bold()));

        if !self.stderr.trim().is_empty() {
            let output = output_lines(
                Some(&CommandOutput::new(
                    /*exit_code*/ 1,
                    self.stderr.clone(),
                )),
                OutputLinesParams {
                    line_limit: TOOL_CALL_MAX_LINES,
                    only_err: true,
                    include_angle_pipe: true,
                    include_prefix: true,
                },
            );
            lines.extend(output.lines);
        }

        lines
    }
}

pub(crate) fn new_view_image_tool_call(path: LegacyAppPathString, cwd: &Path) -> PlainHistoryCell {
    let display_path = path
        .to_inferred_path_uri()
        .and_then(|path| path.to_abs_path().ok())
        .map(|path| display_path_for(path.as_path(), cwd))
        .unwrap_or_else(|| path.into_string());

    let lines: Vec<Line<'static>> = vec![
        vec!["• ".dim(), "Viewed Image".bold()].into(),
        vec!["  └ ".dim(), display_path.dim()].into(),
    ];

    PlainHistoryCell { lines }
}

pub(crate) fn new_image_generation_call(
    call_id: String,
    status: &str,
    revised_prompt: Option<String>,
    saved_path: Option<AbsolutePathBuf>,
) -> PlainHistoryCell {
    let detail = revised_prompt.unwrap_or(call_id);
    let heading = if status == "failed" {
        vec!["✗ ".red().bold(), "Image generation failed".bold()].into()
    } else {
        vec!["• ".dim(), "Generated Image:".bold()].into()
    };
    let mut lines: Vec<Line<'static>> = vec![heading, vec!["  └ ".dim(), detail.dim()].into()];
    if let Some(saved_path) = saved_path {
        let saved_path = Url::from_file_path(saved_path.as_path())
            .map(|url| url.to_string())
            .unwrap_or_else(|_| saved_path.display().to_string());
        lines.push(vec!["  └ ".dim(), "Saved to: ".dim(), saved_path.into()].into());
    }

    PlainHistoryCell { lines }
}
