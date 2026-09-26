use super::StreamCore;
use super::render_source;
use crate::history_cell::HistoryRenderMode;
use crate::markdown_render::preferences;
use codex_config::types::TuiRendering;
use pretty_assertions::assert_eq;

#[test]
fn lists_match_incremental_and_emitted_responses() {
    let cwd = std::env::temp_dir();
    let source = concat!(
        "- [ ] First task with enough words to wrap\n",
        "  - [x] Nested task 日本語 👩‍💻\n- [X] Done\n\n",
        "> 9. [ ] [link](https://example.com)\n> 10. [x] Done\n\n",
        "After.\n\nOne.\n\nTwo.\n\nThree.\n"
    );
    for lists in [true, false] {
        preferences::init(TuiRendering { lists });
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
                    "lists={lists}, width={width}, mode={mode:?}"
                );
            }
        }
    }
}
