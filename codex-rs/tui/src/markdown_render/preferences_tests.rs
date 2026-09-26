//! List rendering is independent of source storage and unrelated Markdown styles.

use super::current;
use super::init;
use crate::markdown_render::render_markdown_text;
use codex_config::types::TuiRendering;
use pretty_assertions::assert_eq;

#[test]
fn disabled_lists_keep_textual_markers_and_restore_without_changing_source() {
    let source = "- [ ] Pending\n- [x] **Done**\n- Ordinary\n";
    init(TuiRendering { lists: false });
    assert_eq!(current(), TuiRendering { lists: false });
    let plain = render_markdown_text(source);
    assert_eq!(plain.to_string(), "- [ ] Pending\n- [x] Done\n- Ordinary");
    init(TuiRendering::default());
    let rich = render_markdown_text(source);
    assert_eq!(rich.to_string(), "☐ Pending\n☑ Done\n• Ordinary");
    // Only marker presentation changes; inline emphasis remains rendered.
    assert!(plain.lines[1].spans.iter().any(|span| {
        span.style
            .add_modifier
            .contains(ratatui::style::Modifier::BOLD)
    }));
    assert!(rich.lines[1].spans.iter().any(|span| {
        span.style
            .add_modifier
            .contains(ratatui::style::Modifier::BOLD)
    }));
    insta::assert_snapshot!(format!(
        "plain:\n{plain}\n\nrich:\n{rich}\n\nsource:\n{source}"
    ));
}
