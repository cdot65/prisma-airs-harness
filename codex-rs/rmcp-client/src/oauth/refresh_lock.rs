//! Cross-process serialization for one MCP OAuth credential's refresh transaction.
//!
//! The guard is intentionally acquired before the authoritative credential reread and retained
//! through provider refresh and persistence. This prevents two processes from replaying the same
//! rotating refresh token or observing a partially persisted transaction.

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use codex_utils_home_dir::find_codex_home;
use sha2::Digest;
use sha2::Sha256;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use tokio::time::timeout;

const REFRESH_LOCK_DIR: &str = "mcp-oauth-locks";
const REFRESH_LOCK_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(/*secs*/ 10);
const REFRESH_LOCK_RETRY_SLEEP: Duration = Duration::from_millis(/*millis*/ 50);
// Keep this internal target stable so diagnostics and cross-process tests can distinguish actual
// WouldBlock contention from a contender that merely started late and observed persisted tokens.
const LOCK_CONTENTION_EVENT_TARGET: &str = "codex_rmcp_client::oauth::refresh_lock::contention";

pub(crate) struct RefreshCredentialLock {
    _files: Vec<File>,
}

impl RefreshCredentialLock {
    pub(crate) async fn acquire_for_server(server_name: &str, url: &str) -> Result<Self> {
        let store_key = super::compute_store_key(server_name, url)?;
        let codex_home = find_codex_home()?;
        // Native keyring entries are shared across CODEX_HOME / AIRS environments. Always
        // take the user-level lock first, then retain the legacy home lock for older clients.
        let shared_home = dirs::home_dir()
            .context("could not locate OAuth lock home")?
            .join(".codex-mcp");
        Self::acquire_for_homes(
            &shared_home,
            &codex_home,
            &store_key,
            REFRESH_LOCK_ACQUIRE_TIMEOUT,
        )
        .await
    }

    async fn acquire_for_homes(
        shared_home: &Path,
        codex_home: &Path,
        store_key: &str,
        acquire_timeout: Duration,
    ) -> Result<Self> {
        timeout(acquire_timeout, async {
            let mut guard = Self::acquire_in(shared_home, store_key, acquire_timeout).await?;
            if shared_home.canonicalize()? != codex_home {
                let local = Self::acquire_in(codex_home, store_key, acquire_timeout).await?;
                guard._files.extend(local._files);
            }
            Ok(guard)
        })
        .await
        .context("timed out waiting for OAuth credential coordination")?
    }

    async fn acquire_in(
        codex_home: &Path,
        store_key: &str,
        acquire_timeout: Duration,
    ) -> Result<Self> {
        let mut hasher = Sha256::new();
        hasher.update(store_key.as_bytes());
        let path = codex_home
            .join(REFRESH_LOCK_DIR)
            .join(format!("{:x}.lock", hasher.finalize()));
        if let Some(parent) = path.parent() {
            let mut builder = fs::DirBuilder::new();
            builder.recursive(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(parent)?;
            anyhow::ensure!(
                fs::symlink_metadata(parent)?.is_dir(),
                "invalid OAuth lock directory"
            );
        }

        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
        }
        let file = options
            .open(&path)
            .with_context(|| format!("failed to open OAuth refresh lock {}", path.display()))?;
        anyhow::ensure!(file.metadata()?.is_file(), "invalid OAuth credential lock");
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            anyhow::ensure!(
                file.metadata()?.file_attributes() & 0x0400 == 0,
                "invalid OAuth credential lock"
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = file.metadata()?;
            anyhow::ensure!(
                metadata.uid() == unsafe { libc::geteuid() },
                "OAuth credential lock must be owned by this user"
            );
            if metadata.mode() & 0o077 != 0 {
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(fs::Permissions::from_mode(0o600))?;
            }
        }

        // Bound every contender, but keep the acquired lock for the full provider request and
        // persistence transaction. Releasing it while awaiting the provider would allow concurrent
        // use of a rotating refresh token.
        let mut reported_contention = false;
        timeout(acquire_timeout, async {
            loop {
                match file.try_lock() {
                    Ok(()) => return Ok(()),
                    Err(std::fs::TryLockError::WouldBlock) => {
                        if !reported_contention {
                            tracing::debug!(
                                target: LOCK_CONTENTION_EVENT_TARGET,
                                lock_path = %path.display(),
                                "waiting for another process to finish refreshing MCP OAuth credentials"
                            );
                            reported_contention = true;
                        }
                        sleep(REFRESH_LOCK_RETRY_SLEEP).await;
                    }
                    Err(error) => return Err(std::io::Error::from(error)),
                }
            }
        })
        .await
        .map_err(|_| {
            anyhow!(
                "timed out after {acquire_timeout:?} waiting for OAuth refresh lock {}",
                path.display()
            )
        })?
        .with_context(|| format!("failed to lock OAuth refresh lock {}", path.display()))?;

        Ok(Self { _files: vec![file] })
    }
}

#[cfg(test)]
#[path = "refresh_lock_tests.rs"]
mod tests;
