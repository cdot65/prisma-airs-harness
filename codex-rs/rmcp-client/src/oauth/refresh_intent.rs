//! Retire a rotating grant durably before dispatch, using the existing storage format.
//!
//! A tokenless, expired record preserves the endpoint/client/issuer binding while making
//! interrupted exchanges unrefreshable, including for older binaries after rollback.
//! Only the owned transaction retains the old grant in memory. A successful exchange
//! replaces this record before exposing its returned credentials.

use anyhow::Result;
use codex_keyring_store::KeyringStore;
use oauth2::AccessToken;
use std::time::Duration;

use super::ResolvedOAuthCredentialStore;
use super::StoredOAuthTokens;

pub(super) fn pending_tokens(previous: &StoredOAuthTokens) -> StoredOAuthTokens {
    let mut pending = previous.clone();
    pending
        .token_response
        .0
        .set_access_token(AccessToken::new(String::new()));
    pending.token_response.0.set_refresh_token(None);
    pending
        .token_response
        .0
        .set_expires_in(Some(&Duration::ZERO));
    pending.expires_at = Some(0);
    pending
}

/// Retry persistence of exactly the returned generation; never repeat the exchange.
pub(super) fn commit_returned_tokens<K: KeyringStore + Clone + 'static>(
    store: ResolvedOAuthCredentialStore,
    keyring: &K,
    tokens: &StoredOAuthTokens,
) -> Result<()> {
    for delay in [Duration::from_millis(100), Duration::from_millis(400)] {
        if store.save(keyring, &tokens.server_name, tokens).is_ok() {
            return Ok(());
        }
        std::thread::sleep(delay);
    }
    store.save(keyring, &tokens.server_name, tokens)
}
