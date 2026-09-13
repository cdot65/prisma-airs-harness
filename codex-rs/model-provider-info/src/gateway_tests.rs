use super::GatewayRouting;
use pretty_assertions::assert_eq;

#[test]
fn default_route_is_omitted_and_explicit_model_is_preserved() {
    let routing = GatewayRouting {
        default_route: "gateway-default".to_string(),
    };
    assert_eq!(routing.request_model("gateway-default"), Ok(None));
    assert_eq!(
        routing.request_model("@local-vllm/unsloth/Qwen:quantized"),
        Ok(Some("@local-vllm/unsloth/Qwen:quantized".to_string()))
    );
}

#[test]
fn invalid_explicit_routes_fail_instead_of_falling_back() {
    let routing = GatewayRouting {
        default_route: "gateway-default".to_string(),
    };
    for selection in [
        "",
        "ai-gateway",
        "gpt-4.1",
        "@/model",
        "@provider/",
        "@p/m\n",
    ] {
        assert!(routing.request_model(selection).is_err(), "{selection:?}");
    }
}

#[test]
fn local_route_id_cannot_be_confused_with_a_provider_selection() {
    for default_route in ["", "@provider/model", "two words"] {
        let routing = GatewayRouting {
            default_route: default_route.to_string(),
        };
        assert!(routing.request_model(default_route).is_err());
    }
}
#[test]
fn gateway_header_debug_redacts_credentials_without_changing_wire_value() {
    let provider = crate::ModelProviderInfo {
        gateway: Some(super::GatewayRouting {
            default_route: "default".to_string(),
        }),
        http_headers: Some(std::collections::HashMap::from([(
            "x-portkey-api-key".to_string(),
            "test-only-credential".into(),
        )])),
        ..crate::ModelProviderInfo::default()
    };
    let headers = provider.build_header_map().unwrap();
    assert!(!format!("{headers:?}").contains("test-only-credential"));
    assert_eq!(
        headers["x-portkey-api-key"].to_str().unwrap(),
        "test-only-credential"
    );
}

#[test]
fn only_gateway_routing_selects_flat_function_tools() {
    let mut provider = crate::ModelProviderInfo::default();
    assert_eq!(
        provider.to_api_provider(None).unwrap().response_tool_format,
        codex_api::ResponseToolFormat::Namespaced
    );
    provider.gateway = Some(GatewayRouting {
        default_route: "default".to_owned(),
    });
    assert_eq!(
        provider.to_api_provider(None).unwrap().response_tool_format,
        codex_api::ResponseToolFormat::FlatFunctions
    );
}
