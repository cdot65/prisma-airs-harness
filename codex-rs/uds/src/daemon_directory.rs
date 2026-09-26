//! The host-local rendezvous root for privileged app-server RPC sockets.
//!
//! Upstream privileged listeners reserve this root. Sandboxes can hide it even
//! when AIRS does not run a daemon. It must not depend on HOME, TMPDIR,
//! CODEX_HOME, or command settings.

use std::fs;
use std::io;
use std::path::PathBuf;

/// Returns the fixed executor-local directory that every sandbox must hide.
pub fn shared_daemon_socket_directory() -> io::Result<PathBuf> {
    // Resolve the system alias /tmp -> /private/tmp on macOS.
    let temporary_root = fs::canonicalize("/tmp")?;
    // SAFETY: geteuid has no arguments or side effects.
    let uid = unsafe { libc::geteuid() };
    Ok(temporary_root.join(format!("codex-daemon-{uid}")))
}
