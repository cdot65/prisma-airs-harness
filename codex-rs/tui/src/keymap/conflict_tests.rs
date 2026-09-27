//! Regression coverage for conflict diagnostics and validation pass ordering.

use super::RuntimeKeymap;
use codex_config::types::TuiKeymap;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

#[test]
fn conflicting_contexts_report_the_first_conflict_in_validation_order() {
    let contexts = [
        ("editor", "insert_newline", "yank"),
        ("vim_normal", "enter_insert", "cancel_operator"),
        ("vim_operator", "delete_line", "cancel"),
        ("vim_text_object", "word", "cancel"),
        ("pager", "scroll_up", "close_transcript"),
        ("list", "move_up", "cancel"),
        ("agents", "search", "toggle_grouping"),
        ("approval", "open_fullscreen", "cancel"),
    ];
    let mut config = serde_json::Map::new();
    for (context, first, second) in contexts {
        config.insert(context.to_string(), json!({first: "f12", second: "f12"}));
    }

    for (context, first, second) in contexts {
        let keymap: TuiKeymap = serde_json::from_value(Value::Object(config.clone()))
            .expect("valid keymap configuration");
        assert_eq!(
            RuntimeKeymap::from_config(&keymap).expect_err("expected binding conflict"),
            format!(
                "Ambiguous `tui.keymap.{context}` bindings: `{first}` and `{second}` use the same key. \
Set unique keys in `~/.codex/config.toml` and retry. \
See the Codex keymap documentation for supported actions and examples."
            )
        );
        config.remove(context);
    }
}

#[test]
fn activity_default_preserves_configured_keys_and_prefixes() {
    use crate::key_hint;
    use crossterm::event::KeyCode;

    for (context, action) in [
        ("global", "copy"),
        ("editor", "move_left"),
        ("list", "move_up"),
    ] {
        for configured in ["f4", "f4 f12"] {
            let keymap: TuiKeymap =
                serde_json::from_value(json!({context: {action: configured}})).unwrap();
            let runtime = RuntimeKeymap::from_config(&keymap)
                .expect("a new activity default must preserve existing configuration");
            assert!(
                !super::user_bindings(&runtime.app.focus_activity)
                    .contains(&key_hint::plain(KeyCode::F(4)))
            );
        }
    }
}

#[test]
fn explicit_activity_remapping_and_unbinding_replace_all_defaults() {
    use crate::key_hint;
    use crossterm::event::KeyCode;

    for (configured, expected, hint) in [
        (
            json!("f12"),
            vec![key_hint::plain(KeyCode::F(12))],
            Some("f12"),
        ),
        (json!([]), Vec::new(), None),
        (json!("ctrl-x t"), Vec::new(), Some("ctrl + x t")),
    ] {
        let keymap: TuiKeymap =
            serde_json::from_value(json!({"global": {"focus_activity": configured}})).unwrap();
        let runtime = RuntimeKeymap::from_config(&keymap).expect("valid explicit activity binding");
        assert_eq!(super::user_bindings(&runtime.app.focus_activity), expected);
        assert_eq!(
            runtime
                .primary_hint(super::KeymapContext::Global, "focus_activity")
                .map(crate::key_hint::ShortcutHint::display_label),
            hint.map(str::to_string),
        );
    }
}

#[test]
fn explicit_activity_bindings_cannot_shadow_list_navigation() {
    for (focus, movement) in [
        ("f12", "f12"),
        ("f12 t", "f12"),
        ("f12", "f12 t"),
        ("f12 t", "f12 t"),
    ] {
        let keymap: TuiKeymap = serde_json::from_value(json!({
            "global": {"focus_activity": focus},
            "list": {"move_up": movement},
        }))
        .unwrap();
        let error = RuntimeKeymap::from_config(&keymap)
            .expect_err("activity focus shares list navigation while inspecting groups");
        assert!(
            error.contains("focus_activity") && error.contains("move_up"),
            "{error}"
        );
    }
}
