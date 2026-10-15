/// Identifies privileged Codex approvals. Changing this key or adding another
/// privileged discriminator requires updating the MCP elicitation router's
/// form-forwarding safeguards.
pub const APPROVAL_KIND_KEY: &str = "codex_approval_kind";
pub const APPROVAL_KIND_MCP_TOOL_CALL: &str = "mcp_tool_call";
pub const APPROVAL_KIND_TOOL_SUGGESTION: &str = "tool_suggestion";
/// Reserved for metadata that core alone adds for the AIRS Desktop runtime.
/// The MCP elicitation router removes this namespace from every
/// server-originated request, so its presence proves a core origin.
pub const AIRS_DESKTOP_META_PREFIX: &str = "airs_desktop_";
/// The `mcpToolCall` thread item a core MCP tool approval belongs to.
pub const AIRS_DESKTOP_ITEM_ID_KEY: &str = "airs_desktop_item_id";
pub const REQUEST_TYPE_KEY: &str = "codex_request_type";
pub const REQUEST_TYPE_APPROVAL_REQUEST: &str = "approval_request";
pub const STRICT_AUTO_REVIEW_KEY: &str = "codex_strict_auto_review";
/// Marks a sensitive action requiring synchronous review when routed to automatic review.
/// Absent or false preserves the existing approval path; this does not opt into review.
pub const SENSITIVE_ACTION_KEY: &str = "codex_sensitive_action";
pub const APPROVALS_REVIEWER_KEY: &str = "approvals_reviewer";
pub const PERSIST_KEY: &str = "persist";
pub const PERSIST_SESSION: &str = "session";
pub const PERSIST_ALWAYS: &str = "always";
pub const SOURCE_KEY: &str = "source";
pub const SOURCE_CONNECTOR: &str = "connector";
pub const CONNECTOR_ID_KEY: &str = "connector_id";
pub const CONNECTOR_NAME_KEY: &str = "connector_name";
pub const CONNECTOR_DESCRIPTION_KEY: &str = "connector_description";
pub const TOOL_NAME_KEY: &str = "tool_name";
pub const TOOL_TITLE_KEY: &str = "tool_title";
pub const TOOL_DESCRIPTION_KEY: &str = "tool_description";
pub const TOOL_PARAMS_KEY: &str = "tool_params";
pub const TOOL_PARAMS_DISPLAY_KEY: &str = "tool_params_display";
