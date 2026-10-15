//! Workspace MCP servers granted by the native Desktop host at bootstrap.
//!
//! Only public routing data (name, server id, URL) enters the runtime context
//! and generated configuration. Access tokens stay in process memory: they are
//! bound to one server name and URL in the model-provider credential registry.
use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use zeroize::Zeroize;
use zeroize::Zeroizing;

pub(super) const MAX_SERVERS: usize = 8;
const MAX_URL_BYTES: usize = 2048;
const MAX_TOKEN_BYTES: usize = 16 * 1024;
const STARTUP_TIMEOUT_SECONDS: i64 = 30;
const TOOL_TIMEOUT_SECONDS: i64 = 120;
/// Native host tools and the host-owned apps registration keep their meaning.
const RESERVED_NAMES: [&str; 2] = ["airs_desktop", "codex_apps"];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct McpServerGrant {
    name: String,
    server_id: String,
    url: String,
    access_token: String,
}

impl Drop for McpServerGrant {
    fn drop(&mut self) {
        self.access_token.zeroize();
    }
}

impl McpServerGrant {
    pub(super) fn access_token(&self) -> &str {
        &self.access_token
    }
}

/// Public, replay-bound identity of one granted MCP server.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct McpServerBinding {
    pub(crate) name: String,
    pub(crate) server_id: String,
    pub(crate) url: String,
}

fn valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|first| first.is_ascii_lowercase())
        && name.len() <= 32
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && !RESERVED_NAMES.contains(&name)
}

fn valid_url(input: &str, allow_http_loopback: super::HttpLoopback) -> anyhow::Result<String> {
    anyhow::ensure!(
        !input.is_empty()
            && input.len() <= MAX_URL_BYTES
            && input.bytes().all(|byte| byte.is_ascii_graphic()),
        "Invalid MCP server URL"
    );
    let url = url::Url::parse(input).context("Invalid MCP server URL")?;
    anyhow::ensure!(
        url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
            && url.host().is_some(),
        "MCP server URL must not contain credentials, a query or a fragment"
    );
    let loopback = match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain(host)) => host == "localhost",
        None => false,
    };
    anyhow::ensure!(
        url.scheme() == "https"
            || (allow_http_loopback == super::HttpLoopback::Allowed
                && loopback
                && url.scheme() == "http"),
        "MCP server URL must use HTTPS"
    );
    let url = url.to_string();
    anyhow::ensure!(url.len() <= MAX_URL_BYTES, "Invalid MCP server URL");
    Ok(url)
}

/// Validates every grant; any violation fails the entire bootstrap.
pub(super) fn validate(
    grants: &[McpServerGrant],
    allow_http_loopback: super::HttpLoopback,
) -> anyhow::Result<Vec<McpServerBinding>> {
    anyhow::ensure!(grants.len() <= MAX_SERVERS, "Too many Desktop MCP servers");
    let mut names = HashSet::new();
    grants
        .iter()
        .map(|grant| {
            anyhow::ensure!(
                valid_name(&grant.name) && names.insert(grant.name.as_str()),
                "Invalid Desktop MCP server name"
            );
            anyhow::ensure!(
                (1..=64).contains(&grant.server_id.len())
                    && grant
                        .server_id
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'),
                "Invalid Desktop MCP server id"
            );
            // Bearer tokens are visible ASCII; this excludes whitespace and controls.
            anyhow::ensure!(
                !grant.access_token.is_empty()
                    && grant.access_token.len() <= MAX_TOKEN_BYTES
                    && grant
                        .access_token
                        .bytes()
                        .all(|byte| byte.is_ascii_graphic()),
                "Invalid Desktop MCP credential"
            );
            Ok(McpServerBinding {
                name: grant.name.clone(),
                server_id: grant.server_id.clone(),
                url: valid_url(&grant.url, allow_http_loopback)?,
            })
        })
        .collect()
}

/// Streamable HTTP registrations without any credential material.
pub(super) fn config(bindings: &[McpServerBinding]) -> anyhow::Result<toml::Value> {
    let servers = bindings
        .iter()
        .map(|binding| {
            (
                binding.name.clone(),
                serde_json::json!({
                    "url": binding.url,
                    "enabled": true,
                    // An unavailable workspace server never blocks inference.
                    "required": false,
                    "startup_timeout_sec": STARTUP_TIMEOUT_SECONDS,
                    "tool_timeout_sec": TOOL_TIMEOUT_SECONDS,
                    // Every tool call asks the native operator.
                    "default_tools_approval_mode": "prompt",
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    Ok(serde_json::from_value(serde_json::Value::Object(servers))?)
}

/// Memory-only credentials bound to the validated name and canonical URL.
pub(super) fn credentials(
    grants: &[McpServerGrant],
    bindings: &[McpServerBinding],
) -> anyhow::Result<Vec<codex_model_provider::DesktopMcpCredential>> {
    grants
        .iter()
        .zip(bindings)
        .map(|(grant, binding)| {
            let authorization = Zeroizing::new(format!("Bearer {}", grant.access_token));
            Ok(codex_model_provider::DesktopMcpCredential {
                server_name: binding.name.clone(),
                url: binding.url.clone(),
                authorization: http::HeaderValue::from_str(&authorization)
                    .context("Invalid Desktop MCP credential")?,
            })
        })
        .collect()
}
