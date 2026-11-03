//! Select only internal Desktop Linux helper routes before account setup.
use std::ffi::OsString;
use std::path::Path;

pub(super) fn is_desktop_helper(mut args: impl Iterator<Item = OsString>) -> bool {
    let Some(argv0) = args.next() else {
        return false;
    };
    if Path::new(&argv0)
        .file_name()
        .is_some_and(|name| name == "codex-linux-sandbox")
    {
        return args
            .take_while(|arg| arg != "--")
            .any(|arg| arg == "--desktop-command");
    }
    args.next()
        .is_some_and(|arg| arg == "--codex-run-as-fs-helper")
        && args
            .next()
            .is_some_and(|arg| arg == "--airs-desktop-owned-helper")
        && args.next().is_none()
}

#[cfg(test)]
#[path = "airs_desktop_linux_dispatch_tests.rs"]
mod tests;
