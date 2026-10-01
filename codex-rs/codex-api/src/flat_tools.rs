//! Adapt namespace tools at the HTTP boundary for AIRS gateway inference.
//! Canonical history and routing retain their namespace; only wire copies change.
use crate::common::ResponseEvent;
use crate::common::ResponseStream;
use crate::error::ApiError;
use codex_protocol::models::ResponseItem;
use serde_json::Value;
use std::collections::BTreeMap;
use tokio::sync::mpsc;

#[derive(Default)]
pub(crate) struct FlatTools {
    names: BTreeMap<String, (Option<String>, String)>,
}

impl FlatTools {
    pub(crate) fn prepare(body: &mut Value) -> Result<Self, ApiError> {
        let mut mapping = Self::default();
        if let Some(tools) = body.get_mut("tools").and_then(Value::as_array_mut) {
            let mut flat = Vec::new();
            for mut tool in std::mem::take(tools) {
                if tool["type"] == "namespace" {
                    let namespace = tool["name"].as_str().ok_or_else(invalid)?.to_owned();
                    let children = tool["tools"].as_array_mut().ok_or_else(invalid)?;
                    for mut child in std::mem::take(children) {
                        if child["type"] != "function" {
                            return Err(invalid());
                        }
                        let name = child["name"].as_str().ok_or_else(invalid)?;
                        child["name"] = Value::String(mapping.register(Some(&namespace), name)?);
                        flat.push(child);
                    }
                } else {
                    if let Some(name) = tool.get("name").and_then(Value::as_str) {
                        mapping.register(None, name)?;
                    }
                    flat.push(tool);
                }
            }
            *tools = flat;
        }
        if let Some(input) = body.get_mut("input").and_then(Value::as_array_mut) {
            for item in input {
                if item["type"] == "function_call" {
                    let namespace = item.get("namespace").and_then(Value::as_str);
                    let name = item["name"].as_str().ok_or_else(invalid)?;
                    let flat = mapping.register(namespace, name)?;
                    item["name"] = Value::String(flat);
                    item.as_object_mut()
                        .ok_or_else(invalid)?
                        .remove("namespace");
                }
            }
        }
        // The typed request currently uses "auto", but the raw client also accepts
        // an explicit function choice. Keep that choice consistent with the catalog.
        if let Some(choice) = body.get_mut("tool_choice")
            && choice["type"] == "function"
        {
            let flat = mapping.register(
                choice.get("namespace").and_then(Value::as_str),
                choice["name"].as_str().ok_or_else(invalid)?,
            )?;
            choice["name"] = Value::String(flat);
            choice
                .as_object_mut()
                .ok_or_else(invalid)?
                .remove("namespace");
        }
        Ok(mapping)
    }

    fn register(&mut self, namespace: Option<&str>, name: &str) -> Result<String, ApiError> {
        let flat = match namespace {
            Some(namespace) => format!("{namespace}__{name}"),
            None => name.to_owned(),
        };
        // Match the harness MCP name bound and fail closed on ambiguous aliases.
        if flat.is_empty()
            || flat.len() > 128
            || !flat
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        {
            return Err(invalid());
        }
        let canonical = (namespace.map(str::to_owned), name.to_owned());
        if let Some(previous) = self.names.insert(flat.clone(), canonical.clone())
            && previous != canonical
        {
            return Err(ApiError::Stream("gateway tool aliases collide".to_owned()));
        }
        Ok(flat)
    }

    fn restore_event(&self, event: &mut ResponseEvent) {
        if let ResponseEvent::OutputItemAdded(item) | ResponseEvent::OutputItemDone(item) = event
            && let ResponseItem::FunctionCall {
                namespace, name, ..
            } = item
            && namespace.is_none()
            && let Some((original_namespace, original_name)) = self.names.get(name)
        {
            *namespace = original_namespace.clone();
            *name = original_name.clone();
        }
    }

    pub(crate) fn restore_stream(self, mut stream: ResponseStream) -> ResponseStream {
        if !self
            .names
            .values()
            .any(|(namespace, _)| namespace.is_some())
        {
            return stream;
        }
        let (tx, rx_event) = mpsc::channel(32);
        let upstream_request_id = stream.upstream_request_id.take();
        let interrupt = stream.interrupt.take();
        tokio::spawn(async move {
            loop {
                let next = tokio::select! {
                    _ = tx.closed() => break,
                    next = stream.rx_event.recv() => next,
                };
                let Some(mut event) = next else { break };
                if let Ok(event) = &mut event {
                    self.restore_event(event);
                }
                if tx.send(event).await.is_err() {
                    break;
                }
            }
        });
        ResponseStream {
            rx_event,
            upstream_request_id,
            interrupt,
        }
    }
}

fn invalid() -> ApiError {
    ApiError::Stream(
        "gateway requires unambiguous function tools with names up to 128 ASCII characters"
            .to_owned(),
    )
}

#[cfg(test)]
#[path = "flat_tools_tests.rs"]
mod tests;
