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
