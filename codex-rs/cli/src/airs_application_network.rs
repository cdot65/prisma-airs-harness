//! Application policy for short-lived AIRS authentication and diagnostic helpers.
//!
//! Each helper validates local managed requirements before network access. It does
//! not share the parent process's live controller or claim to govern user-launched
//! browser, shell, CLI or Jev SDK traffic.
use codex_config::LoaderOverrides;
use codex_config::NetworkDomainPermissionToml;
use codex_http_client::DestinationPolicy;
use codex_http_client::NetworkPolicy;
use codex_http_client::NetworkPolicyController;

pub(super) async fn load() -> anyhow::Result<NetworkPolicy> {
    load_with_overrides(&LoaderOverrides::default()).await
}

async fn load_with_overrides(overrides: &LoaderOverrides) -> anyhow::Result<NetworkPolicy> {
    let application = codex_config::loader::load_local_application_requirements(
        codex_exec_server::LOCAL_FS.as_ref(),
        overrides,
    )
    .await?
    .compose(Default::default())?;
    Ok(from_requirements(application.as_ref()))
}

fn from_requirements(
    application: Option<&codex_config::ApplicationRequirementsToml>,
) -> NetworkPolicy {
    let destination = match application.and_then(|application| application.network.as_ref()) {
        Some(network) if network.enabled => DestinationPolicy::Restricted {
            allowed_hosts: network
                .domains
                .iter()
                .filter(|&(_host, permission)| *permission == NetworkDomainPermissionToml::Allow)
                .map(|(host, _permission)| host.clone())
                .collect(),
        },
        Some(_) | None => DestinationPolicy::Unrestricted,
    };
    let controller = NetworkPolicyController::default();
    let policy = controller.policy();
    controller.publish(policy.revision(), destination);
    policy
}

/// Standalone MCP helpers retain the effective requirements of their loaded config.
/// An existing managed factory keeps its original live controller.
pub(super) fn for_config(
    config: &codex_core::config::Config,
) -> codex_http_client::HttpClientFactory {
    let factory = config.http_client_factory();
    if factory.network_policy().is_managed() {
        return factory;
    }
    factory.with_network_policy(from_requirements(
        config
            .config_layer_stack
            .requirements()
            .application
            .as_ref()
            .map(|requirements| &requirements.value),
    ))
}

#[cfg(test)]
#[path = "airs_application_network_tests.rs"]
mod tests;
