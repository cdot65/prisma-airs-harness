/// The current Codex CLI version as embedded at compile time.
#[cfg(not(test))]
pub const CODEX_CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

// Keep rendering fixtures independent of the release package version.
#[cfg(test)]
pub const CODEX_CLI_VERSION: &str = "0.0.0";
