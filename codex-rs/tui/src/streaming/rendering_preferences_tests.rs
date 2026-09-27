use super::StreamCore;
use super::render_source;
use crate::history_cell::HistoryRenderMode;
use crate::markdown_render::preferences;
use codex_config::types::TuiRendering;
use pretty_assertions::assert_eq;

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
                        }
                    }
                    let (remaining, original) = stream.finalize_remaining();
                    assert_eq!(original, source, "raw source remains byte-for-byte intact");
                    emitted.extend(remaining);
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
