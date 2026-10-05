use super::ContextBinding;
use super::FRAME_LIMIT;
use super::REPLAY_LIMIT;
use super::read_frame;
use codex_app_server::in_process::DesktopReplay;
use codex_app_server::in_process::DesktopRestoreParams;
use codex_app_server::in_process::InProcessClientHandle;
use codex_app_server::in_process::InProcessServerEvent;
use codex_app_server_protocol::ClientNotification;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::JSONRPCMessage;
use codex_app_server_protocol::JSONRPCRequest;
use codex_app_server_protocol::RequestId;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;
use std::collections::HashSet;
use tokio::io::AsyncBufRead;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use zeroize::Zeroizing;

#[path = "airs_desktop_policy.rs"]
mod policy;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProtectedReplay {
    context: ContextBinding,
    replay: DesktopReplay,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RestoreRequest {
    context: ContextBinding,
    replay: DesktopReplay,
    dynamic_tools: Vec<codex_protocol::dynamic_tools::DynamicToolSpec>,
}

pub(super) async fn write_json<W: AsyncWrite + Unpin>(
    writer: &mut W,
    value: &Value,
    secret: &str,
) -> anyhow::Result<()> {
    let mut value = value.clone();
    redact(&mut value, secret)?;
    let frame = serde_json::to_string(&value)?;
    anyhow::ensure!(
        frame.len() < REPLAY_LIMIT,
        "Desktop output exceeded its limit"
    );
    writer.write_all(frame.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

fn redact(value: &mut Value, secret: &str) -> anyhow::Result<()> {
    let mut limit = 0usize;
    let mut pending = vec![&*value];
    while let Some(part) = pending.pop() {
        match part {
            Value::String(text) => limit = limit.max(text.len()),
            Value::Array(values) => pending.extend(values),
            Value::Object(values) => {
                limit = limit.max(values.keys().map(String::len).max().unwrap_or_default());
                pending.extend(values.values());
            }
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
    }
    let limit = limit.min(REPLAY_LIMIT);
    let mut encodings = Zeroizing::new(Vec::new());
    let mut encoded = Zeroizing::new(secret.to_owned());
    loop {
        let quoted = Zeroizing::new(serde_json::to_string(encoded.as_str())?);
        let next = Zeroizing::new(
            quoted
                .get(1..quoted.len().saturating_sub(1))
                .unwrap_or_default()
                .to_owned(),
        );
        if next.is_empty() || next == encoded || next.len() > limit {
            break;
        }
        encodings.push(next.to_string());
        encoded = next;
    }
    redact_nested(value, secret, &encodings, /*depth*/ 0);
    Ok(())
}

fn redact_nested(value: &mut Value, secret: &str, encodings: &[String], depth: usize) {
    if secret.is_empty() {
        return;
    }
    match value {
        Value::String(text) => {
            *text = text.replace(secret, "[protected credential]");
            if depth < 8
                && let Ok(mut decoded) = serde_json::from_str::<Value>(text)
            {
                let original = decoded.clone();
                redact_nested(&mut decoded, secret, encodings, depth + 1);
                if decoded != original {
                    *text = serde_json::to_string(&decoded).unwrap_or_default();
                }
            }
            // Function arguments and diagnostic text can themselves contain
            // serialized JSON, so the decoded outer string can still escape a
            // quote/backslash in the credential. Bound repeated JSON encoding.
            for encoded in encodings {
                if text.contains(encoded) {
                    *text = text.replace(encoded, "[protected credential]");
                }
            }
        }
        Value::Array(values) => values
            .iter_mut()
            .for_each(|value| redact_nested(value, secret, encodings, depth)),
        Value::Object(values) => {
            let previous = std::mem::take(values);
            for (key, mut value) in previous {
                redact_nested(&mut value, secret, encodings, depth);
                let mut key = Value::String(key);
                redact_nested(&mut key, secret, encodings, depth);
                values.insert(
                    key.as_str().unwrap_or("[protected field]").to_owned(),
                    value,
                );
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn rejection(id: RequestId) -> Value {
    json!({"id":id,"error":{"code":-32602,"message":"Desktop request rejected"}})
}

pub(super) async fn serve<R: AsyncBufRead + Unpin>(
    mut input: R,
    mut runtime: InProcessClientHandle,
    context: ContextBinding,
    secret: Zeroizing<String>,
) -> anyhow::Result<()> {
    // Cancellation must also release a pending write when the native host has
    // stopped reading. Keep desktop's owned pipe outside blocking stdio tasks.
    #[cfg(unix)]
    let mut output = {
        use std::os::fd::AsFd;
        tokio::net::unix::pipe::Sender::from_owned_fd(
            std::io::stdout().as_fd().try_clone_to_owned()?,
        )?
    };
    #[cfg(not(unix))]
    let mut output = tokio::io::stdout();
    write_json(&mut output, &json!({"id":0,"result":{"v":1,"protocol":"airs-desktop/1","runtimeVersion":codex_utils_home_dir::AIRS_HARNESS_VERSION,"tenantId":context.tenant_id,"workspaceId":context.workspace_id,"credentialId":context.credential_id,"credentialEpoch":context.credential_epoch}}), &secret).await?;
    let sender = runtime.sender();
    let mut pending = FuturesUnordered::new();
    let mut request_ids = HashSet::new();
    let mut approvals: HashMap<RequestId, (String, String, String)> = HashMap::new();
    let mut threads = HashSet::new();
    let mut initialized = false;
    let result = async {
        loop {
            tokio::select! {
                frame = read_frame(&mut input, REPLAY_LIMIT) => {
                    let Some(frame) = frame? else { break; };
                    let mut message: Value = serde_json::from_slice(&frame).map_err(|_| anyhow::anyhow!("Invalid desktop frame"))?;
                    // The protected inference identity cannot enter the model
                    // through an accidental host tool echo or user RPC either.
                    redact(&mut message, &secret)?;
                    let message: JSONRPCMessage = serde_json::from_value(message).map_err(|_| anyhow::anyhow!("Invalid desktop frame"))?;
                    match message {
                        JSONRPCMessage::Request(mut request) => {
                            let replay_method = matches!(request.method.as_str(), "desktop/history/read" | "desktop/history/restore");
                            if (!replay_method && frame.len() > FRAME_LIMIT)
                                || request_ids.len() >= 8
                                || request_ids.contains(&request.id)
                                || (request.method != "initialize" && !initialized)
                            {
                                write_json(&mut output, &rejection(request.id), &secret).await?;
                                continue;
                            }
                            let id = request.id.clone();
                            let method = request.method.clone();
                            let native_request = if replay_method { None } else { policy::validate(&mut request, &context, &threads).ok() };
                            let restore: Option<RestoreRequest> = if method == "desktop/history/restore" { request.params.clone().and_then(|params| serde_json::from_value(params).ok()) } else { None };
                            let history_thread = request.params.as_ref().and_then(|p| p.get("threadId")).and_then(Value::as_str).map(str::to_owned);
                            if (method == "desktop/history/read" && !history_thread.as_ref().is_some_and(|thread| threads.contains(thread)))
                                || (method == "desktop/history/restore" && !restore.as_ref().is_some_and(|replay| replay.context == context && replay.replay.history.iter().all(|item| match item {
                                    codex_history::RolloutItem::EventMsg(codex_protocol::protocol::EventMsg::ThreadSettingsApplied(event)) => event.thread_settings.gateway_config == context.gateway_config,
                                    _ => true,
                                })))
                                || (!replay_method && native_request.is_none())
                            {
                                write_json(&mut output, &rejection(id), &secret).await?;
                                continue;
                            }
                            request_ids.insert(id.clone());
                            if method == "turn/interrupt" && let Some(thread) = &history_thread {
                                approvals.retain(|_, (_,pending_thread,_)| pending_thread != thread);
                            }
                            let sender = sender.clone();
                            let context = context.clone();
                            pending.push(async move {
                                // Stop owned local commands before core cancellation
                                // can kill their parents and lose descendant ancestry.
                                #[cfg(target_os = "macos")]
                                if method == "turn/interrupt"
                                    && !matches!(tokio::task::spawn_blocking(codex_utils_pty::cancel_desktop_commands).await, Ok(Ok(()))) {
                                    return (id.clone(), method, rejection(id));
                                }
                                let result = if let Some(request) = native_request {
                                    match sender.request(request).await {
                                        Ok(Ok(result)) => {
                                            if method == "thread/start"
                                                && let Some(config) = &context.gateway_config {
                                                    let route = ClientRequest::try_from(JSONRPCRequest {id:RequestId::String(format!("desktop-routing-{}",uuid::Uuid::new_v4())),method:"thread/settings/update".into(),params:Some(json!({"threadId":result["thread"]["id"],"gatewayConfig":config,"model":context.model})),trace:None});
                                                    if let Ok(route) = route {
                                                        if !matches!(sender.request(route).await, Ok(Ok(_))) { return (id.clone(), method, rejection(id)); }
                                                    } else { return (id.clone(), method, rejection(id)); }
                                                }
                                            json!({"id":id,"result":result})
                                        }
                                        // Local cleanup can complete a short tool
                                        // before core observes the interruption.
                                        // It is already safely stopped; preserve
                                        // idempotence for this exact terminal race.
                                        #[cfg(target_os = "macos")]
                                        Ok(Err(error)) if method == "turn/interrupt"
                                            && error.code == -32600
                                            && error.message == "no active turn to interrupt" => json!({"id":id,"result":{}}),
                                        Ok(Err(_)) | Err(_) => rejection(id.clone()),
                                    }
                                } else if let Some(replay) = restore {
                                    match sender.restore_desktop_history(DesktopRestoreParams{replay:replay.replay,current_tools:replay.dynamic_tools}).await {
                                        Ok(Ok(mut result)) => {
                                            if let Some(config) = &context.gateway_config {
                                                let route = ClientRequest::try_from(JSONRPCRequest {id:RequestId::String(format!("desktop-routing-{}",uuid::Uuid::new_v4())),method:"thread/settings/update".into(),params:Some(json!({"threadId":result.thread.id,"gatewayConfig":config,"model":context.model})),trace:None});
                                                if let Ok(route) = route {
                                                    if !matches!(sender.request(route).await, Ok(Ok(_))) { return (id.clone(), method, rejection(id)); }
                                                } else { return (id.clone(), method, rejection(id)); }
                                            }
                                            result.thread.turns.clear();
                                            json!({"id":id,"result":result})
                                        }
                                        Ok(Err(error)) => json!({"id":id,"error":error}),
                                        Err(_) => rejection(id.clone()),
                                    }
                                } else {
                                    match sender.export_desktop_history(history_thread.unwrap_or_default()).await {
                                        Ok(Ok(replay)) => json!({"id":id,"result":ProtectedReplay{context,replay}}),
                                        Ok(Err(error)) => json!({"id":id,"error":error}),
                                        Err(_) => rejection(id.clone()),
                                    }
                                };
                                (id, method, result)
                            });
                        }
                        JSONRPCMessage::Notification(notification) => {
                            anyhow::ensure!(frame.len() <= FRAME_LIMIT, "Desktop notification exceeded its limit");
                            anyhow::ensure!(initialized && notification.method == "initialized", "Unsupported desktop notification");
                            sender.notify(ClientNotification::Initialized)?;
                        }
                        JSONRPCMessage::Response(response) => {
                            anyhow::ensure!(frame.len() <= FRAME_LIMIT, "Desktop reply exceeded its limit");
                            let kind = approvals.remove(&response.id).ok_or_else(||anyhow::anyhow!("Stale desktop approval or tool reply"))?;
                            if matches!(kind.0.as_str(),"item/commandExecution/requestApproval"|"item/fileChange/requestApproval") {
                                anyhow::ensure!(response.result.get("decision").and_then(Value::as_str).is_some_and(|decision| matches!(decision,"accept"|"decline"|"cancel")), "Unsupported desktop approval decision");
                            }
                            if kind.0 == "item/tool/call" {
                                let items = response.result.get("contentItems").and_then(Value::as_array).ok_or_else(||anyhow::anyhow!("Invalid desktop tool reply"))?;
                                anyhow::ensure!(items.len() <= 8 && response.result.get("success").is_some_and(Value::is_boolean), "Invalid desktop tool reply");
                                let mut size = 0usize;
                                for item in items {
                                    anyhow::ensure!(item.get("type").and_then(Value::as_str) == Some("inputText"), "Unavailable desktop tool output");
                                    let text = item.get("text").and_then(Value::as_str).ok_or_else(||anyhow::anyhow!("Invalid desktop tool reply"))?;
                                    size = size.saturating_add(text.len());
                                }
                                anyhow::ensure!(size <= 32 * 1024, "Desktop tool output exceeded its limit");
                            }
                            sender.respond_to_server_request(response.id, response.result)?;
                        }
                        JSONRPCMessage::Error(error) => {
                            anyhow::ensure!(frame.len() <= FRAME_LIMIT, "Desktop reply exceeded its limit");
                            anyhow::ensure!(approvals.remove(&error.id).is_some(), "Stale desktop approval or tool reply");
                            sender.fail_server_request(error.id, codex_app_server_protocol::JSONRPCErrorError {code:-32603,message:"Desktop tool or approval failed".into(),data:None})?;
                        }
                    }
                }
                Some((id, method, response)) = pending.next(), if !pending.is_empty() => {
                    request_ids.remove(&id);
                    if method == "initialize" && response.get("result").is_some() { initialized = true; }
                    if matches!(method.as_str(), "thread/start" | "desktop/history/restore") && let Some(thread) = response.get("result").and_then(|r| r.get("thread")).and_then(|t| t.get("id")).and_then(Value::as_str) {
                        anyhow::ensure!(threads.len() < 16, "Desktop thread limit exceeded");
                        threads.insert(thread.into());
                    }
                    write_json(&mut output, &response, &secret).await?;
                }
                event = runtime.next_event() => {
                    match event {
                        Some(InProcessServerEvent::ServerRequest(request)) => {
                            let value = serde_json::to_value(request)?;
                            let id: RequestId = serde_json::from_value(value["id"].clone())?;
                            let method = value["method"].as_str().unwrap_or_default();
                            let params = &value["params"];
                            let unavailable = match method {
                                "item/commandExecution/requestApproval" => params.get("additionalPermissions").is_some_and(|value| !value.is_null()) || params.get("networkApprovalContext").is_some_and(|value| !value.is_null()),
                                "item/fileChange/requestApproval" => context.local_access == super::LocalAccess::ReadOnly || params.get("grantRoot").is_some_and(|value| !value.is_null()),
                                "item/tool/call" => false,
                                _ => true,
                            };
                            if unavailable {
                                sender.fail_server_request(id, codex_app_server_protocol::JSONRPCErrorError{code:-32602,message:"Use native Desktop permission controls to change local access".into(),data:None})?;
                                continue;
                            }
                            anyhow::ensure!(approvals.len() < 32 && !approvals.contains_key(&id), "Desktop pending tool limit exceeded");
                            approvals.insert(id,(method.to_owned(), params["threadId"].as_str().unwrap_or_default().to_owned(), params["turnId"].as_str().unwrap_or_default().to_owned()));
                            write_json(&mut output, &value, &secret).await?;
                        }
                        Some(InProcessServerEvent::ServerNotification(notification)) => {
                            let value = serde_json::to_value(notification)?;
                            if value["method"] == "turn/completed" {
                                let thread = value["params"]["threadId"].as_str().unwrap_or_default();
                                let turn = value["params"]["turn"]["id"].as_str().unwrap_or_default();
                                approvals.retain(|_,(_,pending_thread,pending_turn)| pending_thread != thread || pending_turn != turn);
                            }
                            write_json(&mut output, &value, &secret).await?;
                        }
                        Some(InProcessServerEvent::Lagged{..}) => anyhow::bail!("Desktop event stream exceeded its capacity"),
                        None => break,
                    }
                }
            }
        }
        Ok(())
    }.await;
    let shutdown = runtime.shutdown().await;
    result?;
    shutdown.map_err(anyhow::Error::from)
}
