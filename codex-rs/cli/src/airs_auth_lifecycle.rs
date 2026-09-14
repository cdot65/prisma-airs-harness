//! Serialize short authentication commits separately from browser/refresh work.
use super::airs_environment;
use anyhow::Context;
use codex_utils_home_dir::airs_session::AUTH_GENERATION_FILE;
use codex_utils_home_dir::airs_session::AuthGeneration;
use codex_utils_home_dir::airs_session::AuthGenerationState;
use codex_utils_home_dir::airs_session::read_auth_generation;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use uuid::Uuid;

/// A login may commit only if no logout or other login changed its starting epoch.
pub(super) struct LoginAttempt {
    home: PathBuf,
    starting: Option<AuthGeneration>,
}

fn state_lock(home: &Path) -> anyhow::Result<File> {
    let mut options = std::fs::OpenOptions::new();
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
    let file = options.open(home.join(".auth-generation.lock"))?;
    let metadata = file.metadata()?;
    anyhow::ensure!(metadata.is_file(), "Invalid authentication state lock");
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        anyhow::ensure!(
            metadata.file_attributes() & 0x0400 == 0,
            "Invalid authentication state lock"
        );
    }
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(std::fs::TryLockError::WouldBlock) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(std::fs::TryLockError::WouldBlock) => {
                anyhow::bail!("Authentication state is busy; retry the command")
            }
            Err(std::fs::TryLockError::Error(error)) => return Err(error.into()),
        }
    }
}

impl LoginAttempt {
    /// Capture before acquiring the configuration lock or waiting for user input.
    pub(super) fn begin(home: &Path) -> anyhow::Result<Self> {
        let _lock = state_lock(home)?;
        Ok(Self {
            home: home.to_owned(),
            starting: read_auth_generation(home)?,
        })
    }

    /// Complete an explicitly requested, verified same-identity restore. The caller holds
    /// the environment lock and has persisted only tokens for the existing binding.
    /// Logout and ordinary login still invalidate this attempt and all prior clients.
    pub(super) fn complete_restore(self) -> anyhow::Result<()> {
        let _lock = state_lock(&self.home)?;
        anyhow::ensure!(
            self.starting
                .as_ref()
                .is_some_and(|generation| generation.state == AuthGenerationState::Active)
                && read_auth_generation(&self.home)? == self.starting
                && !self.home.join("logged-out").try_exists()?,
            "Authentication changed while restoring sign-in; start a new session after login"
        );
        Ok(())
    }

    /// Install only local binding/configuration files while holding this lock.
    /// Native storage and all network/user interaction must happen beforehand.
    pub(super) fn commit(self, install: impl FnOnce() -> anyhow::Result<()>) -> anyhow::Result<()> {
        let _lock = state_lock(&self.home)?;
        anyhow::ensure!(
            read_auth_generation(&self.home)? == self.starting,
            "Authentication changed while signing in; this attempt was cancelled. Run login again"
        );
        // Keep interrupted installation or activation fail-closed. Native records
        // remain recoverable, but an older running client cannot adopt this login.
        let pending = AuthGeneration::new(
            AuthGenerationState::Revoked,
            Uuid::new_v4().simple().to_string(),
        )?;
        airs_environment::atomic_write(
            &self.home.join(AUTH_GENERATION_FILE),
            pending.encode().as_bytes(),
        )?;
        install().context("Sign-in was not activated; run login again")?;
        let active = AuthGeneration::new(
            AuthGenerationState::Active,
            Uuid::new_v4().simple().to_string(),
        )?;
        airs_environment::atomic_write(
            &self.home.join(AUTH_GENERATION_FILE),
            active.encode().as_bytes(),
        )
        .context("Credential saved, but session activation failed; run login again")
    }
}

/// Invalidate running clients before waiting for any long configuration lock.
pub(super) fn revoke(home: &Path) -> anyhow::Result<()> {
    let _lock = state_lock(home)?;
    let revoked = AuthGeneration::new(
        AuthGenerationState::Revoked,
        Uuid::new_v4().simple().to_string(),
    )?;
    airs_environment::atomic_write(
        &home.join(AUTH_GENERATION_FILE),
        revoked.encode().as_bytes(),
    )?;
    airs_environment::atomic_write(
        &home.join("logged-out"),
        b"Local credentials disabled. Run login to reauthenticate.\n",
    )
}

#[cfg(test)]
#[path = "airs_auth_lifecycle_tests.rs"]
mod tests;
