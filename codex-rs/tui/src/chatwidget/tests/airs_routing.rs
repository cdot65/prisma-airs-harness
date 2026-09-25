use super::*;
use crate::airs_routing::Event;
use crate::airs_routing::Kind;
use crate::airs_routing::Proposal;
use crate::airs_routing::Selection;
use crate::airs_routing::views;
use crate::render::renderable::Renderable;
use ratatui::backend::TestBackend;

#[tokio::test]
async fn routing_menus_confirm_and_wait_preserve_the_unsent_draft() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.insert_str("Keep this unsent request");
    let selection = Selection {
        saved_config: Some("pc-example-123".into()),
        model: Some("@provider/model".into()),
    };
    let proposal = Proposal {
        thread: ThreadId::new(),
        baseline: selection.clone(),
        target: selection.changed(Kind::Config, Some("pc-other-456".into())),
        kind: Kind::Config,
    };
    let mut snapshots = Vec::new();
    for (name, width, view) in [
        (
            "airs_routing_config",
            110,
            views::menu(Kind::Config, &selection, &[]),
        ),
        (
            "airs_routing_model_narrow",
            55,
            views::menu(
                Kind::Model,
                &Selection::default(),
                &["@fixture/model".into()],
            ),
        ),
        ("airs_routing_confirm_narrow", 55, views::confirm(proposal)),
        ("airs_routing_waiting", 80, views::waiting(/*attempt*/ 7)),
        ("airs_routing_applying", 80, views::applying()),
    ] {
        chat.show_airs_routing(view);
        let mut terminal =
            ratatui::Terminal::new(TestBackend::new(width, chat.desired_height(width).min(40)))
                .unwrap();
        terminal
            .draw(|frame| chat.render(frame.area(), frame.buffer_mut()))
            .unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content
            .chunks(usize::from(width))
            .map(|row| {
                row.iter()
                    .map(ratatui::buffer::Cell::symbol)
                    .collect::<String>()
                    .trim_end()
                    .to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n");
        snapshots.push(format!("{name}\n{rendered}"));
        chat.dismiss_airs_routing();
        assert_eq!(chat.bottom_pane.composer_text(), "Keep this unsent request");
    }
    insta::assert_snapshot!("airs_routing_views", snapshots.join("\n\n"));
    while let Ok(event) = rx.try_recv() {
        assert!(!matches!(
            event,
            AppEvent::AirsRouting(_) | AppEvent::CodexOp(_)
        ));
    }
}

#[test]
fn selecting_a_different_config_clears_override_but_same_config_preserves_it() {
    let original = Selection {
        saved_config: Some("pc-first".into()),
        model: Some("@provider/model".into()),
    };
    assert_eq!(
        original.changed(Kind::Config, original.saved_config.clone()),
        original
    );
    assert_eq!(
        original.changed(Kind::Config, Some("pc-second".into())),
        Selection {
            saved_config: Some("pc-second".into()),
            model: None
        }
    );
    assert_eq!(original.changed(Kind::Config, None), Selection::default());
    assert_eq!(
        original.changed(Kind::Model, None),
        Selection {
            saved_config: Some("pc-first".into()),
            model: None
        }
    );
}

#[tokio::test]
async fn routing_commit_waits_for_matching_authoritative_state_and_ignores_stale_deadlines() {
    let (mut chat, mut rx, _ops) = make_chatwidget_manual(/*model_override*/ None).await;
    chat.config.model_provider.gateway = Some(codex_model_provider_info::GatewayRouting {
        default_route: "gateway-default".into(),
    });
    let thread = ThreadId::new();
    chat.thread_id = Some(thread);
    let target = Selection {
        saved_config: Some("pc-example".into()),
        model: Some("@provider/model".into()),
    };
    let proposal = Proposal {
        thread,
        baseline: Selection::default(),
        target: target.clone(),
        kind: Kind::Model,
    };
    chat.start_airs_routing_commit(/*attempt*/ 7, proposal);
    assert!(chat.airs_routing_blocked());
    chat.fail_airs_routing_commit(/*attempt*/ 6);
    assert!(!chat.airs_routing.uncertain);
    chat.airs_routing_settings_applied();
    assert!(
        !std::iter::from_fn(|| rx.try_recv().ok())
            .any(|event| matches!(event, AppEvent::AirsRouting(Event::Applied(_))))
    );
    chat.insert_str("Draft during delayed confirmation");
    chat.fail_airs_routing_commit(/*attempt*/ 7);
    assert!(chat.airs_routing.uncertain);
    assert!(chat.airs_routing_blocked());
    assert_eq!(
        chat.bottom_pane.composer_text(),
        "Draft during delayed confirmation"
    );
    chat.airs_routing.saved_config = target.saved_config;
    chat.current_collaboration_mode.settings.model = "@provider/model".into();
    chat.active_collaboration_mask = None;
    chat.airs_routing_settings_applied();
    assert!(
        std::iter::from_fn(|| rx.try_recv().ok())
            .any(|event| matches!(event, AppEvent::AirsRouting(Event::Applied(7))))
    );
    chat.finish_airs_routing(/*attempt*/ 7);
    assert!(!chat.airs_routing_blocked());
    chat.fail_airs_routing_commit(/*attempt*/ 7);
    assert!(!chat.airs_routing_blocked());
}
