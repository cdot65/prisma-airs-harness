//! Isolate catalog filesystem reads so a stalled mount cannot hold up doctor.
//!
//! Adapted from upstream doctor filesystem probes (b33199b1fb). The deadline
//! bounds the helper wait, not synchronous OS process creation. A helper stuck
//! in an uninterruptible filesystem call is killed without awaiting its exit.
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

const PROBE: &str = "AIRS_DOCTOR_CATALOG_PROBE";
const BUDGET: Duration = Duration::from_secs(2);

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    Valid,
    Invalid,
    TimedOut,
    Unavailable,
}

fn valid_catalog(path: &Path) -> bool {
    super::airs_status::public_file(path)
        .ok()
        .and_then(|contents| serde_json::from_str::<serde_json::Value>(&contents).ok())
        .is_some()
}

/// Internal helper: no credential reads, network requests or catalog contents on stdout.
pub(super) fn child_probe() -> bool {
    let Some(path) = std::env::var_os(PROBE) else {
        return false;
    };
    std::process::exit(if valid_catalog(Path::new(&path)) {
        0
    } else {
        2
    });
}

async fn probe(command: &mut Command, budget: Duration) -> Outcome {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let Ok(mut child) = command.spawn() else {
        return Outcome::Unavailable;
    };
    match tokio::time::timeout(budget, child.wait()).await {
        Ok(Ok(status)) => match status.code() {
            Some(0) => Outcome::Valid,
            Some(2) => Outcome::Invalid,
            _ => Outcome::Unavailable,
        },
        Ok(Err(_)) => Outcome::Unavailable,
        Err(_) => {
            let _ = child.start_kill();
            Outcome::TimedOut
        }
    }
}

pub(super) async fn check(home: &Path, path: &Path) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    let executable = std::path::PathBuf::from("/proc/self/exe");
    #[cfg(not(target_os = "linux"))]
    let executable = std::env::current_exe()
        .map_err(|_| anyhow::anyhow!("Cannot locate the catalog diagnostic helper"))?;
    let mut command = Command::new(executable);
    command
        .args(["doctor", "--json"])
        .env("AIRS_HARNESS_HOME", home)
        .env_remove("AIRS_DOCTOR_STORAGE_PROBE")
        .env(PROBE, path);
    match probe(&mut command, BUDGET).await {
        Outcome::Valid => Ok(()),
        Outcome::Invalid => anyhow::bail!(
            "Capability catalog must be readable, regular UTF-8 JSON of at most 1 MiB"
        ),
        Outcome::TimedOut => anyhow::bail!(
            "Capability catalog check timed out. Check its filesystem or mount, then retry doctor; no configuration was changed."
        ),
        Outcome::Unavailable => anyhow::bail!(
            "Capability catalog check could not complete. Retry doctor from this environment; no configuration was changed."
        ),
    }
}

#[cfg(test)]
#[path = "airs_doctor_catalog_tests.rs"]
mod tests;
