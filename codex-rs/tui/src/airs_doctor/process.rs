//! Call the exact running binary, pin its environment, and bound all output/work.
use super::Mode;
use super::Report;
use std::path::Path;
use std::process::Stdio;
use tokio::io::AsyncReadExt;

const MAX_OUTPUT: usize = 64 * 1024;

// Cancellation must also stop the doctor's bounded credential helper children.
#[cfg(unix)]
struct DiagnosticGroup(u32);

#[cfg(unix)]
impl Drop for DiagnosticGroup {
    fn drop(&mut self) {
        // SAFETY: the positive PID belongs to the dedicated process group we
        // created below. No shared terminal or harness process is in this group.
        unsafe {
            libc::kill(-(self.0 as libc::pid_t), libc::SIGKILL);
        }
    }
}

pub(super) fn parse(bytes: &[u8]) -> Result<Report, String> {
    let invalid =
        "Could not read diagnostics. Retry /doctor or run airs doctor in this environment.";
    if bytes.len() > MAX_OUTPUT {
        return Err(invalid.into());
    }
    let report: Report = serde_json::from_slice(bytes).map_err(|_| invalid.to_owned())?;
    if report.schema_version != 1
        || report.product != "Prisma AIRS Harness"
        || report.checks.len() > 32
    {
        return Err(invalid.into());
    }
    Ok(report)
}

pub(crate) async fn run(home: &Path, cwd: &Path, mode: Mode) -> Result<Report, String> {
    run_selected(home, cwd, mode, /*selection*/ None).await
}

pub(crate) async fn routing(
    home: &Path,
    cwd: &Path,
    selection: &crate::airs_routing::Selection,
) -> Result<Report, String> {
    run_selected(home, cwd, Mode::Verify, Some(selection)).await
}

async fn run_selected(
    home: &Path,
    cwd: &Path,
    mode: Mode,
    selection: Option<&crate::airs_routing::Selection>,
) -> Result<Report, String> {
    #[cfg(target_os = "linux")]
    let executable = std::path::PathBuf::from("/proc/self/exe");
    #[cfg(not(target_os = "linux"))]
    let executable =
        std::env::current_exe().map_err(|_| "Cannot locate the running AIRS executable.")?;
    let mut command = tokio::process::Command::new(executable);
    command.args(["doctor", "--json"]);
    if mode == Mode::Verify {
        command.arg("--verify-access");
    }
    if let Some(selection) = selection {
        command.arg("--routing-probe");
        if let Some(config) = &selection.saved_config {
            command.arg("--gateway-config").arg(config);
        }
        if let Some(model) = &selection.model {
            command.arg("--gateway-model").arg(model);
        }
    }
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command
        .env("AIRS_HARNESS_HOME", home)
        .env("AIRS_DOCTOR_CONNECTION_HEALTH", "1")
        .env_remove("AIRS_DOCTOR_STORAGE_PROBE")
        .env_remove("AIRS_DOCTOR_CATALOG_PROBE")
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "Could not start diagnostics.")?;
    #[cfg(unix)]
    let _group = DiagnosticGroup(child.id().ok_or("Diagnostics process unavailable.")?);
    let mut bytes = Vec::new();
    child
        .stdout
        .take()
        .ok_or("Diagnostics output unavailable.")?
        .take((MAX_OUTPUT + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| "Could not read diagnostics.")?;
    // Parse before waiting: oversized output must kill the child, never deadlock.
    let report = parse(&bytes)?;
    // A failed check deliberately returns nonzero with a valid, useful report.
    child
        .wait()
        .await
        .map_err(|_| "Diagnostics process did not finish.")?;
    Ok(report)
}
