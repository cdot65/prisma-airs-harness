//! Durable, nonsecret deletion intent for an explicitly logged-out environment.
use super::Binding;
use super::SERVICE;
use super::Source;
use super::airs_environment;
use anyhow::Context;
use codex_keyring_store::KeyringStore;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use uuid::Uuid;

const JOURNAL: &str = "credential-pending-logout.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum StoreKind {
    WorkspaceKeyringV1,
    OidcIdentityV1,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    schema_version: u32,
    account: Uuid,
    store: StoreKind,
}

/// Delete only the native representation recorded when logout was requested.
pub(super) trait Store {
    fn delete(&self, store: StoreKind, account: Uuid) -> anyhow::Result<()>;
}

pub(super) struct NativeStore;

impl Store for NativeStore {
    fn delete(&self, store: StoreKind, account: Uuid) -> anyhow::Result<()> {
        match store {
            StoreKind::WorkspaceKeyringV1 => {
                codex_keyring_store::DefaultKeyringStore.delete(SERVICE, &account.to_string())
            }
            // This store owns OIDC's native chunk manifest/tombstone protocol.
            StoreKind::OidcIdentityV1 => {
                codex_airs_identity::CredentialStore.delete(SERVICE, &account.to_string())
            }
        }
        .map(|_| ())
        .map_err(|error| {
            super::super::airs_storage_error::report(error, "delete after local logout")
        })
    }
}

fn store_kind(source: &Source) -> Option<StoreKind> {
    match source {
        Source::Keyring => Some(StoreKind::WorkspaceKeyringV1),
        Source::Oidc { .. } => Some(StoreKind::OidcIdentityV1),
        Source::File { .. } | Source::Environment { .. } => None,
    }
}

pub(super) fn prepare(home: &Path, binding: &Binding) -> anyhow::Result<()> {
    let Some(store) = binding.source.as_ref().and_then(store_kind) else {
        return Ok(());
    };
    anyhow::ensure!(
        home.join("logged-out").try_exists()?,
        "Credential deletion requires an explicit local logout"
    );
    anyhow::ensure!(
        !home.join(JOURNAL).try_exists()?,
        "Finish pending credential deletion before creating another logout record"
    );
    airs_environment::atomic_write(
        &home.join(JOURNAL),
        &serde_json::to_vec(&Pending {
            schema_version: 1,
            account: binding.id,
            store,
        })?,
    )
}

pub(super) fn recover(home: &Path, store: &impl Store) -> anyhow::Result<()> {
    let path = home.join(JOURNAL);
    let Some(pending): Option<Pending> = super::transaction::read_cleanup(&path)? else {
        return Ok(());
    };
    anyhow::ensure!(
        pending.schema_version == 1,
        "Unsupported logout cleanup journal"
    );
    anyhow::ensure!(
        home.join("logged-out").try_exists()?,
        "Pending logout cleanup has inconsistent local state; inspect this environment before retrying"
    );
    // An interrupted logout may have published its journal before clearing the
    // binding. Finish that step only for the same account and native store.
    match std::fs::symlink_metadata(home.join("credential-binding.json")) {
        Ok(_) => {
            let mut binding = super::read_binding(home)?;
            anyhow::ensure!(
                binding.id == pending.account
                    && binding
                        .source
                        .as_ref()
                        .is_none_or(|source| store_kind(source) == Some(pending.store)),
                "Pending logout cleanup does not match the current credential binding"
            );
            if binding.source.is_some() {
                binding.source = None;
                airs_environment::atomic_write(
                    &home.join("credential-binding.json"),
                    &serde_json::to_vec_pretty(&binding)?,
                )?;
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    store.delete(pending.store, pending.account).context(
        "Credential deletion is pending; retry login or logout when secure storage is available",
    )?;
    std::fs::remove_file(path)?;
    Ok(())
}

#[cfg(test)]
#[path = "airs_logout_cleanup_tests.rs"]
mod tests;
