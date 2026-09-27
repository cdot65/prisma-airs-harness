use super::*;
use pretty_assertions::assert_eq;

#[test]
fn diff_sources_keep_signs_and_graphemes_without_line_number_gutters() {
    let text = "let 界 = \"👩‍💻 e\u{301}\";";
    for width in [8, 12, 24, 80] {
        let lines = render_wrapped_diff_line(
            /*line_number*/ 12,
            DiffLineType::Insert,
            text,
            width,
            /*line_number_width*/ 2,
            /*syntax_spans*/ None,
            current_diff_render_style_context(),
        );
        let mut copied = String::new();
        for line in &lines {
            let source = line.source.as_ref().unwrap();
            let fragment = &source.text[source.range.clone()];
            assert_eq!(&line.line.to_string()[source.prefix_bytes..], fragment);
            copied.push_str(fragment);
        }
        assert_eq!(copied, format!("+{text}"));
        let emoji_row = lines
            .iter()
            .find(|line| line.line.to_string().contains('👩'))
            .unwrap();
        assert!(emoji_row.line.to_string().contains("👩‍💻"));
    }
}
