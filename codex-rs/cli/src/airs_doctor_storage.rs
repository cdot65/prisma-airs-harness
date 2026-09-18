//! Read-only native-service diagnostics isolated in a bounded child process.
use codex_keyring_store::KeyringStore;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;

const PROBE: &str = "AIRS_DOCTOR_STORAGE_PROBE";

pub(super) fn child_probe() -> anyhow::Result<bool> {
    if std::env::var_os(PROBE).as_deref() != Some(std::ffi::OsStr::new("1")) {
        return Ok(false);
    }
    // Never touch an actual binding or echo a native backend error/secret.
    let account = format!("doctor-probe-{}", uuid::Uuid::new_v4());
    let result =
        codex_keyring_store::DefaultKeyringStore.load(super::airs_credentials::SERVICE, &account);
    let (passed, detail) = match result {
        Ok(_) => (true, "Credential service responded. Access to the saved credential is not verified; choose Verify gateway access.".to_owned()),
        Err(error) => {
            let diagnostic = error.diagnostic();
            let recovery = if cfg!(target_os = "linux") {
                "Check your signed-in user's Secret Service and D-Bus session, then retry."
            } else {
                "Check native credential access from your signed-in desktop session, then retry."
            };
            (false, format!("Credential service unavailable in this session (category: {}; OS status: {}). {recovery} This does not establish that the store is locked.", diagnostic.kind.as_str(), diagnostic.native_code.map_or_else(|| "unavailable".into(), |code| code.to_string())))
        }
    };
    println!("{}", serde_json::to_string(&(passed, detail))?);
    Ok(true)
}

pub(super) async fn check(home: &Path) -> (bool, String) {
    let work = async {
        #[cfg(target_os = "linux")]
        let executable = std::path::PathBuf::from("/proc/self/exe");
        #[cfg(not(target_os = "linux"))]
        let executable = std::env::current_exe()?;
        let mut child = tokio::process::Command::new(executable)
            .args(["doctor", "--json"])
            .env("AIRS_HARNESS_HOME", home)
            .env(PROBE, "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;
        let mut bytes = Vec::new();
        child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("no probe output"))?
            .take(4097)
            .read_to_end(&mut bytes)
            .await?;
        anyhow::ensure!(bytes.len() <= 4096, "probe output too large");
        anyhow::ensure!(child.wait().await?.success(), "probe failed");
        Ok::<(bool, String), anyhow::Error>(serde_json::from_slice(&bytes)?)
    };
    match tokio::time::timeout(Duration::from_secs(5), work).await {
        Ok(Ok(result)) => result,
        _ => (false, "Credential service did not complete its bounded check. Retry from your signed-in user session; no credentials were changed.".into()),
    }
}

pub(super) fn authentication(home: &Path) -> (&'static str, bool) {
    use super::airs_credentials::Source;
    let binding = super::airs_status::public_file(&home.join("credential-binding.json"))
        .ok()
        .and_then(|value| super::airs_credentials::parse_binding(value.as_bytes()).ok());
    match binding.and_then(|binding| binding.source) {
        Some(Source::Oidc { .. }) => ("Company SSO", true),
        Some(Source::Keyring | Source::KeyringV2) => ("Workspace API key", true),
        Some(Source::Environment { .. } | Source::File { .. }) => {
            ("Workspace credential reference", false)
        }
        None if super::airs_status::configuration(home).is_ok_and(|(_, config)| {
            config
                .get("model_providers")
                .and_then(|v| v.get("airs"))
                .and_then(|v| v.get("env_http_headers"))
                .and_then(|v| v.get("x-portkey-api-key"))
                .is_some()
        }) =>
        {
            ("Workspace credential reference", false)
        }
        None => ("Not established; inspect credential configuration", false),
    }
}
