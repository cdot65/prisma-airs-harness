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
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command
        .env("AIRS_HARNESS_HOME", home)
        .env("AIRS_DOCTOR_CONNECTION_HEALTH", "1")
        .env_remove("AIRS_DOCTOR_STORAGE_PROBE")
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
