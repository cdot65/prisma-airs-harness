use super::StreamCore;
use super::render_source;
use crate::history_cell::HistoryRenderMode;
use crate::markdown_render::preferences;
use crate::terminal_hyperlinks::lines_with_sources_eq;
use codex_config::types::TuiRendering;
use pretty_assertions::assert_eq;
use std::sync::Arc;

#[test]
fn unfinished_math_preview_retains_hard_lines_and_authored_whitespace() {
    let cwd = std::env::temp_dir();
    let source = "\\[\n  x = \\frac{日本語  a}{b}\n  x = \\frac{日本語  a}{b}\n";
    preferences::init(TuiRendering::default());
    for width in [8, 24, 80] {
        let mut stream = StreamCore::new(
            Some(width),
            &cwd,
            HistoryRenderMode::Rich,
            /*inline_visualization_context*/ None,
        );
        stream.push_delta(source);
        let preview = stream.current_tail_lines();
        let mut logical: Vec<Arc<str>> = Vec::new();
        for line in &preview {
            let origin = line.source.as_ref().expect("pending math source");
            if !logical
                .last()
                .is_some_and(|last| Arc::ptr_eq(last, &origin.text))
            {
                logical.push(Arc::clone(&origin.text));
            }
        }
        assert_eq!(
            logical
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                .join("\n"),
            source,
            "hard lines and repeated source survive width {width}"
        );
        let (_, retained) = stream.finalize_remaining();
        assert_eq!(retained, source);
    }
}

#[test]
fn spatial_math_rows_keep_distinct_logical_sources() {
    let cwd = std::env::temp_dir();
    preferences::init(TuiRendering::default());
    let lines = render_source(
        r"\[\frac{a}{b}\]",
        Some(80),
        &cwd,
        HistoryRenderMode::Rich,
        /*inline_visualization_context*/ None,
    );
    let sources: Vec<_> = lines
        .iter()
        .filter_map(|line| line.source.as_ref())
        .collect();
    assert_eq!(
        sources
            .iter()
            .map(|source| source.text.as_ref())
            .collect::<Vec<_>>(),
        vec!["a", "─", "b"]
    );
    for pair in sources.windows(2) {
        assert!(
            !Arc::ptr_eq(&pair[0].text, &pair[1].text),
            "equation rows are hard breaks"
        );
    }
}

#[test]
fn lists_and_math_match_incremental_and_emitted_responses() {
    let cwd = std::env::temp_dir();
    let source = concat!(
        "- [ ] First task with enough words to wrap\n",
        "  - [x] Nested task 日本語 👩‍💻\n- [X] Done\n\n",
        "> 9. [ ] [link](https://example.com)\n> 10. [x] Done\n\n",
        "日本語 \\(\\alpha^2\\)\n\n",
        "\\[\n\\begin{aligned}x&=1\\\\y&=2\\end{aligned}\n\\]\n\n",
        "After.\n\nOne.\n\nTwo.\n\nThree.\n"
    );
    for lists in [true, false] {
        for math in [true, false] {
            preferences::init(TuiRendering { lists, math });
            for width in [1, 8, 24, 80] {
                for mode in [HistoryRenderMode::Rich, HistoryRenderMode::Raw] {
                    let mut stream = StreamCore::new(
                        Some(width),
                        &cwd,
                        mode,
                        /*inline_visualization_context*/ None,
                    );
                    let render = |source: &str| {
                        render_source(
                            source,
                            Some(width),
                            &cwd,
                            mode,
                            /*inline_visualization_context*/ None,
                        )
                    };
                    let mut emitted = Vec::new();
                    let mut committed = String::new();
                    // Split inside markers as well as words to exercise partial task-list input.
                    for character in source.chars() {
                        committed.push(character);
                        stream.push_delta(&character.to_string());
                        emitted.extend(stream.tick_batch(usize::MAX));
                        if character == '\n' {
                            assert_eq!(stream.render.lines, render(&committed));
                            assert!(
                                lines_with_sources_eq(&stream.render.lines, &render(&committed)),
                                "source metadata differs: lists={lists}, math={math}, width={width}, mode={mode:?}"
                            );
                        }
                    }
                    let (remaining, original) = stream.finalize_remaining();
                    assert_eq!(original, source, "raw source remains byte-for-byte intact");
                    emitted.extend(remaining);
                    assert!(
                        lines_with_sources_eq(&emitted, &render(source)),
                        "emitted source differs: lists={lists}, math={math}, width={width}, mode={mode:?}"
                    );
                    assert_eq!(
                        emitted,
                        render(source),
                        "lists={lists}, math={math}, width={width}, mode={mode:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn unfinished_math_reflows_and_finalizes_without_rewriting_source() {
    let cwd = std::env::temp_dir();
    let source = "\\[\n\\frac{日本語 + abcdefghijklmnop}{x}\n";
    for math in [true, false] {
        preferences::init(TuiRendering {
            math,
            ..TuiRendering::default()
        });
        for mode in [HistoryRenderMode::Rich, HistoryRenderMode::Raw] {
            let mut stream = StreamCore::new(
                Some(80),
                &cwd,
                mode,
                /*inline_visualization_context*/ None,
            );
            stream.push_delta(source);
            // Resize before finalization, including the narrowest terminal width.
            for width in [1, 8, 80] {
                stream.set_width(Some(width));
                assert_eq!(
                    stream.render.lines,
                    render_source(
                        source,
                        Some(width),
                        &cwd,
                        mode,
                        /*inline_visualization_context*/ None
                    )
                );
            }
            let (lines, raw) = stream.finalize_remaining();
            assert_eq!(raw, source);
            assert_eq!(
                lines,
                render_source(
                    source,
                    Some(80),
                    &cwd,
                    mode,
                    /*inline_visualization_context*/ None
                )
            );
        }
    }
}
