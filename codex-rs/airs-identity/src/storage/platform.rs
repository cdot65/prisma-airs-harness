//! Persist OIDC rotation state directly in the OS store, without a kernel cache.
//! Linux Secret Service also works where container policy denies keyctl syscalls.
use codex_keyring_store::CredentialStoreError;
use codex_keyring_store::KeyringStore;

#[derive(Debug)]
pub(super) struct PlatformStore;

/// Native credential storage for token bundles on Linux, macOS and Windows.
#[derive(Debug)]
pub struct CredentialStore;

impl KeyringStore for CredentialStore {
    fn load(&self, service: &str, account: &str) -> Result<Option<String>, CredentialStoreError> {
        super::chunks::ChunkedStore(PlatformStore).load(service, account)
    }
    fn save(&self, service: &str, account: &str, value: &str) -> Result<(), CredentialStoreError> {
        super::chunks::ChunkedStore(PlatformStore).save(service, account, value)
    }
    fn delete(&self, service: &str, account: &str) -> Result<bool, CredentialStoreError> {
        super::chunks::ChunkedStore(PlatformStore).delete(service, account)
    }
}

#[cfg(target_os = "linux")]
fn entry(service: &str, account: &str) -> Result<keyring::Entry, CredentialStoreError> {
    let credential = keyring::secret_service::default_credential_builder()
        .build(/*target*/ None, service, account)
        .map_err(CredentialStoreError::new)?;
    Ok(keyring::Entry::new_with_credential(credential))
}

impl KeyringStore for PlatformStore {
    fn load(&self, service: &str, account: &str) -> Result<Option<String>, CredentialStoreError> {
        #[cfg(target_os = "linux")]
        {
            match entry(service, account)?.get_password() {
                Ok(value) => Ok(Some(value)),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(error) => Err(CredentialStoreError::new(error)),
            }
        }
        #[cfg(not(target_os = "linux"))]
        codex_keyring_store::DefaultKeyringStore.load(service, account)
    }

    fn save(&self, service: &str, account: &str, value: &str) -> Result<(), CredentialStoreError> {
        #[cfg(target_os = "linux")]
        {
            entry(service, account)?
                .set_password(value)
                .map_err(CredentialStoreError::new)
        }
        #[cfg(not(target_os = "linux"))]
        codex_keyring_store::DefaultKeyringStore.save(service, account, value)
    }

    fn delete(&self, service: &str, account: &str) -> Result<bool, CredentialStoreError> {
        #[cfg(target_os = "linux")]
        {
            match entry(service, account)?.delete_credential() {
                Ok(()) => Ok(true),
                Err(keyring::Error::NoEntry) => Ok(false),
                Err(error) => Err(CredentialStoreError::new(error)),
            }
        }
        #[cfg(not(target_os = "linux"))]
        codex_keyring_store::DefaultKeyringStore.delete(service, account)
    }
}
