//! Local logout epoch for AIRS network clients. This contains no credentials.
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

pub const AUTH_GENERATION_FILE: &str = "auth-generation";
const REVOKED: &str = "This AIRS session was signed out or its authentication changed. Start a new session after signing in.";
static PROCESS_GUARD: OnceLock<Result<AirsSessionGuard, ()>> = OnceLock::new();

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthGenerationState {
    Active,
    Revoked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthGeneration {
    pub state: AuthGenerationState,
    pub nonce: String,
}

impl AuthGeneration {
    pub fn new(state: AuthGenerationState, nonce: String) -> io::Result<Self> {
        if nonce.len() != 32
            || !nonce
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid AIRS authentication generation",
            ));
        }
        Ok(Self { state, nonce })
    }

    pub fn encode(&self) -> String {
        let state = match self.state {
            AuthGenerationState::Active => "active",
            AuthGenerationState::Revoked => "revoked",
        };
        format!("v1 {state} {}\n", self.nonce)
    }
}

/// Missing records are allowed only as a legacy baseline; malformed records fail closed.
pub fn read_auth_generation(home: &Path) -> io::Result<Option<AuthGeneration>> {
    let path = home.join(AUTH_GENERATION_FILE);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || metadata.len() > 128 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid AIRS authentication generation file",
        ));
    }
    let mut encoded = String::new();
    fs::File::open(path)?
        .take(129)
        .read_to_string(&mut encoded)?;
    let invalid = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid AIRS authentication generation",
        )
    };
    let fields: Vec<_> = encoded.trim_end_matches('\n').split(' ').collect();
    let ["v1", state, nonce] = fields.as_slice() else {
        return Err(invalid());
    };
    let state = match *state {
        "active" => AuthGenerationState::Active,
        "revoked" => AuthGenerationState::Revoked,
        _ => return Err(invalid()),
    };
    let generation = AuthGeneration::new(state, (*nonce).to_owned())?;
    if encoded != generation.encode() {
        return Err(invalid());
    }
    Ok(Some(generation))
}

#[derive(Clone, Debug)]
pub struct AirsSessionGuard {
    home: PathBuf,
    generation: Option<AuthGeneration>,
    invalidated: Arc<AtomicBool>,
}

impl AirsSessionGuard {
    /// Capture once before loading credentials, then recheck immediately before using them.
    pub fn capture(home: &Path) -> io::Result<Self> {
        let guard = Self {
            home: home.to_path_buf(),
            generation: read_auth_generation(home)?,
            invalidated: Arc::new(AtomicBool::new(false)),
        };
        guard.check()?;
        Ok(guard)
    }

    /// A changed epoch permanently invalidates this guard and every clone.
    pub fn check(&self) -> io::Result<()> {
        let valid = !self.invalidated.load(Ordering::Acquire)
            && matches!(fs::symlink_metadata(self.home.join("logged-out")), Err(error) if error.kind() == io::ErrorKind::NotFound)
            && !self
                .generation
                .as_ref()
                .is_some_and(|generation| generation.state == AuthGenerationState::Revoked)
            && read_auth_generation(&self.home)
                .is_ok_and(|generation| generation == self.generation);
        if !valid {
            self.invalidated.store(true, Ordering::Release);
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, REVOKED));
        }
        Ok(())
    }
}

/// Pin after startup authentication, before constructing clients. Never reset on refresh.
pub fn pin_current_airs_session() -> io::Result<()> {
    current_airs_session_guard().map(|_| ())
}

/// Every AIRS client in this process shares a baseline and irreversible revocation latch.
/// Upstream Codex callers do not opt into this application-specific policy.
pub fn current_airs_session_guard() -> io::Result<Option<AirsSessionGuard>> {
    if !crate::is_airs_harness() {
        return Ok(None);
    }
    let captured = PROCESS_GUARD.get_or_init(|| {
        crate::find_codex_home()
            .and_then(|home| AirsSessionGuard::capture(home.as_path()))
            .map_err(|_| ())
    });
    match captured {
        Ok(guard) => {
            guard.check()?;
            Ok(Some(guard.clone()))
        }
        Err(()) => Err(io::Error::new(io::ErrorKind::PermissionDenied, REVOKED)),
    }
}

#[cfg(test)]
#[path = "airs_session_tests.rs"]
mod tests;
