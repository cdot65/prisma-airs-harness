use super::*;
use codex_api::ResponseToolFormat;
use pretty_assertions::assert_eq;
use serde_json::json;

#[tokio::test]
async fn typed_and_raw_requests_flatten_only_for_gateway_providers() -> Result<()> {
    for format in [
        ResponseToolFormat::Namespaced,
        ResponseToolFormat::FlatFunctions,
    ] {
        let tools = json!([{"type":"namespace","name":"mcp__airs","tools":[
            {"type":"function","name":"list","parameters":{"type":"object"}}
        ]}]);
        let request = ResponsesApiRequest {
            model: None,
            instructions: "Use the read tool".into(),
            input: vec![],
            tools: Some(Arc::<RawValue>::from(RawValue::from_string(tools.to_string())?).into()),
            tool_choice: "auto".into(),
            parallel_tool_calls: false,
            reasoning: None,
            store: false,
            stream: true,
            stream_options: None,
            include: vec![],
            service_tier: None,
            prompt_cache_key: None,
            text: None,
            client_metadata: None,
            access_programs: None,
        };
        let raw = serde_json::to_value(&request)?;
        let state = RecordingState::default();
        let mut deployment = provider("custom gateway name");
        deployment.response_tool_format = format;
        let client = ResponsesClient::new(
            RecordingTransport::new(state.clone()),
            deployment,
            Arc::new(NoAuth),
        );
        let _typed = client
            .stream_request(request, ResponsesOptions::default())
            .await?;
        let _raw = client
            .stream(
                raw.clone(),
                HeaderMap::new(),
                Compression::None,
                /*turn_state*/ None,
            )
            .await?;
        let requests = state.take_stream_requests();
        assert_eq!(requests.len(), 2);
        let mut expected = raw;
        if format == ResponseToolFormat::FlatFunctions {
            expected["tools"] = json!([{"type":"function","name":"mcp__airs__list","parameters":{"type":"object"}}]);
        }
        for request in requests {
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(request_body_bytes(&request))?,
                expected
            );
        }
    }
    Ok(())
}
