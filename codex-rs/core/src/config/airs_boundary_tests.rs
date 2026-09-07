use super::*;

#[test]
fn rejects_destination_and_credential_override_but_allows_route_selection() {
    let home = tempfile::tempdir().unwrap();
    let input = "model = 'airs-gateway-default'\nmodel_provider = 'airs'\n[model_providers.airs]\nname = 'AIRS'\nbase_url = 'https://gateway.example/v1'\nwire_api = 'responses'\n";
    std::fs::write(home.path().join("config.toml"), input).unwrap();
    let original: ConfigToml = toml::from_str(input).unwrap();
    let mut effective = original.clone();
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
