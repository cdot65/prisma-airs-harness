//! Track new OS entries until verified configuration owns them.
use super::Binding;
use super::SERVICE;
use super::Source;
use super::airs_environment;
use anyhow::Context;
use codex_airs_identity::WorkspaceCredentialFormat;
use codex_airs_identity::WorkspaceCredentialStore;
use codex_keyring_store::DefaultKeyringStore;
use codex_keyring_store::KeyringStore;
use codex_login::auth::CredentialRecovery;
use serde::Deserialize;
use serde::Serialize;
use std::io::Read;
use std::path::Path;
use uuid::Uuid;

const JOURNAL: &str = "credential-pending-cleanup.json";

/// Operations on native credentials, with errors already safely classified.
pub(super) trait Store {
    fn save(&self, kind: StoreKind, account: Uuid, token: &str) -> anyhow::Result<()>;
    fn load(&self, kind: StoreKind, account: Uuid) -> anyhow::Result<Option<String>>;
    fn delete(&self, kind: StoreKind, account: Uuid) -> anyhow::Result<()>;
}

pub(super) struct NativeStore;

impl Store for NativeStore {
    fn save(&self, kind: StoreKind, account: Uuid, token: &str) -> anyhow::Result<()> {
        match kind {
            StoreKind::WorkspaceKeyringV1 => {
                DefaultKeyringStore.save(SERVICE, &account.to_string(), token)
            }
            StoreKind::WorkspaceKeyringV2 => {
                WorkspaceCredentialStore.save_new(SERVICE, account, token)
            }
            StoreKind::OidcIdentityV1 => {
                codex_airs_identity::CredentialStore.save(SERVICE, &account.to_string(), token)
            }
        }
        .map_err(|error| super::super::airs_storage_error::report(error, "save"))
    }
    fn load(&self, kind: StoreKind, account: Uuid) -> anyhow::Result<Option<String>> {
        match kind {
            StoreKind::WorkspaceKeyringV1 => WorkspaceCredentialStore.load(
                SERVICE,
                WorkspaceCredentialFormat::LegacyRaw,
                account,
            ),
            StoreKind::WorkspaceKeyringV2 => WorkspaceCredentialStore.load(
                SERVICE,
                WorkspaceCredentialFormat::ChunkedV2,
                account,
            ),
            StoreKind::OidcIdentityV1 => {
                codex_airs_identity::CredentialStore.load(SERVICE, &account.to_string())
            }
        }
        .map_err(|error| super::super::airs_storage_error::report(error, "verify saved credential"))
    }
    fn delete(&self, kind: StoreKind, account: Uuid) -> anyhow::Result<()> {
        match kind {
            StoreKind::WorkspaceKeyringV1 => WorkspaceCredentialStore.delete(
                SERVICE,
                WorkspaceCredentialFormat::LegacyRaw,
                account,
            ),
            StoreKind::WorkspaceKeyringV2 => WorkspaceCredentialStore.delete(
                SERVICE,
                WorkspaceCredentialFormat::ChunkedV2,
                account,
            ),
            StoreKind::OidcIdentityV1 => {
                codex_airs_identity::CredentialStore.delete(SERVICE, &account.to_string())
            }
        }
        .map(|_| ())
        .map_err(|error| {
            super::super::airs_storage_error::report(error, "clean up uncommitted credential")
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    schema_version: u32,
    store: StoreKind,
    account: Uuid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub(super) enum StoreKind {
    #[serde(rename = "workspace-keyring-v1")]
    WorkspaceKeyringV1,
    #[serde(rename = "workspace-keyring-v2")]
    WorkspaceKeyringV2,
    #[serde(rename = "oidc-identity-v1")]
    OidcIdentityV1,
}

fn store_kind(source: &Source) -> Option<StoreKind> {
    match source {
        Source::Keyring => Some(StoreKind::WorkspaceKeyringV1),
        Source::KeyringV2 => Some(StoreKind::WorkspaceKeyringV2),
        Source::Oidc { .. } => Some(StoreKind::OidcIdentityV1),
        Source::File { .. } | Source::Environment { .. } => None,
    }
}

// Legacy raw credentials and OIDC manifests share SERVICE + UUID. A kind
// mismatch cannot authorize cleanup or replacement within that same namespace.
fn ensure_unambiguous_native_owner(
    bound: Option<&Binding>,
    account: Uuid,
    kind: StoreKind,
) -> anyhow::Result<()> {
    let conflict = bound
        .filter(|binding| binding.id == account)
        .and_then(|binding| binding.source.as_ref().and_then(store_kind))
        .is_some_and(|owner| {
            matches!(
                (kind, owner),
                (StoreKind::WorkspaceKeyringV1, StoreKind::OidcIdentityV1)
                    | (StoreKind::OidcIdentityV1, StoreKind::WorkspaceKeyringV1)
            )
        });
    anyhow::ensure!(
        !conflict,
        "Credential metadata conflicts across legacy native formats; existing credentials were not changed"
    );
    Ok(())
}

fn optional_bytes(path: &Path) -> anyhow::Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn restore_file(path: &Path, previous: Option<&[u8]>) -> anyhow::Result<()> {
    if let Some(previous) = previous {
        airs_environment::atomic_write(path, previous)
    } else {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

// Callers supply classified native-store errors or local filesystem failures,
// never raw provider/backend payloads. Preserve the original error for downcasts;
// a secondary typed recovery reason matters only when the primary has none.
fn preserve_primary_failure(primary: anyhow::Error, secondary: anyhow::Error) -> anyhow::Error {
    let recovery = secondary
        .downcast_ref::<CredentialRecovery>()
        .copied()
        .filter(|_| primary.downcast_ref::<CredentialRecovery>().is_none());
    let combined = primary.context(format!("{secondary:#}"));
    match recovery {
        Some(recovery) => combined.context(recovery),
        None => combined,
    }
}

pub(super) fn read_cleanup<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> anyhow::Result<Option<T>> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_FLAG_OPEN_REPARSE_POINT: inspect the entry, never follow a link.
        options.custom_flags(0x0020_0000);
    }
    let file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let metadata = file.metadata()?;
    anyhow::ensure!(
        metadata.is_file() && metadata.len() <= 1024,
        "Invalid credential cleanup journal"
    );
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // FILE_ATTRIBUTE_REPARSE_POINT also excludes non-symlink reparse files.
        anyhow::ensure!(
            metadata.file_attributes() & 0x0400 == 0,
            "Invalid credential cleanup journal"
        );
    }
    let mut bytes = Vec::new();
    file.take(1025).read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() <= 1024, "Invalid credential cleanup journal");
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|_| anyhow::anyhow!("Invalid credential cleanup journal"))
}

pub(super) fn recover(home: &Path, store: &impl Store) -> anyhow::Result<()> {
    let path = home.join(JOURNAL);
    let Some(pending): Option<Pending> = read_cleanup(&path)? else {
        return Ok(());
    };
    anyhow::ensure!(
        pending.schema_version == 1,
        "Unsupported credential cleanup journal"
    );
    let bound = optional_bytes(&home.join("credential-binding.json"))?
        .map(|bytes| super::parse_binding(&bytes))
        .transpose()?;
    ensure_unambiguous_native_owner(bound.as_ref(), pending.account, pending.store)?;
    // A process may have stopped after installing the binding. Never remove
    // its referenced key; a subsequent login can finish configuration safely.
    if !bound.is_some_and(|binding| {
        binding.id == pending.account
            && binding.source.as_ref().and_then(store_kind) == Some(pending.store)
    }) {
        store.delete(pending.store, pending.account).context(
            "Credential cleanup is pending; retry login or logout when secure storage is available",
        )?;
    }
    std::fs::remove_file(path)?;
    Ok(())
}

pub(super) fn persist(
    home: &Path,
    binding: &Binding,
    token: &str,
    store: &impl Store,
    install: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let kind = binding
        .source
        .as_ref()
        .and_then(store_kind)
        .context("Expected a native credential binding")?;
    let previous = optional_bytes(&home.join("credential-binding.json"))?;
    let previous_config = optional_bytes(&home.join("config.toml"))?;
    let prior_binding = previous
        .as_ref()
        .map(|bytes| super::parse_binding(bytes))
        .transpose()?;
    ensure_unambiguous_native_owner(prior_binding.as_ref(), binding.id, kind)?;
    recover(home, store)?;
    let already_owned = prior_binding.is_some_and(|prior| {
        prior.id == binding.id && prior.source.as_ref().and_then(store_kind) == Some(kind)
    });
    if !already_owned {
        airs_environment::atomic_write(
            &home.join(JOURNAL),
            &serde_json::to_vec(&Pending {
                schema_version: 1,
                store: kind,
                account: binding.id,
            })?,
        )?;
    }
    let result = (|| {
        store.save(kind, binding.id, token)?;
        anyhow::ensure!(
            store.load(kind, binding.id)?.as_deref() == Some(token),
            "Credential storage verification failed; sign-in was not completed"
        );
        if let Err(mut error) = install() {
            // Restore both files even if the first restore fails. Snapshot bytes
            // stay in memory; the durable cleanup journal never copies secrets.
            let config_restore =
                restore_file(&home.join("config.toml"), previous_config.as_deref());
            let binding_restore =
                restore_file(&home.join("credential-binding.json"), previous.as_deref());
            if let Err(rollback) = config_restore {
                error = preserve_primary_failure(error, rollback.context(
                    "Credential configuration rollback failed; retry login after resolving the filesystem error",
                ));
            }
            if let Err(rollback) = binding_restore {
                error = preserve_primary_failure(error, rollback.context(
                    "Credential binding rollback failed; retry login after resolving the filesystem error",
                ));
            }
            return Err(error);
        }
        Ok(())
    })();
    if !already_owned && let Err(cleanup) = recover(home, store) {
        return Err(match result {
            Ok(()) => cleanup,
            Err(primary) => {
                preserve_primary_failure(primary, cleanup.context("Credential cleanup also failed"))
            }
        });
    }
    result
}

#[cfg(test)]
#[path = "airs_credential_transaction_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "airs_credential_namespace_tests.rs"]
mod namespace_tests;
