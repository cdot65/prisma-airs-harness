//! Shareable snapshots contain only known fields, never diagnostic detail strings.
use super::Report;
use codex_protocol::ThreadId;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

const MAX_REPORT: usize = 8192;

#[derive(Clone, Copy)]
pub(crate) enum Action {
    Open,
    Preview,
    Copy,
    Save,
    Close,
}

/// A report action is valid only while its originating view and thread survive.
pub(crate) struct Session {
    pub(crate) text: Arc<str>,
    thread: Option<ThreadId>,
    active: AtomicBool,
}

impl Session {
    pub(crate) fn new(text: Arc<str>, thread: Option<ThreadId>) -> Arc<Self> {
        Arc::new(Self {
            text,
            thread,
            active: AtomicBool::new(true),
        })
    }

    pub(crate) fn allows(&self, thread: Option<ThreadId>) -> bool {
        thread.is_some() && self.thread == thread && self.active.load(Ordering::Acquire)
    }

    pub(crate) fn invalidate(&self) {
        self.active.store(false, Ordering::Release);
    }
}

pub(crate) fn render(report: Option<&Report>) -> Arc<str> {
    let authentication = match report.map(|report| report.authentication.as_str()) {
        Some("Company SSO") => "Company SSO",
        Some("Workspace API key") => "Workspace API key",
        Some("Workspace credential reference") => "Workspace credential reference",
        Some("Not established; inspect credential configuration") => "Not established",
        _ => "Unknown",
    };
    let mut text = format!(
        "AIRS diagnostic report v1\nPrisma AIRS Harness {}\nPlatform: {} / {}\nAuthentication: {authentication}\n\nSnapshot of the last /doctor check; no new requests were made.\nNames, addresses, paths, raw details and conversation content are omitted.\n\n",
        codex_utils_home_dir::AIRS_HARNESS_VERSION,
        std::env::consts::OS,
        std::env::consts::ARCH,
    );
    // All labels and recovery instructions are owned strings. Even an expected
    // check can carry private data in its detail; no detail is ever copied.
    for (name, recovery) in [
        (
            "credential_service",
            "Check your signed-in user's native credential service. A failed check does not establish that the store is locked.",
        ),
        (
            "credential_cleanup",
            "Restore native storage access, then run airs --environment NAME login or logout. NAME is your local environment.",
        ),
        (
            "local_tools",
            "Run airs doctor locally to identify missing executables.",
        ),
        (
            "linux_user_namespaces",
            "Check host/kernel/container namespace policy. No unsandboxed fallback.",
        ),
        (
            "configuration",
            "Run airs env list and select a configured local environment.",
        ),
        (
            "credential_configuration",
            "Run airs --environment NAME login for your local environment.",
        ),
        (
            "capabilities",
            "Run airs doctor locally to inspect model capability configuration.",
        ),
        (
            "gateway_health",
            "Check gateway DNS, TLS and the API-root health endpoint.",
        ),
        (
            "mcp_configuration",
            "Use /mcp to inspect connections and authorize tools.",
        ),
        (
            "gateway_access",
            "Choose Verify gateway access in /doctor, or run airs doctor --verify-access in the same environment.",
        ),
    ] {
        let mut matching = report
            .into_iter()
            .flat_map(|report| &report.checks)
            .filter(|check| check.name == name)
            .peekable();
        if matching.peek().is_none() {
            if name == "gateway_access" {
                text.push_str("gateway_access: Not verified\n");
            }
            continue;
        }
        let passed = matching.all(|check| check.passed);
        text.push_str(&format!(
            "{name}: {}\n",
            if passed { "PASS" } else { "Needs attention" }
        ));
        if !passed {
            text.push_str(&format!("  Recovery: {recovery}\n"));
        }
    }
    if report.is_none() {
        text.push_str("Diagnostics unavailable. Retry /doctor.\n");
    }
    text.push_str("\nCredential-service health does not verify a saved credential.\nGateway health does not verify inference access. MCP discovery does not verify a completed tool call.\nNo credentials, configuration, raw logs or tool results are included.\n");
    debug_assert!(text.len() <= MAX_REPORT);
    text.into()
}

pub(crate) fn save(home: &Path, text: &str) -> Result<PathBuf, &'static str> {
    let failure =
        "Could not save the report. Preview it or retry in a writable environment directory.";
    if text.len() > MAX_REPORT {
        return Err(failure);
    }
    // NamedTempFile creates a new unpredictable file exclusively (0600 on Unix),
    // never follows a preexisting filename, and removes partial writes on error.
    let mut file = tempfile::Builder::new()
        .prefix("diagnostic-report-")
        .suffix(".txt")
        .tempfile_in(home)
        .map_err(|_| failure)?;
    file.write_all(text.as_bytes()).map_err(|_| failure)?;
    file.flush().map_err(|_| failure)?;
    file.keep().map(|(_, path)| path).map_err(|_| failure)
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
