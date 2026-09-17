use anyhow::Result;
use predicates::str::contains;
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::path::Path;

fn command(home: &Path) -> Result<assert_cmd::Command> {
    let mut command = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("airs-harness")?);
    command.env("AIRS_HARNESS_HOME", home);
    // Status checks availability locally; no test sends this synthetic key to a gateway.
    command.env("AIRS_API_KEY", "synthetic-environment-lifecycle-key");
    Ok(command)
}

fn registry(home: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&std::fs::read(
        home.join("environments.json"),
    )?)?)
}

#[test]
fn product_commands_never_start_an_agent_or_select_a_gateway_environment() -> Result<()> {
    let root = tempfile::tempdir()?;
    for product in [
        "runtime",
        "redteam",
        "aigateway",
        "model-security",
        "agentguard",
        "tenant",
    ] {
        command(root.path())?
            .args([product, "list"])
            .assert()
            .failure()
            .stderr(contains("Product commands moved to 'airs cli"));
    }
    command(root.path())?
        .args(["--environment", "unconfigured", "cli", "doctor"])
        .assert()
        .failure()
        .stderr(contains("--environment do not select a product tenant"));
    assert!(!root.path().join("environments.json").exists());
    Ok(())
}

#[test]
fn environment_lifecycle_preserves_state_and_isolates_recreated_names() -> Result<()> {
    let root = tempfile::tempdir()?;
    let home = root.path();
    command(home)?
        .arg("env")
        .assert()
        .success()
        .stdout(contains("env create"));
    command(home)?
        .args([
            "env",
            "create",
            "work",
            "--gateway-url",
            "https://gateway.example/v1",
        ])
        .assert()
        .success()
        .stdout(contains("--environment work login"));
    let created = registry(home)?;
    let id = created["environments"]["work"]["id"].as_str().unwrap();
    let state = home.join("environments").join(id);
    let config = std::fs::read(state.join("config.toml"))?;
    std::fs::write(state.join("history.jsonl"), b"preserved session")?;
    // Use a synthetic credential marker, never a real OS credential.
    std::fs::write(state.join("credential-marker"), b"preserved binding")?;

    command(home)?
        .args(["env", "show"])
        .assert()
        .success()
        .stdout(contains(id));
    command(home)?
        .args(["env", "status", "work"])
        .assert()
        .success()
        .stdout(contains("https://gateway.example/v1"));
    command(home)?
        .args(["env", "rename", "work", "team"])
        .assert()
        .success();
    let renamed = registry(home)?;
    assert_eq!(renamed["active"], "team");
    assert_eq!(
        renamed["environments"]["team"],
        created["environments"]["work"]
    );
    command(home)?
        .args(["env", "show", "work"])
        .assert()
        .failure();

    command(home)?
        .args([
            "env",
            "create",
            "other",
            "--gateway-url",
            "https://other.example/v1",
        ])
        .assert()
        .success();
    command(home)?
        .args(["--environment", "team", "env", "show"])
        .assert()
        .success()
        .stdout(contains(id));
    command(home)?
        .args(["env", "status", "team"])
        .assert()
        .success()
        .stdout(contains("https://gateway.example/v1"));
    command(home)?
        .args(["--environment", "team", "env", "status"])
        .assert()
        .success()
        .stdout(contains("https://gateway.example/v1"));
    assert_eq!(registry(home)?["active"], "other");
    command(home)?
        .args(["env", "use", "team"])
        .assert()
        .success();
    let before = std::fs::read(home.join("environments.json"))?;
    for args in [
        vec!["env", "rename", "team", "other"],
        vec!["env", "rename", "team", "bad/name"],
        vec!["env", "rename", "missing", "new"],
        vec![
            "env",
            "create",
            "team",
            "--gateway-url",
            "https://changed.example/v1",
        ],
    ] {
        command(home)?.args(args).assert().failure();
        assert_eq!(std::fs::read(home.join("environments.json"))?, before);
    }
    command(home)?
        .args(["env", "remove", "team"])
        .assert()
        .success();
    assert_eq!(registry(home)?["active"], Value::Null);
    assert_eq!(std::fs::read(state.join("config.toml"))?, config);
    assert_eq!(
        std::fs::read(state.join("history.jsonl"))?,
        b"preserved session"
    );
    assert_eq!(
        std::fs::read(state.join("credential-marker"))?,
        b"preserved binding"
    );
    command(home)?
        .args([
            "env",
            "create",
            "team",
            "--gateway-url",
            "https://gateway.example/v1",
        ])
        .assert()
        .success();
    assert_ne!(registry(home)?["environments"]["team"]["id"], id);
    let active = registry(home)?["active"].clone();
    command(home)?
        .args(["env", "rename", "other", "secondary"])
        .assert()
        .success();
    assert_eq!(registry(home)?["active"], active);
    Ok(())
}

#[test]
fn creation_errors_do_not_register_an_environment() -> Result<()> {
    let root = tempfile::tempdir()?;
    for args in [
        vec!["env", "create"],
        vec![
            "env",
            "create",
            "--gateway-url",
            "https://gateway.example/v1",
        ],
        vec![
            "env",
            "create",
            "work",
            "--gateway-url",
            "http://gateway.example/v1",
        ],
        vec![
            "--environment",
            "other",
            "env",
            "create",
            "work",
            "--gateway-url",
            "https://gateway.example/v1",
        ],
    ] {
        command(root.path())?.args(args).assert().failure();
        assert!(!root.path().join("environments.json").exists());
        assert!(!root.path().join("config.toml").exists());
    }
    Ok(())
}
