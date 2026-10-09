//! Fail closed until Desktop's process and credential isolation is ported.
use clap::Args as ClapArgs;
use codex_arg0::Arg0DispatchPaths;

#[derive(Debug, ClapArgs)]
pub(super) struct Args {
    /// Retain command-line parsing without authorizing a platform bypass.
    #[arg(long, hide = true)]
    pub allow_http_loopback: bool,
}

pub(super) async fn run(_arg0_paths: Arg0DispatchPaths, _args: &Args) -> anyhow::Result<()> {
    anyhow::bail!(
        "Desktop Harness transport is available only on macOS. This platform has no approved Desktop sandbox; no bootstrap credentials or input were read."
    )
}
