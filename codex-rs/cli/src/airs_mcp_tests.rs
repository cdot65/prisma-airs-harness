use super::*;
use pretty_assertions::assert_eq;

#[cfg(unix)]
#[test]
fn helper_command_quotes_paths_without_shell_expansion() {
    let input = "/tmp/user's dir/$(false)/`false`";
    let output = std::process::Command::new("sh")
        .args(["-c", &format!("printf %s {}", shell_quote(input))])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), input);
}

#[tokio::test]
async fn setup_rejects_insecure_or_credential_bearing_destinations() {
    let temp = tempfile::tempdir().unwrap();
    for url in [
        "http://example.com/mcp",
        "https://user:secret@example.com/mcp",
        "https://example.com/mcp?key=secret",
    ] {
        let args = SetupArgs {
            name: "scanner".into(),
            url: url.into(),
            credential_file: Some(temp.path().join("missing")),
            issuer_url: None,
            oidc_client_id: None,
            audience: None,
            device_auth: false,
            tool: Vec::new(),
            required: false,
        };
        let error = setup(temp.path(), &args).await.unwrap_err();
        assert!(error.to_string().contains("HTTPS without"));
        assert!(!temp.path().join("config.toml").exists());
    }
}
