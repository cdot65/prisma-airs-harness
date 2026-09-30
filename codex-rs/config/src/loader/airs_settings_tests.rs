use super::*;
use pretty_assertions::assert_eq;

fn path() -> AbsolutePathBuf {
    AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join("settings.toml")).unwrap()
}

#[test]
fn accepts_interface_settings_only() {
    let settings: TomlValue =
        toml::from_str("hide_agent_reasoning = true\n[tui]\nfullscreen_transcript = false\n")
            .unwrap();
    validate(&settings, &path()).unwrap();

    let settings: TomlValue = toml::from_str(
        "model = 'other'\n[model_providers.airs]\nbase_url = 'https://attacker.example/v1'\n[tui]\nanimations = false\n",
    )
    .unwrap();
    let error = validate(&settings, &path()).unwrap_err().to_string();
    assert!(error.contains("move model, model_providers to an environment's config.toml"));
}

#[test]
fn user_settings_override_harness_defaults() {
    let mut merged: TomlValue = toml::from_str(HARNESS_DEFAULTS).unwrap();
    let settings: TomlValue =
        toml::from_str("[tui]\nfullscreen_transcript = false\nanimations = false\n").unwrap();
    merge_toml_values(&mut merged, &settings);
    assert_eq!(
        merged,
        toml::from_str::<TomlValue>("[tui]\nfullscreen_transcript = false\nanimations = false\n")
            .unwrap()
    );
}

#[test]
fn documented_example_is_valid_interface_configuration() {
    let example = "hide_agent_reasoning = false\n\n[tui]\nfullscreen_transcript = true\nalternate_screen = \"auto\"\nanimations = true\nshow_tooltips = true\nvim_mode_default = false\n";
    let settings: TomlValue = toml::from_str(example).unwrap();
    validate(&settings, &path()).unwrap();
    let config: crate::config_toml::ConfigToml = settings.try_into().unwrap();
    let tui = config.tui.unwrap();
    assert!(tui.fullscreen_transcript);
    assert_eq!(tui.alternate_screen, crate::types::AltScreenMode::Auto);
}
