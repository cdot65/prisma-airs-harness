//! Reuse CLI credential transactions; pass secrets only over a private stdin pipe.
use super::Operation;
use std::path::Path;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

pub(crate) async fn run(
    home: &Path,
    operation: Operation,
    key: Option<String>,
) -> Result<String, String> {
    #[cfg(target_os = "linux")]
    let executable = std::path::PathBuf::from("/proc/self/exe");
    #[cfg(not(target_os = "linux"))]
    let executable =
        std::env::current_exe().map_err(|_| "Cannot locate the running AIRS executable.")?;
    run_executable(&executable, home, operation, key).await
}

async fn run_executable(
    executable: &Path,
    home: &Path,
    operation: Operation,
    key: Option<String>,
) -> Result<String, String> {
    let work = async {
        let mut command = tokio::process::Command::new(executable);
        command.args(["env", "typesafe"]);
        match operation {
            Operation::Status => {
                command.arg("status");
            }
            Operation::Set => {
                command.args(["set", "--stdin"]);
            }
            Operation::Clear => {
                command.arg("clear");
            }
        }
        let mut child = command
            .env("AIRS_HARNESS_HOME", home)
            .stdin(if operation == Operation::Set {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| "Could not start TypeSafe settings.")?;
        if operation == Operation::Set {
            let key = key.ok_or("No API key was supplied.")?;
            let mut stdin = child
                .stdin
                .take()
                .ok_or("Private credential input unavailable.")?;
            stdin
                .write_all(key.as_bytes())
                .await
                .map_err(|_| "Could not pass the API key to secure storage.")?;
            stdin
                .shutdown()
                .await
                .map_err(|_| "Could not finish private credential input.")?;
        }
        let mut bytes = Vec::new();
        child
            .stdout
            .take()
            .ok_or("TypeSafe status unavailable.")?
            .take(8193)
            .read_to_end(&mut bytes)
            .await
            .map_err(|_| "Could not read TypeSafe status.")?;
        if bytes.len() > 8192 {
            return Err("TypeSafe status exceeded its size limit.".into());
        }
        if !child
            .wait()
            .await
            .map_err(|_| "TypeSafe settings did not finish.")?
            .success()
        {
            return Err("Could not update TypeSafe settings. Unlock your native credential store, then retry /typesafe. Check /doctor if it remains unavailable.".into());
        }
        Ok(match operation {
            Operation::Status => crate::airs_doctor::display(&String::from_utf8_lossy(&bytes)),
            Operation::Set => "API key saved. The judge can use it now; no judgment request was sent. An inherited TYPESAFE_API_KEY takes precedence.".into(),
            Operation::Clear => "Saved API key removed. An inherited TYPESAFE_API_KEY still takes precedence.".into(),
        })
    };
    tokio::time::timeout(std::time::Duration::from_secs(30), work)
        .await
        .map_err(|_| {
            "Secure storage timed out. Reopen /typesafe to inspect the saved state before retrying."
                .to_owned()
        })?
}

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
