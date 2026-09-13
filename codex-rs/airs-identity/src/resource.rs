use super::IdentityConfig;

pub(crate) fn validate(
    metadata: &serde_json::Value,
    config: &IdentityConfig,
) -> anyhow::Result<()> {
    let issuers = metadata
        .get("authorization_servers")
        .and_then(serde_json::Value::as_array);
    let scopes = metadata
        .get("scopes_supported")
        .and_then(serde_json::Value::as_array);
    anyhow::ensure!(
        metadata.get("resource").and_then(serde_json::Value::as_str) == config.resource.as_deref()
            && issuers.is_some_and(|values| values
                .iter()
                .any(|v| v.as_str() == Some(config.issuer.as_str())))
            && config.scopes.iter().all(|scope| scopes
                .is_some_and(|values| values.iter().any(|v| v.as_str() == Some(scope.as_str())))),
        "MCP resource metadata does not match the configured resource, issuer and scopes"
    );
    Ok(())
}

#[cfg(test)]
#[path = "resource_tests.rs"]
mod tests;
