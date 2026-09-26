//! Prove mixed-tool overlap causally, without timing model/session startup.

use super::run_turn;
use anyhow::Context;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::sse;
use core_test_support::skip_if_no_network;
use core_test_support::streaming_sse::StreamingSseChunk;
use core_test_support::streaming_sse::start_streaming_sse_server;
use core_test_support::test_codex::test_codex;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;
use std::time::Duration;
use tokio::sync::oneshot;

pub(super) async fn assert_mixed_tools_overlap() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));
    let directory = tempfile::tempdir()?;
    let marker = directory.path().join("shell-started");
    let barrier = json!({
        "barrier": { "id": "mixed-tools", "participants": 2, "timeout_ms": 30_000 }
    })
    .to_string();
    let shell = json!({
        "cmd": format!("printf shell-overlap > {}", shlex::try_quote(&marker.to_string_lossy())?),
        "login": false,
        "yield_time_ms": 1_000,
    })
    .to_string();
    let (release, held) = oneshot::channel();
    let (server, _) = start_streaming_sse_server(vec![
        vec![
            StreamingSseChunk {
                gate: None,
                body: sse(vec![
                    ev_response_created("mixed-first"),
                    ev_function_call("blocked-sync", "test_sync_tool", &barrier),
                    ev_function_call("shell", "exec_command", &shell),
                ]),
            },
            StreamingSseChunk {
                gate: Some(held),
                body: sse(vec![
                    ev_function_call("release-sync", "test_sync_tool", &barrier),
                    ev_completed("mixed-first"),
                ]),
            },
        ],
        vec![StreamingSseChunk {
            gate: None,
            body: sse(vec![
                ev_assistant_message("mixed-message", "done"),
                ev_completed("mixed-followup"),
            ]),
        }],
    ])
    .await;
    let test = test_codex()
        .with_model("test-gpt-5.1-codex")
        .build_with_streaming_server(&server)
        .await?;
    let turn = run_turn(&test, "mix tools");
    tokio::pin!(turn);
    let marker_written = async {
        while tokio::fs::read(&marker).await.unwrap_or_default() != b"shell-overlap" {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    };
    // The first sync tool cannot finish until we deliver its second participant.
    // Serial execution blocks the shell behind that tool and cannot create the marker.
    tokio::select! {
        result = &mut turn => {
            result?;
            anyhow::bail!("turn completed while its synchronization barrier was still held");
        }
        result = tokio::time::timeout(Duration::from_secs(5), marker_written) => {
            result.context("shell must execute while the sync tool remains blocked")?;
        }
    }
    release.send(()).expect("stream should still be waiting");
    turn.await?;

    let requests = server.requests().await;
    assert_eq!(requests.len(), 2);
    let followup: Value = serde_json::from_slice(&requests[1])?;
    for id in ["blocked-sync", "release-sync"] {
        let output = followup["input"]
            .as_array()
            .expect("followup input")
            .iter()
            .find(|item| item["type"] == "function_call_output" && item["call_id"] == id)
            .expect("sync output must be included");
        assert_eq!(output["output"], "ok", "barrier must not time out");
    }
    server.shutdown().await;
    Ok(())
}
