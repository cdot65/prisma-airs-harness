//! Fallible deletion in the same default user keychain used by keyring.
use keyring::macos::MacCredential;
use keyring::macos::decode_error;
use security_framework::item::ItemClass;
use security_framework::item::ItemSearchOptions;
use security_framework::os::macos::keychain::SecKeychain;
use security_framework::os::macos::keychain::SecPreferencesDomain;

pub(super) fn delete(service: &str, account: &str) -> keyring::Result<()> {
    delete_with(service, account, |service, account| {
        let keychain = SecKeychain::default_for_domain(SecPreferencesDomain::User)?;
        // keyring 3.6's SecKeychainItem::delete discards OSStatus. SecItemDelete
        // preserves that failure without reading a secret or adding a second
        // authorization prompt solely to verify deletion. Scope the query to
        // the exact default user keychain, service and account used for writes.
        ItemSearchOptions::new()
            .keychains(&[keychain])
            .class(ItemClass::generic_password())
            .service(service)
            .account(account)
            .delete()
    })
}

fn delete_with(
    service: &str,
    account: &str,
    native_delete: impl FnOnce(&str, &str) -> security_framework::base::Result<()>,
) -> keyring::Result<()> {
    // Preserve keyring's validation before constructing a native query.
    MacCredential::new_with_target(/*target*/ None, service, account)?;
    native_delete(service, account).map_err(decode_error)
}

#[cfg(test)]
#[path = "macos_tests.rs"]
mod tests;
