//! Pin ordinary app-server requests to the selected desktop context.
use super::ContextBinding;
use crate::airs_desktop::LocalAccess;
use anyhow::Context;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::JSONRPCRequest;
use serde_json::Value;
use serde_json::json;
use std::collections::HashSet;

#[cfg(test)]
#[path = "airs_desktop_policy_tests.rs"]
mod tests;

pub(super) fn validate(
    request: &mut JSONRPCRequest,
    context: &ContextBinding,
    threads: &HashSet<String>,
) -> anyhow::Result<ClientRequest> {
    let params = request.params.get_or_insert_with(|| json!({}));
    let p = params
        .as_object_mut()
        .context("Invalid request parameters")?;
    let thread = p.get("threadId").and_then(Value::as_str);
    if request.method != "initialize" && request.method != "thread/start" {
        anyhow::ensure!(
            thread.is_some_and(|id| threads.contains(id)),
            "Unknown desktop thread"
        );
    }
    if request.method == "turn/start" {
        let input = p
            .get("input")
            .and_then(Value::as_array)
            .context("Missing turn input")?;
        anyhow::ensure!(
            !input.is_empty()
                && input.len() <= 8
                && input.iter().all(|item| {
                    item.get("type").and_then(Value::as_str) == Some("text")
                        && item
                            .get("text_elements")
                            .is_none_or(|value| value.as_array().is_some_and(Vec::is_empty))
                        && item
                            .get("text")
                            .and_then(Value::as_str)
                            .is_some_and(|text| text.len() <= 8192)
                }),
            "Desktop turn input must be bounded text"
        );
        anyhow::ensure!(
            input
                .iter()
                .filter_map(|item| item.get("text").and_then(Value::as_str))
                .map(str::len)
                .sum::<usize>()
                <= 32 * 1024,
            "Desktop turn text exceeds its total bound"
        );
        // Core combines text chunks into one user message. Include JSON escaping and reserve
        // room below the 40K replay item cap for harness-generated IDs and attribution metadata.
        let model_item = json!({"type":"message", "role":"user", "content":input.iter().map(|item| {
            json!({"type":"input_text", "text":item["text"]})
        }).collect::<Vec<_>>()});
        anyhow::ensure!(
            serde_json::to_vec(&model_item)?.len() <= 39_000,
            "Desktop turn context exceeds its serialized byte bound"
        );
    }
    match request.method.as_str() {
        "initialize" => {
            p.insert("capabilities".into(), json!({"experimentalApi":true}));
        }
        "thread/start" => {
            for field in [
                "config",
                "baseInstructions",
                "selectedCapabilityRoots",
                "runtimeWorkspaceRoots",
                "permissions",
                "serviceName",
                "projectId",
            ] {
                anyhow::ensure!(
                    p.get(field).is_none_or(Value::is_null),
                    "Desktop settings are immutable"
                );
            }
            let expected_sandbox = match context.local_access {
                LocalAccess::ReadOnly => "read-only",
                LocalAccess::WorkspaceWrite => "workspace-write",
            };
            for (field, expected) in [
                ("modelProvider", "airs"),
                ("approvalPolicy", "untrusted"),
                ("approvalsReviewer", "user"),
                ("sandbox", expected_sandbox),
            ] {
                // on-request from pinned clients is replaced by the safer command approval policy.
                anyhow::ensure!(
                    p.get(field).is_none_or(|value| value.is_null()
                        || value.as_str() == Some(expected)
                        || (field == "approvalPolicy" && value.as_str() == Some("on-request"))),
                    "Desktop settings are immutable"
                );
            }
            if let Some(cwd) = p.get("cwd").and_then(Value::as_str) {
                anyhow::ensure!(
                    context
                        .project_path
                        .as_ref()
                        .is_some_and(|path| path.to_str() == Some(cwd)),
                    "Desktop project is immutable"
                );
            }
            let model = context.model.as_deref().unwrap_or("airs-gateway-default");
            anyhow::ensure!(
                p.get("model")
                    .is_none_or(|value| value.is_null() || value.as_str() == Some(model)),
                "Desktop model is immutable"
            );
            anyhow::ensure!(
                p.get("environments").is_none_or(
                    |value| value.is_null() || value.as_array().is_some_and(Vec::is_empty)
                ),
                "Remote environments are unavailable"
            );
            if let Some(tools) = p.get("dynamicTools").and_then(Value::as_array) {
                let functions: Vec<&Value> = tools
                    .iter()
                    .flat_map(|tool| {
                        tool.get("tools")
                            .and_then(Value::as_array)
                            .map_or_else(|| vec![tool], |tools| tools.iter().collect())
                    })
                    .collect();
                anyhow::ensure!(
                    tools.len() <= 64
                        && functions.len() <= 64
                        && functions.iter().all(|tool| tool.to_string().len() <= 4096)
                        && tools.iter().all(|tool| {
                            tool.get("tools").is_none_or(|_| {
                                tool.get("name")
                                    .and_then(Value::as_str)
                                    .is_some_and(|name| name.len() <= 128)
                                    && tool
                                        .get("description")
                                        .and_then(Value::as_str)
                                        .is_none_or(|description| description.len() <= 4096)
                                    && tool.to_string().len() <= 40_000
                            })
                        })
                        && tools
                            .iter()
                            .map(|tool| tool.to_string().len())
                            .sum::<usize>()
                            <= 48 * 1024,
                    "Desktop tool catalog is too large"
                );
            }
            if let Some(instructions) = p.get("developerInstructions").and_then(Value::as_str) {
                anyhow::ensure!(
                    instructions.len() <= 4096,
                    "Desktop instructions are too large"
                );
            }
            p.remove("sandbox");
            p.insert(
                "runtimeWorkspaceRoots".into(),
                json!(context.project_path.iter().collect::<Vec<_>>()),
            );
            p.insert(
                "cwd".into(),
                json!(
                    context
                        .project_path
                        .as_ref()
                        .map_or_else(|| std::path::PathBuf::from("/"), Clone::clone)
                ),
            );
            p.insert("permissions".into(), json!("desktop-local"));
            p.insert("approvalPolicy".into(), json!("untrusted"));
            p.insert("approvalsReviewer".into(), json!("user"));
            p.insert("modelProvider".into(), json!("airs"));
            p.insert("model".into(), json!(model));
            // RAM persistence is the protected export source, never a local rollout backend.
            p.insert("ephemeral".into(), json!(false));
            if context.project_path.is_none() {
                p.insert("environments".into(), json!([]));
            }
        }
        "turn/start" => {
            for field in [
                "cwd",
                "runtimeWorkspaceRoots",
                "approvalPolicy",
                "approvalsReviewer",
                "sandboxPolicy",
                "permissions",
                "model",
                "collaborationMode",
                "multiAgentMode",
                "environments",
                "additionalContext",
                "toolOutput",
            ] {
                anyhow::ensure!(
                    p.get(field).is_none_or(Value::is_null),
                    "Desktop turn settings are immutable"
                );
            }
        }
        "thread/read" => {
            p.insert("includeTurns".into(), json!(false));
        }
        "turn/interrupt" => {}
        _ => anyhow::bail!("Method is unavailable in Desktop"),
    }
    ClientRequest::try_from(request.clone()).map_err(|_| anyhow::anyhow!("Invalid desktop request"))
}
