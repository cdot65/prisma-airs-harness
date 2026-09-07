use super::*;

#[test]
fn runtime_context_describes_enabled_mcp_without_credentials() {
    let servers = toml::from_str(
        "[security]\nurl = 'https://private.invalid/mcp'\nrequired = true\nenabled_tools = ['pan_inline_scan']\n[security.http_headers]\nAuthorization = 'private-test-token'\n[disabled]\nurl = 'https://disabled.invalid/mcp'\nenabled = false\n",
    )
    .unwrap();
    let context = developer_instructions(Some("Keep the user's instructions.".into()), &servers);
    assert!(context.starts_with("Keep the user's instructions.\n\n"));
    assert!(context.contains(
        "\"security\":{\"required_at_startup\":true,\"tool_allowlist\":[\"pan_inline_scan\"]}"
    ));
    assert!(!context.contains("private.invalid"));
    assert!(!context.contains("private-test-token"));
    assert!(!context.contains("disabled.invalid"));
    let oversized = (0..100)
        .map(|i| (format!("server-{i:03}"), servers["security"].clone()))
        .collect();
    let bounded = developer_instructions(/*existing*/ None, &oversized);
    assert!(bounded.contains("server-000"));
    assert!(!bounded.contains("server-004"));
    assert!(bounded.len() < 6000);
}

#[test]
fn rejects_destination_and_credential_override_but_allows_route_selection() {
    let home = tempfile::tempdir().unwrap();
    let input = "model = 'airs-gateway-default'\nmodel_provider = 'airs'\n[model_providers.airs]\nname = 'AIRS'\nbase_url = 'https://gateway.example/v1'\nwire_api = 'responses'\n";
    std::fs::write(home.path().join("config.toml"), input).unwrap();
    let original: ConfigToml = toml::from_str(input).unwrap();
    let mut effective = original.clone();
    effective.model_context_window = Some(2_000_000);
    assert!(validate(home.path(), &effective).is_err());
    effective = original.clone();
    effective.model = Some("@provider/other".into());
    assert!(validate(home.path(), &effective).is_ok());
    effective.model_providers.get_mut("airs").unwrap().base_url =
        Some("https://other.example/v1".into());
    assert!(validate(home.path(), &effective).is_err());
    assert!(validate_provider(&original, &effective.model_providers["airs"]).is_err());
    effective = original;
    effective.model_providers.get_mut("airs").unwrap().env_key = Some("OTHER_KEY".into());
    assert!(validate(home.path(), &effective).is_err());
}
