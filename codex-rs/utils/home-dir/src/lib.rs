pub mod airs_session;

use codex_utils_absolute_path::AbsolutePathBuf;
use dirs::home_dir;
use std::path::PathBuf;
use std::sync::OnceLock;

static APPLICATION_HOME: OnceLock<AbsolutePathBuf> = OnceLock::new();
/// Standalone product version, distinct from the pinned upstream crate versions.
pub const AIRS_HARNESS_VERSION: &str = "0.1.3-alpha.6.mcp.1";
static ENVIRONMENT_HOME: OnceLock<AbsolutePathBuf> = OnceLock::new();

/// Select one independent AIRS environment before loading runtime configuration.
/// Selection is immutable for the lifetime of this process.
pub fn select_airs_environment_home(path: PathBuf) -> std::io::Result<()> {
    if APPLICATION_HOME.get().is_none() {
        return Err(std::io::Error::other(
            "AIRS application home is not initialized",
        ));
    }
    let home = AbsolutePathBuf::from_absolute_path(path.canonicalize()?)?;
    ENVIRONMENT_HOME
        .set(home)
        .map_err(|_| std::io::Error::other("environment was already selected"))
}

/// Whether this process was initialized as the standalone AIRS harness.
pub fn is_airs_harness() -> bool {
    APPLICATION_HOME.get().is_some()
}

/// Public command name for built-in instructions, distinct from package names.
pub fn command_name() -> &'static str {
    if is_airs_harness() { "airs" } else { "codex" }
}

/// Bind this process to Prisma AIRS Harness's independent application state.
/// Call before creating the runtime or loading any configuration. Does not alter
/// HOME or CODEX_HOME, including when spawning local tools.
pub fn initialize_airs_harness_home() -> std::io::Result<()> {
    // The legacy override remains supported for existing automation.
    let configured =
        std::env::var_os("AIRS_HARNESS_HOME").or_else(|| std::env::var_os("AIRS_TERMINAL_HOME"));
    let path = match configured {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        Some(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "AIRS_HARNESS_HOME must not be empty",
            ));
        }
        None => {
            let user_home =
                home_dir().ok_or_else(|| std::io::Error::other("Could not find home directory"))?;
            let current = user_home.join(".airs-harness");
            let legacy = user_home.join(".airs-terminal");
            // Keep absolute catalog/helper paths and encrypted identities intact.
            // Never move a live application home or merge two independent homes.
            if !current.try_exists()? && legacy.try_exists()? {
                legacy
            } else {
                current
            }
        }
    };
    if !path.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "AIRS_HARNESS_HOME must be an absolute directory path",
        ));
    }
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(&path)?;
    let home = AbsolutePathBuf::from_absolute_path(path.canonicalize()?)?;
    APPLICATION_HOME
        .set(home)
        .map_err(|_| std::io::Error::other("application home was already initialized"))
}

/// Returns the path to the Codex configuration directory, which can be
/// specified by the `CODEX_HOME` environment variable. If not set, defaults to
/// `~/.codex`.
///
/// - If `CODEX_HOME` is set, the value must exist and be a directory. The
///   value will be canonicalized and this function will Err otherwise.
/// - If `CODEX_HOME` is not set, this function does not verify that the
///   directory exists.
pub fn find_codex_home() -> std::io::Result<AbsolutePathBuf> {
    if let Some(home) = ENVIRONMENT_HOME.get() {
        return Ok(home.clone());
    }
    if let Some(home) = APPLICATION_HOME.get() {
        return Ok(home.clone());
    }
    let codex_home_env = std::env::var("CODEX_HOME")
        .ok()
        .filter(|val| !val.is_empty());
    find_codex_home_from_env(codex_home_env.as_deref())
}

fn find_codex_home_from_env(codex_home_env: Option<&str>) -> std::io::Result<AbsolutePathBuf> {
    // Honor the `CODEX_HOME` environment variable when it is set to allow users
    // (and tests) to override the default location.
    match codex_home_env {
        Some(val) => {
            let path = PathBuf::from(val);
            let metadata = std::fs::metadata(&path).map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("CODEX_HOME points to {val:?}, but that path does not exist"),
                ),
                _ => std::io::Error::new(
                    err.kind(),
                    format!("failed to read CODEX_HOME {val:?}: {err}"),
                ),
            })?;

            if !metadata.is_dir() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("CODEX_HOME points to {val:?}, but that path is not a directory"),
                ))
            } else {
                let canonical = path.canonicalize().map_err(|err| {
                    std::io::Error::new(
                        err.kind(),
                        format!("failed to canonicalize CODEX_HOME {val:?}: {err}"),
                    )
                })?;
                AbsolutePathBuf::from_absolute_path(canonical)
            }
        }
        None => {
            let mut p = home_dir().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                )
            })?;
            p.push(".codex");
            AbsolutePathBuf::from_absolute_path(p)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::find_codex_home_from_env;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use dirs::home_dir;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn find_codex_home_env_missing_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let missing = temp_home.path().join("missing-codex-home");
        let missing_str = missing
            .to_str()
            .expect("missing codex home path should be valid utf-8");

        let err = find_codex_home_from_env(Some(missing_str)).expect_err("missing CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert!(
            err.to_string().contains("CODEX_HOME"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_file_path_is_fatal() {
        let temp_home = TempDir::new().expect("temp home");
        let file_path = temp_home.path().join("codex-home.txt");
        fs::write(&file_path, "not a directory").expect("write temp file");
        let file_str = file_path
            .to_str()
            .expect("file codex home path should be valid utf-8");

        let err = find_codex_home_from_env(Some(file_str)).expect_err("file CODEX_HOME");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_codex_home_env_valid_directory_canonicalizes() {
        let temp_home = TempDir::new().expect("temp home");
        let temp_str = temp_home
            .path()
            .to_str()
            .expect("temp codex home path should be valid utf-8");

        let resolved = find_codex_home_from_env(Some(temp_str)).expect("valid CODEX_HOME");
        let expected = temp_home
            .path()
            .canonicalize()
            .expect("canonicalize temp home");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }

    #[test]
    fn find_codex_home_without_env_uses_default_home_dir() {
        let resolved =
            find_codex_home_from_env(/*codex_home_env*/ None).expect("default CODEX_HOME");
        let mut expected = home_dir().expect("home dir");
        expected.push(".codex");
        let expected = AbsolutePathBuf::from_absolute_path(expected).expect("absolute home");
        assert_eq!(resolved, expected);
    }
}
