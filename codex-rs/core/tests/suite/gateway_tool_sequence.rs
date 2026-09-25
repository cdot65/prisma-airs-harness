use codex_core::TurnInputRequest;
use codex_model_provider_info::GatewayRouting;
use codex_protocol::protocol::EventMsg;
use codex_protocol::user_input::UserInput;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::mount_sse_once;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event;
use pretty_assertions::assert_eq;
use serde_json::json;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn gateway_routes_request_serial_tools_and_preserve_model_selection() -> anyhow::Result<()> {
    for (selection, gateway) in [
        ("gpt-5.4", None),
        ("gpt-5.4", Some("gpt-5.4")),
        ("@provider/model", Some("gateway-default")),
    ] {
        let server = start_mock_server().await;
        let response = mount_sse_once(
            &server,
            sse(vec![
                ev_response_created("response"),
                ev_completed("response"),
            ]),
        )
        .await;
        let fixture = test_codex()
            .with_model(selection)
            .with_config(move |config| {
                config.model_provider.gateway = gateway.map(|default_route| GatewayRouting {
                    default_route: default_route.to_string(),
                });
                if gateway.is_some() {
                    config.model_provider_id = "airs".to_string();
                    config.model_provider.name = "Test AIRS".to_string();
                    config.model_provider.requires_openai_auth = false;
                    config.model_provider.env_http_headers = None;
                }
                config.model_provider.supports_websockets = false;
            })
            .build_with_auto_env(&server)
            .await?;
        fixture
            .codex
            .start_or_steer_turn(TurnInputRequest::user_input(vec![UserInput::Text {
                text: "hello".to_string(),
                text_elements: Vec::new(),
            }]))
            .await?;
        wait_for_event(&fixture.codex, |event| {
            if let EventMsg::Error(error) = event {
                panic!("{selection} with gateway {gateway:?}: {}", error.message);
            }
            matches!(event, EventMsg::TurnComplete(_))
        })
        .await;
        let body = response.single_request().body_json();
        assert_eq!(body["parallel_tool_calls"], json!(gateway.is_none()));
        assert_eq!(
            body.get("model").cloned(),
            (gateway != Some(selection)).then(|| json!(selection))
        );
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn saved_gateway_config_changes_reset_model_and_invalid_edits_preserve_both()
-> anyhow::Result<()> {
    use codex_protocol::protocol::Op;
    use codex_protocol::protocol::ThreadSettingsOverrides;
    use core_test_support::responses::mount_sse_sequence;
    let server = start_mock_server().await;
    let responses = mount_sse_sequence(
        &server,
        (0..3)
            .map(|index| {
                let id = format!("routing-{index}");
                sse(vec![ev_response_created(&id), ev_completed(&id)])
            })
            .collect(),
    )
    .await;
    let test = test_codex()
        .with_model("@provider/model")
        .with_config(|config| {
            config.model_provider.gateway = Some(GatewayRouting {
                default_route: "gpt-5.4".into(),
            });
            config.model_provider_id = "airs".into();
            config.model_provider.name = "Test AIRS".into();
            config.model_provider.requires_openai_auth = false;
            config.model_provider.env_http_headers = None;
            config.model_provider.supports_websockets = false;
        })
        .build_with_auto_env(&server)
        .await?;
    for (selection, accepted) in [
        (Some("pc-example"), true),
        (Some("{\"targets\":[]}"), false),
        (None, true),
    ] {
        test.codex
            .submit(Op::ThreadSettings {
                thread_settings: ThreadSettingsOverrides {
                    gateway_config: Some(selection.map(str::to_owned)),
                    ..Default::default()
                },
            })
            .await?;
        wait_for_event(&test.codex, |event| {
            if accepted {
                matches!(event, EventMsg::ThreadSettingsApplied(_))
            } else {
                matches!(event, EventMsg::Error(_))
            }
        })
        .await;
        let snapshot = test.codex.thread_settings_snapshot().await;
        assert_eq!(snapshot.model, "gpt-5.4");
        assert_eq!(
            snapshot.gateway_config.as_deref(),
            if accepted {
                selection
            } else {
                Some("pc-example")
            }
        );
        test.codex
            .start_or_steer_turn(TurnInputRequest::user_input(vec![UserInput::Text {
                text: "hello".into(),
                text_elements: vec![],
            }]))
            .await?;
        wait_for_event(&test.codex, |event| {
            if let EventMsg::Error(error) = event {
                panic!("{}", error.message);
            }
            matches!(event, EventMsg::TurnComplete(_))
        })
        .await;
    }
    let requests = responses.requests();
    assert_eq!(requests.len(), 3);
    for (request, selection) in requests
        .iter()
        .zip([Some("pc-example"), Some("pc-example"), None])
    {
        assert_eq!(request.header("x-portkey-config").as_deref(), selection);
        assert_eq!(request.body_json().get("model"), None);
        assert!(!request.body_json().to_string().contains("pc-example"));
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn saved_gateway_config_survives_restart_with_an_explicit_model() -> anyhow::Result<()> {
    use codex_protocol::protocol::Op;
    use codex_protocol::protocol::ThreadSettingsOverrides;
    let server = start_mock_server().await;
    let initial_response = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("initial"),
            ev_completed("initial"),
        ]),
    )
    .await;
    let mut builder = test_codex()
        .with_model("@provider/model")
        .with_config(|config| {
            config.model_provider.gateway = Some(GatewayRouting {
                default_route: "gpt-5.4".into(),
            });
            config.model_provider_id = "airs".into();
            config.model_provider.name = "Test AIRS".into();
            config.model_provider.requires_openai_auth = false;
            config.model_provider.env_http_headers = None;
            config.model_provider.supports_websockets = false;
        });
    let first = builder.build_with_auto_env(&server).await?;
    first
        .codex
        .submit(Op::ThreadSettings {
            thread_settings: ThreadSettingsOverrides {
                gateway_config: Some(Some("pc-resume".into())),
                model: Some("@provider/model".into()),
                ..Default::default()
            },
        })
        .await?;
    wait_for_event(&first.codex, |event| {
        matches!(event, EventMsg::ThreadSettingsApplied(_))
    })
    .await;
    // Re-selecting the same config must preserve the explicit model.
    first
        .codex
        .submit(Op::ThreadSettings {
            thread_settings: ThreadSettingsOverrides {
                gateway_config: Some(Some("pc-resume".into())),
                ..Default::default()
            },
        })
        .await?;
    wait_for_event(&first.codex, |event| {
        matches!(event, EventMsg::ThreadSettingsApplied(_))
    })
    .await;
    first
        .codex
        .start_or_steer_turn(TurnInputRequest::user_input(vec![UserInput::Text {
            text: "first message after selecting routing".into(),
            text_elements: vec![],
        }]))
        .await?;
    wait_for_event(&first.codex, |event| {
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    let mut foreign = first.codex.thread_settings_snapshot().await;
    foreign.gateway_config = Some("pc-foreign-thread".into());
    first
        .codex
        .append_rollout_items(&[codex_rollout::RolloutItem::EventMsg(
            EventMsg::ThreadSettingsApplied(codex_protocol::protocol::ThreadSettingsAppliedEvent {
                thread_id: Some(codex_protocol::ThreadId::new()),
                thread_settings: foreign,
            }),
        )])
        .await?;
    assert_eq!(
        initial_response
            .single_request()
            .header("x-portkey-config")
            .as_deref(),
        Some("pc-resume")
    );
    // Builder mutators are one-shot; retain the same gateway across restart.
    let provider = first.config.model_provider.clone();
    builder = builder
        .with_model("@provider/model")
        .with_config(move |config| {
            config.model_provider = provider;
            config.model_provider_id = "airs".into();
        });
    let resumed = builder.restart(&server, &first).await?;
    let response = mount_sse_once(
        &server,
        sse(vec![
            ev_response_created("resumed"),
            ev_completed("resumed"),
        ]),
    )
    .await;
    let snapshot = resumed.codex.thread_settings_snapshot().await;
    assert_eq!(snapshot.gateway_config.as_deref(), Some("pc-resume"));
    assert_eq!(snapshot.model, "@provider/model");
    resumed
        .codex
        .start_or_steer_turn(TurnInputRequest::user_input(vec![UserInput::Text {
            text: "hello after restart".into(),
            text_elements: vec![],
        }]))
        .await?;
    wait_for_event(&resumed.codex, |event| {
        if let EventMsg::Error(error) = event {
            panic!("{}", error.message);
        }
        matches!(event, EventMsg::TurnComplete(_))
    })
    .await;
    let request = response.single_request();
    assert_eq!(
        request.header("x-portkey-config").as_deref(),
        Some("pc-resume")
    );
    assert_eq!(request.body_json()["model"], json!("@provider/model"));
    Ok(())
}
