//! Full-fidelity RAM replay for the owned desktop host, outside the public RPC protocol.

use super::ThreadRequestProcessor;
use super::build_thread_from_loaded_snapshot;
use super::thread_response_active_permission_profile;
use crate::error_code::internal_error;
use crate::error_code::invalid_request;
use crate::outgoing_message::ConnectionId;
use crate::request_processors::build_legacy_api_turns_from_rollout_items;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::ThreadResumeResponse;
use codex_app_server_protocol::ThreadStatus;
use codex_config::types::HistoryPersistence;
use codex_core::StartThreadOptions;
use codex_core::config::Config;
use codex_core::config::ThreadStoreConfig;
use codex_protocol::ThreadId;
use codex_protocol::config_types::MultiAgentMode;
use codex_protocol::dynamic_tools::DynamicToolNamespaceTool;
use codex_protocol::dynamic_tools::DynamicToolSpec;
use codex_protocol::mcp::ClientMcpExtensions;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ContentItemKind;
use codex_protocol::models::InternalChatMessageMetadataPassthrough;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::SessionMeta;
use codex_protocol::protocol::ThreadHistoryMode;
use codex_protocol::protocol::ThreadMemoryMode;
use codex_rollout::InitialHistory;
use codex_rollout::ResumedHistory;
use codex_rollout::RolloutItem;
use codex_thread_store::CreateThreadParams;
use codex_thread_store::InMemoryThreadStore;
use codex_thread_store::LoadThreadHistoryParams;
use codex_thread_store::ThreadPersistenceMetadata;
use codex_thread_store::ThreadStore;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;

/// Exact source contract accepted by this owned desktop replay implementation.
pub const DESKTOP_REPLAY_ENGINE_REVISION: &str =
    "1885e40eb1dc493ad5b47757694d077011afa431+desktop-v1";
const MAX_REPLAY_BYTES: usize = 4 * 1024 * 1024;
const MAX_REPLAY_ITEMS: usize = 8192;
// Matches the inherited ordinary-item estimate (ceil(serialized bytes / 4) <= 10K).
// This is a conservative byte admission policy, not an exact provider tokenizer.
const MAX_CONTEXT_ITEM_BYTES: usize = 40_000;

/// Sensitive plaintext carried only in memory between the host and its encrypted store.
///
/// This includes harness provenance and compaction replacement records. Hosts must bind it to
/// their selected environment/capability bootstrap before restoring it and must never log it.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopReplay {
    pub schema_version: u32,
    pub engine_revision: String,
    pub create: CreateThreadParams,
    pub history: Vec<RolloutItem>,
}

/// The host supplies its current admitted tools separately from immutable historical records.
pub struct DesktopRestoreParams {
    pub replay: DesktopReplay,
    pub current_tools: Vec<DynamicToolSpec>,
}

impl ThreadRequestProcessor {
    fn desktop_ram_store(&self) -> Result<&InMemoryThreadStore, JSONRPCErrorError> {
        if !matches!(
            self.config.experimental_thread_store,
            ThreadStoreConfig::InMemory { .. }
        ) || self.config.ephemeral
            || self.config.history.persistence != HistoryPersistence::None
            || self.config.analytics_enabled != Some(false)
            || self.config.feedback_enabled
            || self.state_db.is_some()
            || self.log_db.is_some()
            || std::env::var_os("CODEX_ROLLOUT_TRACE_ROOT").is_some()
        {
            return Err(invalid_request(
                "desktop replay requires the protected RAM runtime",
            ));
        }
        self.thread_store
            .as_any()
            .downcast_ref::<InMemoryThreadStore>()
            .ok_or_else(|| invalid_request("desktop replay requires an in-memory thread store"))
    }

    pub(crate) async fn export_desktop_history(
        &self,
        thread_id: String,
    ) -> Result<DesktopReplay, JSONRPCErrorError> {
        let store = self.desktop_ram_store()?;
        let thread_id = ThreadId::from_string(&thread_id)
            .map_err(|_| invalid_request("invalid desktop thread id"))?;
        let thread = self
            .thread_manager
            .get_thread(thread_id)
            .await
            .map_err(|_| invalid_request("desktop thread is not loaded"))?;
        if matches!(
            self.thread_watch_manager
                .loaded_status_for_thread(&thread_id.to_string())
                .await,
            ThreadStatus::Active { .. }
        ) {
            return Err(invalid_request(
                "desktop history export requires an idle thread",
            ));
        }
        thread
            .flush_rollout()
            .await
            .map_err(|_| internal_error("failed to flush desktop RAM history"))?;
        let history = ThreadStore::load_history(
            store,
            LoadThreadHistoryParams {
                thread_id,
                include_archived: true,
            },
        )
        .await
        .map_err(|_| internal_error("failed to read desktop RAM history"))?
        .items;
        let meta = match history.first() {
            Some(RolloutItem::SessionMeta(line)) => &line.meta,
            _ => {
                return Err(invalid_request(
                    "desktop history has no canonical session metadata",
                ));
            }
        };
        let replay = DesktopReplay {
            schema_version: 1,
            engine_revision: DESKTOP_REPLAY_ENGINE_REVISION.to_string(),
            create: create_params(meta)?,
            history,
        };
        validate_replay(&replay, self.config.as_ref())?;
        Ok(replay)
    }

    pub(crate) async fn restore_desktop_history(
        &self,
        params: DesktopRestoreParams,
        connection_id: ConnectionId,
        client_mcp_extensions: ClientMcpExtensions,
    ) -> Result<ThreadResumeResponse, JSONRPCErrorError> {
        let DesktopRestoreParams {
            replay,
            current_tools,
        } = params;
        let store = self.desktop_ram_store()?;
        validate_replay(&replay, self.config.as_ref())?;
        let mut function_count = 0usize;
        let invalid_tool = current_tools.iter().any(|tool| match tool {
            DynamicToolSpec::Function(function) => {
                function_count = function_count.saturating_add(1);
                serde_json::to_vec(function).map_or(true, |bytes| bytes.len() > 4096)
            }
            DynamicToolSpec::Namespace(namespace) => {
                namespace.name.len() > 128
                    || namespace.description.len() > 4096
                    || serde_json::to_vec(namespace)
                        .map_or(true, |bytes| bytes.len() > MAX_CONTEXT_ITEM_BYTES)
                    || namespace.tools.iter().any(|tool| {
                        let DynamicToolNamespaceTool::Function(function) = tool;
                        function_count = function_count.saturating_add(1);
                        serde_json::to_vec(function).map_or(true, |bytes| bytes.len() > 4096)
                    })
            }
        });
        if current_tools.len() > 64
            || function_count > 64
            || invalid_tool
            || serde_json::to_vec(&current_tools).map_or(true, |bytes| bytes.len() > 48 * 1024)
        {
            return Err(invalid_request(
                "desktop current tool catalog exceeds its bounds",
            ));
        }
        // Core's empty list means "restore historical tools", so fail closed when that
        // would revive a catalog the host has intentionally removed.
        if current_tools.is_empty() && !replay.create.dynamic_tools.is_empty() {
            return Err(invalid_request(
                "desktop replay requires a current nonempty tool catalog",
            ));
        }
        let _permit = self.acquire_thread_list_state_permit().await?;
        let thread_id = replay.create.thread_id;
        if self.thread_manager.get_thread(thread_id).await.is_ok()
            || ThreadStore::read_thread(
                store,
                codex_thread_store::ReadThreadParams {
                    thread_id,
                    include_archived: true,
                    include_history: false,
                },
            )
            .await
            .is_ok()
        {
            return Err(invalid_request(
                "desktop thread is already present in this runtime",
            ));
        }
        ThreadStore::create_thread(store, replay.create)
            .await
            .map_err(|_| internal_error("failed to create desktop RAM history"))?;
        let history = Arc::new(replay.history);
        let restored = self
            .thread_manager
            .start_thread(StartThreadOptions {
                initial_history: InitialHistory::Resumed(ResumedHistory {
                    conversation_id: thread_id,
                    history: Arc::clone(&history),
                    rollout_path: None,
                }),
                // No selected workspace means native tools only, as at thread/start.
                environments: self
                    .config
                    .effective_workspace_roots()
                    .is_empty()
                    .then(Vec::new),
                dynamic_tools: current_tools,
                client_mcp_extensions,
                ..StartThreadOptions::new(self.config.as_ref().clone())
            })
            .await;
        let new_thread = match restored {
            Ok(thread) => thread,
            Err(_) => {
                let _ = ThreadStore::delete_thread(
                    store,
                    codex_thread_store::DeleteThreadParams { thread_id },
                )
                .await;
                return Err(internal_error("failed to restore desktop RAM history"));
            }
        };
        self.ensure_conversation_listener(
            thread_id,
            connection_id,
            /*raw_events_enabled*/ true,
        )
        .await?;
        self.thread_watch_manager
            .upsert_thread_silently(&thread_id.to_string())
            .await;
        let snapshot = new_thread.thread.config_snapshot().await;
        let mut thread =
            build_thread_from_loaded_snapshot(thread_id, &snapshot, &new_thread.thread);
        thread.status = ThreadStatus::Idle;
        thread.turns = build_legacy_api_turns_from_rollout_items(history.as_ref());
        let instruction_sources = new_thread.thread.legacy_instruction_sources().await;
        let sandbox = snapshot.sandbox_policy().into();
        let cwd = snapshot.cwd().clone();
        Ok(ThreadResumeResponse {
            gateway_config: snapshot.gateway_config,
            thread,
            model: snapshot.model,
            model_provider: snapshot.model_provider_id,
            service_tier: snapshot.service_tier,
            cwd,
            runtime_workspace_roots: snapshot.workspace_roots,
            instruction_sources,
            approval_policy: snapshot.approval_policy.into(),
            approvals_reviewer: snapshot.approvals_reviewer.into(),
            sandbox,
            active_permission_profile: thread_response_active_permission_profile(
                snapshot.active_permission_profile,
            ),
            reasoning_effort: snapshot.reasoning_effort,
            collaboration_mode: Some(snapshot.collaboration_mode),
            multi_agent_mode: MultiAgentMode::ExplicitRequestOnly,
            initial_turns_page: None,
            turns_backwards_cursor: None,
            items_backwards_cursor: None,
        })
    }
}

fn create_params(meta: &SessionMeta) -> Result<CreateThreadParams, JSONRPCErrorError> {
    Ok(CreateThreadParams {
        session_id: meta.session_id,
        thread_id: meta.id,
        extra_config: None,
        forked_from_id: meta.forked_from_id,
        parent_thread_id: meta.parent_thread_id,
        source: meta.source.clone(),
        thread_source: meta.thread_source.clone(),
        originator: meta.originator.clone(),
        base_instructions: meta
            .base_instructions
            .clone()
            .ok_or_else(|| invalid_request("desktop history is missing base instructions"))?,
        dynamic_tools: meta.dynamic_tools.clone().unwrap_or_default(),
        selected_capability_roots: meta.selected_capability_roots.clone(),
        multi_agent_version: meta.multi_agent_version,
        history_mode: meta.history_mode,
        history_base: meta.history_base,
        subagent_history_start_ordinal: meta.subagent_history_start_ordinal,
        initial_window_id: meta
            .context_window
            .as_ref()
            .ok_or_else(|| invalid_request("desktop history is missing its initial window"))?
            .window_id
            .clone(),
        metadata: ThreadPersistenceMetadata {
            cwd: Some(meta.cwd.clone()),
            model_provider: meta
                .model_provider
                .clone()
                .ok_or_else(|| invalid_request("desktop history is missing its provider"))?,
            memory_mode: if meta.memory_mode.as_deref() == Some("disabled") {
                ThreadMemoryMode::Disabled
            } else {
                ThreadMemoryMode::Enabled
            },
        },
    })
}

fn validate_replay(replay: &DesktopReplay, config: &Config) -> Result<(), JSONRPCErrorError> {
    if replay.schema_version != 1 || replay.engine_revision != DESKTOP_REPLAY_ENGINE_REVISION {
        return Err(invalid_request("desktop replay version is incompatible"));
    }
    if replay.history.is_empty()
        || replay.history.len() > MAX_REPLAY_ITEMS
        || serde_json::to_vec(replay)
            .map_err(|_| invalid_request("invalid desktop replay"))?
            .len()
            > MAX_REPLAY_BYTES
    {
        return Err(invalid_request(
            "desktop replay exceeds its bounded history contract",
        ));
    }
    let create = &replay.create;
    if create.parent_thread_id.is_some()
        || create.forked_from_id.is_some()
        || create.source.is_non_root_agent()
        || create.history_mode != ThreadHistoryMode::Legacy
        || create.history_base.is_some()
        || create.subagent_history_start_ordinal.is_some()
        || create.session_id != create.thread_id.into()
        || create.metadata.cwd.as_deref() != Some(config.cwd.as_path())
        || create.metadata.model_provider != config.model_provider_id
    {
        return Err(invalid_request(
            "desktop replay does not match the current root runtime",
        ));
    }
    let meta = match replay.history.first() {
        Some(RolloutItem::SessionMeta(line)) => &line.meta,
        _ => {
            return Err(invalid_request(
                "desktop replay has no canonical session metadata",
            ));
        }
    };
    if serde_json::to_value(create).ok() != serde_json::to_value(create_params(meta)?).ok()
        || replay
            .history
            .iter()
            .skip(1)
            .any(|item| matches!(item, RolloutItem::SessionMeta(_)))
    {
        return Err(invalid_request(
            "desktop replay session metadata is inconsistent",
        ));
    }
    let profile = config.permissions.effective_permission_profile();
    let workspace_roots = config.effective_workspace_roots();
    let approval = config.permissions.approval_policy.value();
    let mut active_turn = false;
    for item in &replay.history {
        validate_context_item(item)?;
        match item {
            RolloutItem::TurnContext(context)
                if context.cwd != config.cwd
                    || context.approval_policy != approval
                    || context.approvals_reviewer != Some(config.approvals_reviewer)
                    || context.permission_profile() != profile
                    || context.sandbox_policy != config.legacy_sandbox_policy()
                    || context.active_permission_profile
                        != config.permissions.active_permission_profile()
                    || context.workspace_roots.as_deref().unwrap_or_default()
                        != workspace_roots.as_slice()
                    || config
                        .model
                        .as_ref()
                        .is_some_and(|model| model != &context.model) =>
            {
                return Err(invalid_request(
                    "desktop replay turn settings do not match the current runtime",
                ));
            }
            RolloutItem::EventMsg(EventMsg::ThreadSettingsApplied(event))
                if event.thread_settings.cwd != config.cwd
                    || event.thread_settings.model_provider_id != config.model_provider_id
                    || event.thread_settings.approval_policy != approval
                    || event.thread_settings.approvals_reviewer != config.approvals_reviewer
                    || event.thread_settings.permission_profile != profile
                    || event.thread_settings.active_permission_profile
                        != config.permissions.active_permission_profile()
                    || config
                        .model
                        .as_ref()
                        .is_some_and(|model| model != &event.thread_settings.model) =>
            {
                return Err(invalid_request(
                    "desktop replay settings do not match the current runtime",
                ));
            }
            RolloutItem::EventMsg(EventMsg::TurnStarted(_)) => active_turn = true,
            RolloutItem::EventMsg(EventMsg::TurnComplete(_) | EventMsg::TurnAborted(_)) => {
                active_turn = false
            }
            _ => {}
        }
    }
    if active_turn {
        return Err(invalid_request("desktop replay ends in an incomplete turn"));
    }
    Ok(())
}

fn validate_context_item(item: &RolloutItem) -> Result<(), JSONRPCErrorError> {
    fn bounded(value: &impl Serialize) -> Result<(), JSONRPCErrorError> {
        if serde_json::to_vec(value).map_or(true, |bytes| bytes.len() > MAX_CONTEXT_ITEM_BYTES) {
            return Err(invalid_request(
                "desktop replay context item exceeds its serialized byte bound",
            ));
        }
        Ok(())
    }
    match item {
        RolloutItem::ResponseItem(response) => bounded(&response.item),
        RolloutItem::Compacted(compacted) => {
            // Legacy summaries become one assistant response; replacement records stay separate.
            bounded(&ResponseItem::Message {
                id: None,
                role: "user".into(),
                content: vec![ContentItem::InputText {
                    text: compacted.message.clone(),
                }],
                phase: None,
                internal_chat_message_metadata_passthrough: Some(
                    InternalChatMessageMetadataPassthrough {
                        content_item_kinds: Some(vec![ContentItemKind(
                            "compaction.summary".into(),
                        )]),
                        ..Default::default()
                    },
                ),
            })?;
            for response in compacted.replacement_history.iter().flatten() {
                bounded(&response.item)?;
            }
            for response in compacted
                .guardian_history
                .iter()
                .flat_map(|history| &history.0)
            {
                bounded(response)?;
            }
            bounded(&compacted.retained_context)?;
            bounded(&compacted.resume_metadata)
        }
        RolloutItem::SessionMeta(line) => bounded(&line.meta.base_instructions),
        RolloutItem::InterAgentCommunication(communication) => {
            bounded(&communication.to_model_input_item())
        }
        // These records can reconstruct later context fragments; bound the complete record.
        RolloutItem::TurnContext(_)
        | RolloutItem::WorldState(_)
        | RolloutItem::RetainedContext(_) => bounded(item),
        RolloutItem::InterAgentCommunicationMetadata { .. }
        | RolloutItem::TokenUsageRecord(_)
        | RolloutItem::SecurityRiskScore(_)
        | RolloutItem::EventMsg(_)
        | RolloutItem::RealtimeItem(_) => Ok(()),
    }
}
