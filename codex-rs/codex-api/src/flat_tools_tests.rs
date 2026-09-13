use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn declarations_history_and_choice_share_stable_aliases() {
    let mut body = json!({
        "tools": [{"type":"namespace", "name":"mcp__airs", "tools":[
            {"type":"function", "name":"list", "description":"Read workspaces",
             "parameters":{"type":"object"}, "strict":false}
        ]}, {"type":"function", "name":"shell"}],
        "input":[
            {"type":"function_call", "namespace":"mcp__airs", "name":"list", "arguments":"{}", "call_id":"c1"},
            {"type":"function_call_output", "call_id":"c1", "output":"ok"},
            {"type":"function_call", "namespace":"removed", "name":"tool", "arguments":"{}", "call_id":"c2"}
        ],
        "tool_choice":{"type":"function", "namespace":"mcp__airs", "name":"list"}
    });
    let original = body.clone();
    FlatTools::prepare(&mut body).unwrap();
    assert_eq!(
        body,
        json!({
            "tools":[{"type":"function", "name":"mcp__airs__list", "description":"Read workspaces",
                      "parameters":{"type":"object"}, "strict":false}, {"type":"function", "name":"shell"}],
            "input":[
                {"type":"function_call", "name":"mcp__airs__list", "arguments":"{}", "call_id":"c1"},
                {"type":"function_call_output", "call_id":"c1", "output":"ok"},
                {"type":"function_call", "name":"removed__tool", "arguments":"{}", "call_id":"c2"}
            ],
            "tool_choice":{"type":"function", "name":"mcp__airs__list"}
        })
    );
    let mut next = original;
    next["tools"].as_array_mut().unwrap().reverse();
    let next_mapping = FlatTools::prepare(&mut next).unwrap();
    assert_eq!(
        next_mapping.names["mcp__airs__list"],
        (Some("mcp__airs".to_owned()), "list".to_owned())
    );
}

#[test]
fn ambiguous_names_and_unsupported_namespace_tools_fail_closed() {
    for tools in [
        json!([{"type":"namespace","name":"a","tools":[{"type":"function","name":"b"}]},
               {"type":"function","name":"a__b"}]),
        json!([{"type":"namespace","name":"a","tools":[{"type":"custom","name":"b"}]}]),
        json!([{"type":"namespace","name":"a","tools":[{"type":"function","name":"b".repeat(128)}]}]),
    ] {
        assert!(FlatTools::prepare(&mut json!({"tools":tools})).is_err());
    }
    let mut body = json!({"tools":[{"type":"function","name":"a__b"}],
        "input":[{"type":"function_call","namespace":"a","name":"b"}]});
    assert!(FlatTools::prepare(&mut body).is_err());
}

#[tokio::test]
async fn streaming_restores_added_and_done_preserving_arguments_ids_and_errors() {
    let mapping = FlatTools::prepare(&mut json!({"tools":[
        {"type":"namespace","name":"mcp__airs","tools":[{"type":"function","name":"list"}]}
    ]}))
    .unwrap();
    let wire: ResponseItem = serde_json::from_value(json!({"type":"function_call",
        "name":"mcp__airs__list","arguments":"{}","call_id":"c1","id":"fc_1"}))
    .unwrap();
    let canonical: ResponseItem = serde_json::from_value(json!({"type":"function_call",
        "namespace":"mcp__airs","name":"list","arguments":"{}","call_id":"c1","id":"fc_1"}))
    .unwrap();
    let (tx, rx_event) = mpsc::channel(4);
    tx.send(Ok(ResponseEvent::OutputItemAdded(wire.clone())))
        .await
        .unwrap();
    tx.send(Ok(ResponseEvent::OutputItemDone(wire)))
        .await
        .unwrap();
    tx.send(Err(ApiError::Stream("fixture failure".to_owned())))
        .await
        .unwrap();
    drop(tx);
    let mut output = mapping.restore_stream(ResponseStream {
        rx_event,
        upstream_request_id: Some("request-1".to_owned()),
    });
    assert_eq!(output.upstream_request_id, Some("request-1".to_owned()));
    let Some(Ok(ResponseEvent::OutputItemAdded(added))) = output.rx_event.recv().await else {
        panic!("missing added")
    };
    let Some(Ok(ResponseEvent::OutputItemDone(done))) = output.rx_event.recv().await else {
        panic!("missing done")
    };
    assert_eq!(added, canonical);
    assert_eq!(done, canonical);
    assert!(
        matches!(output.rx_event.recv().await, Some(Err(ApiError::Stream(message))) if message == "fixture failure")
    );
    assert!(output.rx_event.recv().await.is_none());
}

#[test]
fn unknown_or_explicitly_namespaced_response_names_are_not_rerouted() {
    let mapping = FlatTools::prepare(&mut json!({"tools":[
        {"type":"namespace","name":"a","tools":[{"type":"function","name":"b"}]}
    ]}))
    .unwrap();
    for item in [
        json!({"type":"function_call","name":"unknown","arguments":"{}","call_id":"c"}),
        json!({"type":"function_call","namespace":"other","name":"a__b","arguments":"{}","call_id":"c"}),
    ] {
        let original: ResponseItem = serde_json::from_value(item).unwrap();
        let mut event = ResponseEvent::OutputItemDone(original.clone());
        mapping.restore_event(&mut event);
        let ResponseEvent::OutputItemDone(actual) = event else {
            panic!("wrong event")
        };
        assert_eq!(actual, original);
    }
}
