// Reuse the upstream CLI and runtime in this standalone binary. Product startup
// is selected at compile time by CARGO_BIN_NAME; no external Codex installation
// or PAH process is launched.
include!("main.rs");
