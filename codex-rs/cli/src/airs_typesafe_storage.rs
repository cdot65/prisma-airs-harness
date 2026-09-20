//! Journal only native-store identifiers, so interrupted writes remain recoverable.
use super::{Settings, Store, read};
use anyhow::Context;
use std::io::Read;
use std::path::Path;
use uuid::Uuid;

const JOURNAL: &str = "typesafe-cleanup.json";

pub(super) fn lock(home: &Path) -> anyhow::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(home.join(".typesafe.lock"))?;
    anyhow::ensure!(file.metadata()?.is_file(), "Invalid TypeSafe configuration lock");
    file.try_lock().context("TypeSafe settings are busy; retry the command")?;
    Ok(file)
}

pub(super) fn pending(home: &Path, ids: &[Uuid]) -> anyhow::Result<()> {
    crate::airs_environment::atomic_write(&home.join(JOURNAL), &serde_json::to_vec(ids)?)
}

/// Delete every journaled entry except the committed active key. Keep the journal
/// until every deletion succeeds; deletion of an already absent entry is idempotent.
pub(super) fn recover(home: &Path, store: &impl Store) -> anyhow::Result<()> {
    let file = match std::fs::File::open(home.join(JOURNAL)) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    anyhow::ensure!(file.metadata()?.is_file() && file.metadata()?.len() <= 16_384, "Invalid TypeSafe cleanup journal");
    let mut bytes = Vec::new();
    file.take(16_385).read_to_end(&mut bytes)?;
    let ids: Vec<Uuid> = serde_json::from_slice(&bytes).context("Invalid TypeSafe cleanup journal")?;
    let active = read(home)?.map(|settings: Settings| settings.id);
    for id in ids {
        if Some(id) != active {
            store.delete(id).context("TypeSafe credential cleanup is pending; retry airs env typesafe set or clear when secure storage is available")?;
        }
    }
    std::fs::remove_file(home.join(JOURNAL))?;
    Ok(())
}
