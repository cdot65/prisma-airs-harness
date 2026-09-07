//! Pin the destination, credential identity and capabilities of an environment's history.
//!
//! Upstream rollouts already persist routing/model selections. This immutable
//! environment revision protects that entire rollout namespace before it is loaded.
use super::airs_credentials;
use super::airs_environment;
use anyhow::Context;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    schema_version: u32,
    gateway_url: String,
    credential_identity: String,
    capability_revision: String,
    context_window: i64,
    #[serde(default)]
    mcp_config_revision: Option<String>,
}

pub fn validate(home: &Path) -> anyhow::Result<()> {
    let _lock = airs_environment::lock(home)?;
    let config: toml::Value = toml::from_str(
        &std::fs::read_to_string(home.join("config.toml"))
            .context("environment is not configured; run airs-harness setup")?,
    )?;
    let catalog = config
        .get("model_catalog_json")
        .and_then(toml::Value::as_str)
        .context("environment has no capability catalog; run setup")?;
    let revision = Revision {
        schema_version: 1,
        gateway_url: airs_environment::gateway(home)?,
        credential_identity: airs_credentials::identity(home)?,
        capability_revision: airs_credentials::fingerprint(&std::fs::read_to_string(catalog)?),
        context_window: config
            .get("model_context_window")
            .and_then(toml::Value::as_integer)
            .context("environment has no context window")?,
        mcp_config_revision: config
            .get("mcp_servers")
            .filter(|servers| {
                servers
                    .as_table()
                    .is_some_and(|servers| !servers.is_empty())
            })
            .map(toml::to_string)
            .transpose()?
            .map(|servers| airs_credentials::fingerprint(&servers)),
    };
    let path = home.join("session-binding.json");
    if path.exists() {
        let previous: Revision = serde_json::from_slice(&std::fs::read(path)?)?;
        anyhow::ensure!(
            previous == revision,
            "session environment revision changed (gateway, credential, capabilities or MCP bindings); create a new environment to keep existing history bound to its original configuration"
        );
    } else {
        airs_environment::atomic_write(&path, &serde_json::to_vec_pretty(&revision)?)?;
    }
    Ok(())
}
