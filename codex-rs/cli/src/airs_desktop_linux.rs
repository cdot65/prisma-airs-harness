//! Linux Desktop project exclusions, resolved without reading credential data.
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub(super) fn protect_project(
    project: &Path,
    account_home: &Path,
    resources: &Path,
) -> io::Result<()> {
    if account_home.starts_with(project) || overlaps(project, resources) {
        return Err(io::Error::other(
            "Project includes private account or trusted runtime state",
        ));
    }
    for root in [
        "/dev", "/proc", "/sys", "/run", "/etc", "/root", "/boot", "/lib", "/lib64", "/var/lib",
    ] {
        let root = Path::new(root);
        if overlaps(project, root) || overlaps(project, &resolve_existing_ancestor(root)?) {
            return Err(io::Error::other(
                "Linux system or session state is not a project scope",
            ));
        }
    }
    for relative in [
        ".airs-harness",
        ".airs-terminal",
        ".codex",
        ".ssh",
        ".aws",
        ".azure",
        ".kube",
        ".gnupg",
        ".pki",
        ".Xauthority",
        ".config/prisma-airs",
        ".config/airs-cli",
        ".config/io.cdot.airs-desktop",
        ".local/share/io.cdot.airs-desktop",
        ".local/state/io.cdot.airs-desktop",
        ".local/state/airs-harness",
        ".local/share/keyrings",
    ] {
        let private = account_home.join(relative);
        if overlaps(project, &private) || overlaps(project, &resolve_existing_ancestor(&private)?) {
            return Err(io::Error::other(
                "Project includes protected Linux credential state",
            ));
        }
    }
    Ok(())
}

fn overlaps(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

// Resolve existing ancestors as well as existing leaves: a missing keyring
// below a symlinked XDG directory must not become writable project state later.
fn resolve_existing_ancestor(path: &Path) -> io::Result<PathBuf> {
    let mut ancestor = path;
    loop {
        match ancestor.canonicalize() {
            Ok(resolved) => {
                let suffix = path.strip_prefix(ancestor).map_err(io::Error::other)?;
                return Ok(resolved.join(suffix));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                ancestor = ancestor.parent().ok_or(error)?;
            }
            Err(error) => return Err(error),
        }
    }
}
